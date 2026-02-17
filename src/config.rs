use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub refresh_seconds: u64,
    pub repo_path: PathBuf,
    pub cache_seconds: u64,
    pub alerts: AlertsConfig,
    pub system_ui: SystemUiConfig,
    pub aws: AwsConfig,
    pub github: GitHubConfig,
    pub plugins: Vec<PluginConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AwsConfig {
    pub region: Option<String>,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GitHubConfig {
    pub repo: Option<String>,
    pub token_env: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AlertsConfig {
    pub cpu_warn_pct: f32,
    pub cpu_crit_pct: f32,
    pub mem_warn_pct: f32,
    pub mem_crit_pct: f32,
    pub stale_warn_secs: f32,
    pub stale_crit_secs: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SystemUiConfig {
    pub layout_mode: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub shell: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_seconds: 5,
            repo_path: PathBuf::from("."),
            cache_seconds: 120,
            alerts: AlertsConfig::default(),
            system_ui: SystemUiConfig::default(),
            aws: AwsConfig::default(),
            github: GitHubConfig::default(),
            plugins: vec![],
        }
    }
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            region: None,
            profile: None,
        }
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            repo: None,
            token_env: "GITHUB_TOKEN".to_string(),
        }
    }
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            cpu_warn_pct: 65.0,
            cpu_crit_pct: 85.0,
            mem_warn_pct: 70.0,
            mem_crit_pct: 88.0,
            stale_warn_secs: 2.5,
            stale_crit_secs: 5.0,
        }
    }
}

impl Default for SystemUiConfig {
    fn default() -> Self {
        Self {
            layout_mode: "auto".to_string(),
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: "plugin".to_string(),
            command: String::new(),
            args: vec![],
            shell: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut candidates = vec![PathBuf::from("devdash.toml")];

        if let Some(mut dir) = dirs::config_dir() {
            dir.push("devdash");
            dir.push("config.toml");
            candidates.push(dir);
        }

        for path in candidates {
            if path.exists() {
                return Self::from_file(&path)
                    .with_context(|| format!("failed loading config from {}", path.display()));
            }
        }

        Ok(Self::default())
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)?;
        let cfg = toml::from_str::<Self>(&raw)?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        if self.alerts.cpu_warn_pct >= self.alerts.cpu_crit_pct {
            anyhow::bail!(
                "cpu_warn_pct ({}) must be < cpu_crit_pct ({})",
                self.alerts.cpu_warn_pct,
                self.alerts.cpu_crit_pct
            );
        }
        if self.alerts.mem_warn_pct >= self.alerts.mem_crit_pct {
            anyhow::bail!(
                "mem_warn_pct ({}) must be < mem_crit_pct ({})",
                self.alerts.mem_warn_pct,
                self.alerts.mem_crit_pct
            );
        }
        if self.alerts.stale_warn_secs >= self.alerts.stale_crit_secs {
            anyhow::bail!(
                "stale_warn_secs ({}) must be < stale_crit_secs ({})",
                self.alerts.stale_warn_secs,
                self.alerts.stale_crit_secs
            );
        }
        Ok(())
    }
}
