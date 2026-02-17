use chrono::{DateTime, Utc};

use crate::collectors::DashboardData;
use crate::config::Config;

use super::types::{
    DetailModal, LayoutState, ListCursorState, NavDir, PaletteCommand, Pane, SystemAlerts,
    SystemLayoutMode, parse_pane,
};

#[derive(Debug)]
pub struct App {
    pub selected: Pane,
    pub data: DashboardData,
    pub last_error: Option<String>,
    pub cpu_history: Vec<u64>,
    pub mem_history: Vec<u64>,
    pub disk_history: Vec<u64>,
    pub layout: LayoutState,
    pub compact_mode: bool,
    pub command_mode: bool,
    pub command_input: String,
    pub status: Option<String>,
    pub cursors: ListCursorState,
    pub detail_modal: Option<DetailModal>,
    pub loading: bool,
    pub spinner_index: usize,
    pub system_flash_ticks: u8,
    pub cpu_delta: f32,
    pub mem_delta: f64,
    pub disk_delta: f64,
    pub cpu_peak: f32,
    pub mem_peak: f32,
    pub disk_peak: f32,
    pub peak_hold_ticks: u8,
    pub system_layout_mode: SystemLayoutMode,
    pub system_alerts: SystemAlerts,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected: Pane::Git,
            data: DashboardData::default(),
            last_error: None,
            cpu_history: vec![0; 32],
            mem_history: vec![0; 32],
            disk_history: vec![0; 32],
            layout: LayoutState::default(),
            compact_mode: false,
            command_mode: false,
            command_input: String::new(),
            status: None,
            cursors: ListCursorState::default(),
            detail_modal: None,
            loading: true,
            spinner_index: 0,
            system_flash_ticks: 0,
            cpu_delta: 0.0,
            mem_delta: 0.0,
            disk_delta: 0.0,
            cpu_peak: 0.0,
            mem_peak: 0.0,
            disk_peak: 0.0,
            peak_hold_ticks: 0,
            system_layout_mode: SystemLayoutMode::Auto,
            system_alerts: SystemAlerts::default(),
        }
    }

    pub fn apply_config(&mut self, cfg: &Config) {
        self.system_layout_mode = parse_layout_mode(&cfg.system_ui.layout_mode);
        self.system_alerts = SystemAlerts {
            cpu_warn_pct: cfg.alerts.cpu_warn_pct,
            cpu_crit_pct: cfg.alerts.cpu_crit_pct,
            mem_warn_pct: cfg.alerts.mem_warn_pct,
            mem_crit_pct: cfg.alerts.mem_crit_pct,
            stale_warn_secs: cfg.alerts.stale_warn_secs,
            stale_crit_secs: cfg.alerts.stale_crit_secs,
        };
    }

    pub fn select_next(&mut self) {
        let idx = Pane::ALL
            .iter()
            .position(|pane| *pane == self.selected)
            .unwrap_or(0);
        self.selected = Pane::ALL[(idx + 1) % Pane::ALL.len()];
    }

    pub fn select_prev(&mut self) {
        let idx = Pane::ALL
            .iter()
            .position(|pane| *pane == self.selected)
            .unwrap_or(0);
        self.selected = Pane::ALL[(idx + Pane::ALL.len() - 1) % Pane::ALL.len()];
    }

    pub fn select_directional(&mut self, dir: NavDir) {
        self.selected = match (self.selected, dir) {
            (Pane::Git, NavDir::Right) => Pane::System,
            (Pane::System, NavDir::Right) => Pane::Prs,
            (Pane::Prs, NavDir::Right) => Pane::Prs,
            (Pane::Docker, NavDir::Right) => Pane::Aws,
            (Pane::Aws, NavDir::Right) => Pane::Plugins,
            (Pane::Plugins, NavDir::Right) => Pane::Plugins,

            (Pane::Prs, NavDir::Left) => Pane::System,
            (Pane::System, NavDir::Left) => Pane::Git,
            (Pane::Git, NavDir::Left) => Pane::Git,
            (Pane::Plugins, NavDir::Left) => Pane::Aws,
            (Pane::Aws, NavDir::Left) => Pane::Docker,
            (Pane::Docker, NavDir::Left) => Pane::Docker,

            (Pane::Git, NavDir::Down) => Pane::Docker,
            (Pane::System, NavDir::Down) => Pane::Aws,
            (Pane::Prs, NavDir::Down) => Pane::Plugins,
            (pane, NavDir::Down) => pane,

            (Pane::Docker, NavDir::Up) => Pane::Git,
            (Pane::Aws, NavDir::Up) => Pane::System,
            (Pane::Plugins, NavDir::Up) => Pane::Prs,
            (pane, NavDir::Up) => pane,
        };
    }

    pub fn update_data(&mut self, data: DashboardData) {
        let prev_cpu = self.data.system.cpu_usage;
        let prev_mem_pct = if self.data.system.mem_total_gb > 0.0 {
            (self.data.system.mem_used_gb / self.data.system.mem_total_gb) * 100.0
        } else {
            0.0
        };
        let prev_disk_pct = if self.data.system.disk_total_gb > 0.0 {
            (self.data.system.disk_used_gb / self.data.system.disk_total_gb) * 100.0
        } else {
            0.0
        };

        let cpu = data.system.cpu_usage.clamp(0.0, 100.0) as u64;
        let mem_pct = if data.system.mem_total_gb > 0.0 {
            ((data.system.mem_used_gb / data.system.mem_total_gb) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        } as u64;
        let disk_pct = if data.system.disk_total_gb > 0.0 {
            ((data.system.disk_used_gb / data.system.disk_total_gb) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        } as u64;

        push_bounded(&mut self.cpu_history, cpu, 64);
        push_bounded(&mut self.mem_history, mem_pct, 64);
        push_bounded(&mut self.disk_history, disk_pct, 64);

        self.data = data;
        self.normalize_cursors();
        self.last_error = None;
        self.loading = false;
        self.cpu_delta = self.data.system.cpu_usage - prev_cpu;
        self.mem_delta = mem_pct as f64 - prev_mem_pct;
        self.disk_delta = disk_pct as f64 - prev_disk_pct;
        self.system_flash_ticks = 7;
        self.cpu_peak = self.cpu_peak.max(self.data.system.cpu_usage);
        self.mem_peak = self.mem_peak.max(mem_pct as f32);
        self.disk_peak = self.disk_peak.max(disk_pct as f32);
        self.peak_hold_ticks = 24;
    }

    pub fn set_error(&mut self, err: impl Into<String>) {
        self.last_error = Some(err.into());
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = Some(msg.into());
    }

    pub fn last_update(&self) -> Option<DateTime<Utc>> {
        self.data.last_update
    }

    pub fn resize_focused(&mut self, delta: i16) {
        let step = if delta < 0 { -2 } else { 2 };
        match self.selected {
            Pane::Git | Pane::System | Pane::Prs => {
                adjust_three_cols(&mut self.layout.top_cols_pct, self.selected, step)
            }
            Pane::Docker | Pane::Aws | Pane::Plugins => {
                adjust_three_cols(&mut self.layout.bottom_cols_pct, self.selected, step)
            }
        }
    }

    pub fn resize_rows(&mut self, delta: i16) {
        let next = self.layout.top_height_pct as i16 + if delta < 0 { -2 } else { 2 };
        self.layout.top_height_pct = next.clamp(35, 75) as u16;
    }

    pub fn enter_command_mode(&mut self) {
        self.command_mode = true;
        self.command_input.clear();
    }

    pub fn exit_command_mode(&mut self) {
        self.command_mode = false;
        self.command_input.clear();
    }

    pub fn parse_command(&self) -> Result<PaletteCommand, String> {
        let raw = self.command_input.trim();
        let mut parts = raw.split_whitespace();
        let cmd = parts.next().ok_or("empty command")?;

        match cmd {
            "refresh" | "r" => Ok(PaletteCommand::Refresh),
            "reload" => Ok(PaletteCommand::ReloadConfig),
            "compact" => Ok(PaletteCommand::ToggleCompact),
            "quit" | "q" | "exit" => Ok(PaletteCommand::Quit),
            "focus" | "f" => {
                let target = parts.next().ok_or("usage: focus <pane>")?;
                let pane = parse_pane(target).ok_or("unknown pane")?;
                Ok(PaletteCommand::Focus(pane))
            }
            "help" | "h" => Ok(PaletteCommand::Help),
            _ => Err(format!("unknown command: {cmd}")),
        }
    }

    pub fn can_scroll_list(&self) -> bool {
        self.list_len_for(self.selected) > 0
    }

    pub fn move_list_cursor(&mut self, delta: i32) {
        let len = self.list_len_for(self.selected);
        if len == 0 {
            return;
        }

        let cursor = self.cursor_mut_for(self.selected);
        let current = *cursor as isize;
        let max = len.saturating_sub(1) as isize;
        let next = (current + delta as isize).clamp(0, max) as usize;
        *cursor = next;
    }

    pub fn current_list_cursor(&self, pane: Pane) -> Option<usize> {
        let len = self.list_len_for(pane);
        if len == 0 {
            return None;
        }

        Some(match pane {
            Pane::System => self.cursors.system.min(len - 1),
            Pane::Prs => self.cursors.prs.min(len - 1),
            Pane::Docker => self.cursors.docker.min(len - 1),
            Pane::Aws => self.cursors.aws.min(len - 1),
            Pane::Plugins => self.cursors.plugins.min(len - 1),
            _ => 0,
        })
    }

    pub fn open_details_for_selected(&mut self) {
        let selected_idx = match self.current_list_cursor(self.selected) {
            Some(v) => v,
            None => {
                self.set_status("nothing to inspect in this pane");
                return;
            }
        };

        let detail = match self.selected {
            Pane::System => {
                let Some(p) = self.data.system.top_processes.get(selected_idx) else {
                    return;
                };

                DetailModal {
                    title: format!("Process {} ({})", p.name, p.pid),
                    lines: vec![
                        format!("pid: {}", p.pid),
                        format!("name: {}", p.name),
                        format!(
                            "command: {}",
                            if p.command.is_empty() {
                                "n/a"
                            } else {
                                &p.command
                            }
                        ),
                        format!("cpu: {:.1}%", p.cpu_pct),
                        format!("memory: {:.1} MB", p.mem_mb),
                        format!("runtime: {}", format_duration_short(p.runtime_secs)),
                        format!("io read: {:.1} MB", p.read_mb),
                        format!("io write: {:.1} MB", p.write_mb),
                    ],
                }
            }
            Pane::Prs => {
                let Some(pr) = self.data.prs.items.get(selected_idx) else {
                    return;
                };

                let mut lines = vec![
                    format!("#{} {}", pr.number, pr.title),
                    format!("author: {}", pr.author),
                    format!("updated: {}", blank_if_empty(&pr.updated_at)),
                    format!("url: {}", blank_if_empty(&pr.url)),
                    format!("source: {}", self.data.prs.source),
                ];
                if let Some(body) = &pr.body {
                    lines.push(String::new());
                    lines.push("body:".to_string());
                    for l in body.lines().take(8) {
                        lines.push(l.to_string());
                    }
                }

                DetailModal {
                    title: format!("PR #{}", pr.number),
                    lines,
                }
            }
            Pane::Docker => {
                let Some(c) = self.data.docker.items.get(selected_idx) else {
                    return;
                };

                DetailModal {
                    title: format!("Container {}", c.name),
                    lines: vec![
                        format!("id: {}", blank_if_empty(&c.id)),
                        format!("name: {}", c.name),
                        format!("status: {}", c.status),
                        format!("image: {}", c.image),
                        format!("ports: {}", blank_if_empty(&c.ports)),
                    ],
                }
            }
            Pane::Aws => {
                let Some(i) = self.data.aws.items.get(selected_idx) else {
                    return;
                };

                DetailModal {
                    title: format!("EC2 {}", i.id),
                    lines: vec![
                        format!("name: {}", i.name),
                        format!("state: {}", i.state),
                        format!("type: {}", i.instance_type),
                        format!("az: {}", i.az),
                        format!("public ip: {}", i.public_ip),
                        format!("private ip: {}", i.private_ip),
                        format!("source: {}", self.data.aws.source),
                    ],
                }
            }
            Pane::Plugins => {
                let Some(p) = self.data.plugins.get(selected_idx) else {
                    return;
                };

                let mut lines = vec![format!("name: {}", p.name)];
                if let Some(err) = &p.error {
                    lines.push(format!("error: {err}"));
                } else {
                    lines.extend(p.lines.iter().cloned());
                }

                DetailModal {
                    title: format!("Plugin {}", p.name),
                    lines,
                }
            }
            _ => return,
        };

        self.detail_modal = Some(detail);
    }

    pub fn close_details(&mut self) {
        self.detail_modal = None;
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_index = (self.spinner_index + 1) % 8;
        if self.system_flash_ticks > 0 {
            self.system_flash_ticks -= 1;
        }
        if self.peak_hold_ticks > 0 {
            self.peak_hold_ticks -= 1;
        } else {
            self.cpu_peak = (self.cpu_peak - 0.4).max(0.0);
            self.mem_peak = (self.mem_peak - 0.25).max(0.0);
        }
    }

    fn normalize_cursors(&mut self) {
        self.cursors.system =
            clamp_cursor(self.cursors.system, self.data.system.top_processes.len());
        self.cursors.prs = clamp_cursor(self.cursors.prs, self.data.prs.open.len());
        self.cursors.docker = clamp_cursor(self.cursors.docker, self.data.docker.running.len());
        self.cursors.aws = clamp_cursor(self.cursors.aws, self.data.aws.instances.len());
        self.cursors.plugins = clamp_cursor(self.cursors.plugins, self.data.plugins.len());
    }

    fn list_len_for(&self, pane: Pane) -> usize {
        match pane {
            Pane::System => self.data.system.top_processes.len(),
            Pane::Prs => self.data.prs.open.len(),
            Pane::Docker => self.data.docker.running.len(),
            Pane::Aws => self.data.aws.instances.len(),
            Pane::Plugins => self.data.plugins.len(),
            _ => 0,
        }
    }

    fn cursor_mut_for(&mut self, pane: Pane) -> &mut usize {
        match pane {
            Pane::System => &mut self.cursors.system,
            Pane::Prs => &mut self.cursors.prs,
            Pane::Docker => &mut self.cursors.docker,
            Pane::Aws => &mut self.cursors.aws,
            Pane::Plugins => &mut self.cursors.plugins,
            _ => &mut self.cursors.prs,
        }
    }
}

fn blank_if_empty(s: &str) -> &str {
    if s.trim().is_empty() { "n/a" } else { s }
}

fn clamp_cursor(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { current.min(len - 1) }
}

fn adjust_three_cols(cols: &mut [u16; 3], selected: Pane, step: i16) {
    let idx = match selected {
        Pane::Git | Pane::Docker => 0,
        Pane::System | Pane::Aws => 1,
        Pane::Prs | Pane::Plugins => 2,
    };

    let target = cols[idx] as i16 + step;
    if !(20..=60).contains(&target) {
        return;
    }

    let donor_idx = if idx == 2 { 1 } else { 2 };
    let donor = cols[donor_idx] as i16 - step;
    if !(20..=60).contains(&donor) {
        return;
    }

    cols[idx] = target as u16;
    cols[donor_idx] = donor as u16;

    let sum = cols[0] + cols[1] + cols[2];
    if sum != 100 {
        let diff = 100_i16 - sum as i16;
        let pivot_idx = if donor_idx == 1 { 0 } else { 1 };
        let pivot = cols[pivot_idx] as i16 + diff;
        cols[pivot_idx] = pivot.clamp(20, 60) as u16;
    }
}

fn push_bounded(buf: &mut Vec<u64>, value: u64, max: usize) {
    buf.push(value);
    if buf.len() > max {
        let overflow = buf.len() - max;
        buf.drain(0..overflow);
    }
}

fn parse_layout_mode(raw: &str) -> SystemLayoutMode {
    match raw.trim().to_ascii_lowercase().as_str() {
        "compact" => SystemLayoutMode::Compact,
        "cockpit" => SystemLayoutMode::Cockpit,
        _ => SystemLayoutMode::Auto,
    }
}

fn format_duration_short(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours:02}h")
    } else {
        format!("{hours:02}h {mins:02}m")
    }
}
