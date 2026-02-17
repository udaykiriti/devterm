use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use sysinfo::{ProcessesToUpdate, System};
use tokio::process::Command;

use crate::config::Config;
use crate::plugin::PluginManager;

use super::types::{
    AwsInstance, AwsStatus, DashboardData, DockerContainer, DockerStatus, GitStatus, PrItem,
    PrStatus, ProcessStat, SystemStatus,
};

#[derive(Debug, Deserialize)]
struct GitHubPull {
    number: u64,
    title: String,
    user: GitHubUser,
    html_url: String,
    updated_at: String,
    body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

pub async fn collect_all(cfg: &Config, plugins: &PluginManager) -> DashboardData {
    let (git, system, docker, aws, prs, plugin_data) = tokio::join!(
        collect_git(cfg),
        collect_system(),
        collect_docker(),
        collect_aws(cfg),
        collect_prs(cfg),
        plugins.collect_all(),
    );

    DashboardData {
        git,
        system,
        docker,
        aws,
        prs,
        plugins: plugin_data,
        last_update: Some(Utc::now()),
    }
}

async fn collect_git(cfg: &Config) -> GitStatus {
    let output = run_cmd(
        "git",
        vec![
            "-C".to_string(),
            cfg.repo_path.display().to_string(),
            "status".to_string(),
            "--porcelain".to_string(),
            "--branch".to_string(),
        ],
    )
    .await;

    let text = match output {
        Ok(v) => v,
        Err(_) => return GitStatus::default(),
    };

    let mut status = GitStatus::default();
    for (idx, line) in text.lines().enumerate() {
        if idx == 0 && line.starts_with("##") {
            let branch_info = line.trim_start_matches("## ");
            let mut parts = branch_info.split("...");
            status.branch = parts.next().unwrap_or("unknown").to_string();
            if let Some(remote_part) = parts.next() {
                status.ahead_behind = remote_part
                    .split('[')
                    .nth(1)
                    .map(|v| v.trim_end_matches(']'))
                    .unwrap_or("")
                    .to_string();
            }
            continue;
        }

        if line.len() < 2 {
            continue;
        }
        
        let x = line.as_bytes()[0] as char;
        let y = line.as_bytes()[1] as char;

        if x != ' ' && x != '?' {
            status.staged += 1;
        }
        if y != ' ' {
            status.unstaged += 1;
        }
        if x == '?' && y == '?' {
            status.untracked += 1;
        }
    }

    status
}

async fn collect_system() -> SystemStatus {
    static SAMPLER: OnceLock<Mutex<System>> = OnceLock::new();
    let sampler = SAMPLER.get_or_init(|| {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        Mutex::new(sys)
    });

    let mut sys = match sampler.lock() {
        Ok(sys) => sys,
        Err(poisoned) => {
            eprintln!("Warning: System sampler mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };

    sys.refresh_cpu_usage();
    sys.refresh_memory();
    let _ = sys.refresh_processes(ProcessesToUpdate::All, false);

    let load = System::load_average();
    
    // Collect disk information
    let mut disk_total_gb = 0.0;
    let mut disk_used_gb = 0.0;
    for disk in sysinfo::Disks::new_with_refreshed_list().iter() {
        disk_total_gb += disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used = disk.total_space() - disk.available_space();
        disk_used_gb += used as f64 / 1024.0 / 1024.0 / 1024.0;
    }
    
    // Collect network information
    let mut network_rx_mb = 0.0;
    let mut network_tx_mb = 0.0;
    for (_interface_name, network) in sysinfo::Networks::new_with_refreshed_list().iter() {
        network_rx_mb += network.total_received() as f64 / 1024.0 / 1024.0;
        network_tx_mb += network.total_transmitted() as f64 / 1024.0 / 1024.0;
    }
    
    let mut top_processes = sys
        .processes()
        .values()
        .map(|p| ProcessStat {
            pid: p.pid().to_string(),
            name: p.name().to_string_lossy().to_string(),
            command: p
                .cmd()
                .iter()
                .take(8)
                .map(|s| s.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" "),
            runtime_secs: p.run_time(),
            cpu_pct: p.cpu_usage(),
            mem_mb: p.memory() as f64 / 1024.0 / 1024.0,
            read_mb: p.disk_usage().total_read_bytes as f64 / 1024.0 / 1024.0,
            write_mb: p.disk_usage().total_written_bytes as f64 / 1024.0 / 1024.0,
        })
        .collect::<Vec<_>>();
    top_processes.sort_by(|a, b| {
        b.cpu_pct
            .partial_cmp(&a.cpu_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.mem_mb
                    .partial_cmp(&a.mem_mb)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    top_processes.truncate(3);

    SystemStatus {
        cpu_usage: sys.global_cpu_usage(),
        cpu_cores: sys
            .cpus()
            .iter()
            .take(16)
            .map(|cpu| cpu.cpu_usage())
            .collect(),
        mem_used_gb: sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
        mem_total_gb: sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
        mem_available_gb: sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
        swap_used_gb: sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0,
        swap_total_gb: sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0,
        load_avg: (load.one, load.five, load.fifteen),
        uptime_secs: System::uptime(),
        process_count: sys.processes().len(),
        top_processes,
        disk_total_gb,
        disk_used_gb,
        network_rx_mb,
        network_tx_mb,
    }
}

async fn collect_docker() -> DockerStatus {
    match run_cmd(
        "docker",
        vec![
            "ps".to_string(),
            "--format".to_string(),
            "{{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Image}}\t{{.Ports}}".to_string(),
        ],
    )
    .await
    {
        Ok(raw) => {
            let mut items = Vec::new();
            for line in raw.lines().take(12) {
                let parts = line.split('\t').collect::<Vec<_>>();
                let id = parts.first().copied().unwrap_or("").to_string();
                let name = parts.get(1).copied().unwrap_or("unknown").to_string();
                let status = parts.get(2).copied().unwrap_or("unknown").to_string();
                let image = parts.get(3).copied().unwrap_or("unknown").to_string();
                let ports = parts.get(4).copied().unwrap_or("none").to_string();
                items.push(DockerContainer {
                    id,
                    name,
                    status,
                    image,
                    ports,
                });
            }

            let mut running = items
                .iter()
                .take(8)
                .map(|c| format!("{}\t{}\t{}", c.name, c.status, c.image))
                .collect::<Vec<_>>();

            if running.is_empty() {
                running.push("no running containers".to_string());
            }

            DockerStatus {
                running,
                items,
                error: None,
            }
        }
        Err(e) => DockerStatus {
            running: vec![],
            items: vec![],
            error: Some(e),
        },
    }
}

async fn collect_aws(cfg: &Config) -> AwsStatus {
    let mut args = vec!["ec2".to_string(), "describe-instances".to_string()];

    let region = cfg
        .aws
        .region
        .clone()
        .or_else(|| std::env::var("AWS_REGION").ok())
        .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok());

    let profile = cfg
        .aws
        .profile
        .clone()
        .or_else(|| std::env::var("AWS_PROFILE").ok());

    if let Some(r) = &region {
        args.push("--region".to_string());
        args.push(r.clone());
    }
    if let Some(p) = &profile {
        args.push("--profile".to_string());
        args.push(p.clone());
    }

    args.push("--query".to_string());
    args.push("Reservations[].Instances[].[InstanceId,State.Name,Tags[?Key=='Name']|[0].Value,InstanceType,Placement.AvailabilityZone,PublicIpAddress,PrivateIpAddress]".to_string());
    args.push("--output".to_string());
    args.push("json".to_string());

    let source = match (&profile, &region) {
        (Some(p), Some(r)) => format!("aws-cli (profile={p}, region={r})"),
        (Some(p), None) => format!("aws-cli (profile={p})"),
        (None, Some(r)) => format!("aws-cli (region={r})"),
        (None, None) => "aws-cli".to_string(),
    };

    match run_cmd("aws", args).await {
        Ok(raw) => {
            let parsed: Result<Value, _> = serde_json::from_str(&raw);
            match parsed {
                Ok(Value::Array(items)) => {
                    let mut rows = Vec::new();
                    for row in items.into_iter().take(12) {
                        if let Value::Array(fields) = row {
                            rows.push(AwsInstance {
                                id: fields
                                    .first()
                                    .and_then(Value::as_str)
                                    .unwrap_or("unknown")
                                    .to_string(),
                                state: fields
                                    .get(1)
                                    .and_then(Value::as_str)
                                    .unwrap_or("?")
                                    .to_string(),
                                name: fields
                                    .get(2)
                                    .and_then(Value::as_str)
                                    .unwrap_or("unnamed")
                                    .to_string(),
                                instance_type: fields
                                    .get(3)
                                    .and_then(Value::as_str)
                                    .unwrap_or("unknown")
                                    .to_string(),
                                az: fields
                                    .get(4)
                                    .and_then(Value::as_str)
                                    .unwrap_or("unknown")
                                    .to_string(),
                                public_ip: fields
                                    .get(5)
                                    .and_then(Value::as_str)
                                    .unwrap_or("none")
                                    .to_string(),
                                private_ip: fields
                                    .get(6)
                                    .and_then(Value::as_str)
                                    .unwrap_or("none")
                                    .to_string(),
                            });
                        }
                    }

                    let mut instances = rows
                        .iter()
                        .take(8)
                        .map(|i| format!("{} | {} | {}", i.id, i.state, i.name))
                        .collect::<Vec<_>>();

                    if instances.is_empty() {
                        instances.push("no instances found".to_string());
                    }

                    AwsStatus {
                        instances,
                        items: rows,
                        source,
                        error: None,
                    }
                }
                Ok(_) => AwsStatus {
                    instances: vec![],
                    items: vec![],
                    source,
                    error: Some("unexpected AWS response shape".to_string()),
                },
                Err(e) => AwsStatus {
                    instances: vec![],
                    items: vec![],
                    source,
                    error: Some(format!("AWS parse error: {e}")),
                },
            }
        }
        Err(e) => AwsStatus {
            instances: vec![],
            items: vec![],
            source,
            error: Some(format_aws_error(&e)),
        },
    }
}

fn format_aws_error(err: &str) -> String {
    let lower = err.to_lowercase();
    if lower.contains("unable to locate credentials") {
        return "AWS auth missing. Run `aws configure`, set AWS_PROFILE, or configure SSO."
            .to_string();
    }
    if lower.contains("you must specify a region") {
        return "AWS region missing. Set [aws].region in devdash.toml or AWS_REGION.".to_string();
    }
    if lower.contains("could not connect") || lower.contains("timed out") {
        return "AWS network/API unreachable.".to_string();
    }
    err.to_string()
}

async fn collect_prs(cfg: &Config) -> PrStatus {
    if let Some(repo) = &cfg.github.repo {
        let token = std::env::var(&cfg.github.token_env).ok();
        if let Ok(status) = collect_prs_github_api(repo, token, &cfg.github.token_env).await {
            return status;
        }
    }

    collect_prs_via_gh(cfg).await
}

async fn collect_prs_github_api(
    repo: &str,
    token: Option<String>,
    token_env: &str,
) -> Result<PrStatus, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{repo}/pulls?state=open&per_page=10");

    let mut req = client
        .get(url)
        .header("User-Agent", "devdash")
        .header("Accept", "application/vnd.github+json");

    let source = if let Some(t) = token {
        req = req.bearer_auth(t);
        format!("github-api ({token_env})")
    } else {
        "github-api (unauthenticated)".to_string()
    };

    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("GitHub API {}", response.status()));
    }

    let pulls = response
        .json::<Vec<GitHubPull>>()
        .await
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    let mut open = Vec::new();
    for pr in pulls.into_iter().take(10) {
        open.push(format!("#{} {} (@{})", pr.number, pr.title, pr.user.login));
        items.push(PrItem {
            number: pr.number,
            title: pr.title,
            author: pr.user.login,
            url: pr.html_url,
            updated_at: pr.updated_at,
            body: pr.body,
        });
    }

    if open.is_empty() {
        open.push("no open PRs".to_string());
    }

    Ok(PrStatus {
        open,
        items,
        source,
        error: None,
    })
}

async fn collect_prs_via_gh(cfg: &Config) -> PrStatus {
    match run_cmd(
        "gh",
        vec![
            "pr".to_string(),
            "list".to_string(),
            "--state".to_string(),
            "open".to_string(),
            "--limit".to_string(),
            "10".to_string(),
            "--json".to_string(),
            "number,title,author,url,updatedAt".to_string(),
        ],
    )
    .await
    {
        Ok(raw) => {
            let parsed: Result<Value, _> = serde_json::from_str(&raw);
            match parsed {
                Ok(Value::Array(items)) => {
                    let mut open = Vec::new();
                    let mut rows = Vec::new();
                    for item in items.into_iter().take(10) {
                        let number = item.get("number").and_then(Value::as_u64).unwrap_or(0);
                        let title = item
                            .get("title")
                            .and_then(Value::as_str)
                            .unwrap_or("untitled")
                            .to_string();
                        let author = item
                            .get("author")
                            .and_then(|v| v.get("login"))
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string();
                        let url = item
                            .get("url")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let updated_at = item
                            .get("updatedAt")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();

                        open.push(format!("#{number} {title} (@{author})"));
                        rows.push(PrItem {
                            number,
                            title,
                            author,
                            url,
                            updated_at,
                            body: None,
                        });
                    }
                    if open.is_empty() {
                        open.push("no open PRs".to_string());
                    }
                    PrStatus {
                        open,
                        items: rows,
                        source: "gh-cli".to_string(),
                        error: None,
                    }
                }
                Ok(_) => PrStatus {
                    open: vec![],
                    items: vec![],
                    source: "gh-cli".to_string(),
                    error: Some("unexpected gh response shape".to_string()),
                },
                Err(e) => PrStatus {
                    open: vec![],
                    items: vec![],
                    source: "gh-cli".to_string(),
                    error: Some(format!("gh parse error: {e}")),
                },
            }
        }
        Err(e) => PrStatus {
            open: vec![],
            items: vec![],
            source: "none".to_string(),
            error: Some(format!(
                "PR auth/setup needed. Configure github.repo + {} or run `gh auth login` ({e})",
                cfg.github.token_env
            )),
        },
    }
}

async fn run_cmd(cmd: &str, args: Vec<String>) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if err.is_empty() {
            format!("{cmd} failed with {}", output.status)
        } else {
            err
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
