use chrono::{DateTime, Utc};

use crate::plugin::PluginOutput;

#[derive(Debug, Clone, Default)]
pub struct DashboardData {
    pub git: GitStatus,
    pub system: SystemStatus,
    pub docker: DockerStatus,
    pub aws: AwsStatus,
    pub prs: PrStatus,
    pub plugins: Vec<PluginOutput>,
    pub last_update: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub ahead_behind: String,
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
}

impl Default for GitStatus {
    fn default() -> Self {
        Self {
            branch: "n/a".into(),
            ahead_behind: String::new(),
            staged: 0,
            unstaged: 0,
            untracked: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub cpu_usage: f32,
    pub cpu_cores: Vec<f32>,
    pub mem_used_gb: f64,
    pub mem_total_gb: f64,
    pub mem_available_gb: f64,
    pub swap_used_gb: f64,
    pub swap_total_gb: f64,
    pub load_avg: (f64, f64, f64),
    pub uptime_secs: u64,
    pub process_count: usize,
    pub top_processes: Vec<ProcessStat>,
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
    pub network_rx_mb: f64,
    pub network_tx_mb: f64,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            cpu_cores: vec![],
            mem_used_gb: 0.0,
            mem_total_gb: 0.0,
            mem_available_gb: 0.0,
            swap_used_gb: 0.0,
            swap_total_gb: 0.0,
            load_avg: (0.0, 0.0, 0.0),
            uptime_secs: 0,
            process_count: 0,
            top_processes: vec![],
            disk_total_gb: 0.0,
            disk_used_gb: 0.0,
            network_rx_mb: 0.0,
            network_tx_mb: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProcessStat {
    pub pid: String,
    pub name: String,
    pub command: String,
    pub runtime_secs: u64,
    pub cpu_pct: f32,
    pub mem_mb: f64,
    pub read_mb: f64,
    pub write_mb: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub status: String,
    pub image: String,
    pub ports: String,
}

#[derive(Debug, Clone, Default)]
pub struct DockerStatus {
    pub running: Vec<String>,
    pub items: Vec<DockerContainer>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AwsInstance {
    pub id: String,
    pub state: String,
    pub name: String,
    pub instance_type: String,
    pub az: String,
    pub public_ip: String,
    pub private_ip: String,
}

#[derive(Debug, Clone, Default)]
pub struct AwsStatus {
    pub instances: Vec<String>,
    pub items: Vec<AwsInstance>,
    pub source: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PrItem {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub url: String,
    pub updated_at: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PrStatus {
    pub open: Vec<String>,
    pub items: Vec<PrItem>,
    pub source: String,
    pub error: Option<String>,
}
