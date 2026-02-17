#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Git,
    System,
    Prs,
    Docker,
    Aws,
    Plugins,
}

impl Pane {
    pub const ALL: [Pane; 6] = [
        Pane::Git,
        Pane::System,
        Pane::Prs,
        Pane::Docker,
        Pane::Aws,
        Pane::Plugins,
    ];
}

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub top_height_pct: u16,
    pub top_cols_pct: [u16; 3],
    pub bottom_cols_pct: [u16; 3],
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            top_height_pct: 56,
            top_cols_pct: [33, 33, 34],
            bottom_cols_pct: [34, 33, 33],
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ListCursorState {
    pub system: usize,
    pub prs: usize,
    pub docker: usize,
    pub aws: usize,
    pub plugins: usize,
}

#[derive(Debug, Clone)]
pub struct DetailModal {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum NavDir {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub enum PaletteCommand {
    Refresh,
    ReloadConfig,
    ToggleCompact,
    Focus(Pane),
    Quit,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemLayoutMode {
    Auto,
    Compact,
    Cockpit,
}

#[derive(Debug, Clone, Copy)]
pub struct SystemAlerts {
    pub cpu_warn_pct: f32,
    pub cpu_crit_pct: f32,
    pub mem_warn_pct: f32,
    pub mem_crit_pct: f32,
    pub stale_warn_secs: f32,
    pub stale_crit_secs: f32,
}

impl Default for SystemAlerts {
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

pub fn parse_pane(s: &str) -> Option<Pane> {
    match s {
        "1" | "git" => Some(Pane::Git),
        "2" | "system" => Some(Pane::System),
        "3" | "prs" | "pr" => Some(Pane::Prs),
        "4" | "docker" => Some(Pane::Docker),
        "5" | "aws" => Some(Pane::Aws),
        "6" | "plugins" | "plugin" => Some(Pane::Plugins),
        _ => None,
    }
}
