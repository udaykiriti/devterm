use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::{LayoutState, Pane};

#[derive(Debug, Clone, Copy)]
pub struct PaneLayout {
    pub header: Rect,
    pub git: Rect,
    pub system: Rect,
    pub prs: Rect,
    pub docker: Rect,
    pub aws: Rect,
    pub plugins: Rect,
    pub footer: Rect,
}

pub fn compute_layout(area: Rect, layout: &LayoutState) -> PaneLayout {
    let header_h = if area.height < 22 { 2 } else { 3 };
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_h),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(area);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(layout.top_height_pct),
            Constraint::Percentage(100 - layout.top_height_pct),
        ])
        .split(vertical[1]);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(layout.top_cols_pct[0]),
            Constraint::Percentage(layout.top_cols_pct[1]),
            Constraint::Percentage(layout.top_cols_pct[2]),
        ])
        .split(body[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(layout.bottom_cols_pct[0]),
            Constraint::Percentage(layout.bottom_cols_pct[1]),
            Constraint::Percentage(layout.bottom_cols_pct[2]),
        ])
        .split(body[1]);

    PaneLayout {
        header: vertical[0],
        git: top[0],
        system: top[1],
        prs: top[2],
        docker: bottom[0],
        aws: bottom[1],
        plugins: bottom[2],
        footer: vertical[2],
    }
}

pub fn pane_at(area: Rect, layout: &LayoutState, x: u16, y: u16) -> Option<Pane> {
    let map = compute_layout(area, layout);
    let point_in = |r: Rect| -> bool {
        x >= r.x && x < r.x.saturating_add(r.width) && y >= r.y && y < r.y.saturating_add(r.height)
    };

    if point_in(map.git) {
        return Some(Pane::Git);
    }
    if point_in(map.system) {
        return Some(Pane::System);
    }
    if point_in(map.prs) {
        return Some(Pane::Prs);
    }
    if point_in(map.docker) {
        return Some(Pane::Docker);
    }
    if point_in(map.aws) {
        return Some(Pane::Aws);
    }
    if point_in(map.plugins) {
        return Some(Pane::Plugins);
    }

    None
}
