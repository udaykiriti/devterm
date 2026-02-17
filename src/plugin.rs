use std::future::Future;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;

use tokio::process::Command;

use crate::config::PluginConfig;

/// Plugin output data structure
#[derive(Debug, Clone, Default)]
pub struct PluginOutput {
    pub name: String,
    pub lines: Vec<String>,
    pub error: Option<String>,
}

/// Trait for implementing dashboard plugins
pub trait DashboardPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn collect<'a>(&'a self) -> Pin<Box<dyn Future<Output = PluginOutput> + Send + 'a>>;
}

/// Manager for running multiple plugins
#[derive(Clone)]
pub struct PluginManager {
    plugins: Vec<Arc<dyn DashboardPlugin>>,
}

impl PluginManager {
    /// Create plugin manager from configuration
    pub fn from_config(cfgs: &[PluginConfig]) -> Self {
        let mut plugins: Vec<Arc<dyn DashboardPlugin>> = Vec::new();
        for cfg in cfgs {
            if cfg.command.trim().is_empty() {
                continue;
            }
            plugins.push(Arc::new(CommandPlugin::new(cfg.clone())));
        }
        Self { plugins }
    }

    /// Run all plugins and collect their outputs
    pub async fn collect_all(&self) -> Vec<PluginOutput> {
        let mut out = Vec::with_capacity(self.plugins.len());
        for plugin in &self.plugins {
            out.push(plugin.collect().await);
        }
        out
    }
}

/// Plugin implementation that executes shell commands
struct CommandPlugin {
    cfg: PluginConfig,
}

impl CommandPlugin {
    fn new(cfg: PluginConfig) -> Self {
        Self { cfg }
    }
}

impl DashboardPlugin for CommandPlugin {
    fn name(&self) -> &str {
        &self.cfg.name
    }

    fn collect<'a>(&'a self) -> Pin<Box<dyn Future<Output = PluginOutput> + Send + 'a>> {
        Box::pin(async move {
            // Build command with or without shell wrapper
            let mut cmd = if self.cfg.shell {
                let mut c = Command::new("bash");
                let joined = if self.cfg.args.is_empty() {
                    self.cfg.command.clone()
                } else {
                    format!("{} {}", self.cfg.command, self.cfg.args.join(" "))
                };
                c.args(["-lc", &joined]);
                c
            } else {
                let mut c = Command::new(&self.cfg.command);
                c.args(&self.cfg.args);
                c
            };

            // Execute with 30-second timeout
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                cmd
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
            )
            .await;

            // Parse result and format output
            match result {
                Ok(Ok(output)) if output.status.success() => {
                    let text = String::from_utf8_lossy(&output.stdout);
                    let mut lines: Vec<String> = text
                        .lines()
                        .take(12)
                        .map(std::string::ToString::to_string)
                        .collect();
                    if lines.is_empty() {
                        lines.push("(no output)".to_string());
                    }
                    PluginOutput {
                        name: self.name().to_string(),
                        lines,
                        error: None,
                    }
                }
                Ok(Ok(output)) => {
                    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    PluginOutput {
                        name: self.name().to_string(),
                        lines: vec![],
                        error: Some(if err.is_empty() {
                            format!("exit {}", output.status)
                        } else {
                            err
                        }),
                    }
                }
                Ok(Err(e)) => PluginOutput {
                    name: self.name().to_string(),
                    lines: vec![],
                    error: Some(e.to_string()),
                },
                Err(_) => PluginOutput {
                    name: self.name().to_string(),
                    lines: vec![],
                    error: Some("timeout (30s)".to_string()),
                },
            }
        })
    }
}
