use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, List, ListItem, ListState, Paragraph, Sparkline, Wrap},
};

use crate::app::{App, Pane, SystemLayoutMode};

use super::chrome::pane_block;
use super::theme::{
    ACCENT, ACCENT_BRIGHT, BAD, BAD_BRIGHT, GLOW, GOOD, GOOD_BRIGHT, HIGHLIGHT_BG, MUTED,
    PANEL_BG_ACTIVE, SECONDARY, TERTIARY, TEXT, TEXT_DIM, WARN, WARN_BRIGHT,
};

pub fn render_git(frame: &mut Frame, app: &App, area: Rect) {
    let git = &app.data.git;

    // Modern git status with ASCII art
    let lines = vec![
        Line::from(vec![
            Span::styled(
                "[G] ",
                Style::default()
                    .fg(ACCENT_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Branch: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                &git.branch,
                Style::default().fg(SECONDARY).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[~] ", Style::default().fg(TERTIARY)),
            Span::styled("Tracking: ", Style::default().fg(TEXT_DIM)),
            Span::styled(blank_to_na(&git.ahead_behind), Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "[+] ",
                Style::default()
                    .fg(GOOD_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Staged ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                format!("{:>3}", git.staged),
                Style::default().fg(GOOD).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    ", Style::default()),
            Span::styled(
                "[*] ",
                Style::default()
                    .fg(WARN_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Unstaged ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                format!("{:>3}", git.unstaged),
                Style::default().fg(WARN).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "[?] ",
                Style::default().fg(BAD_BRIGHT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Untracked ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                format!("{:>3}", git.untracked),
                Style::default().fg(BAD).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let p = Paragraph::new(lines)
        .block(pane_block("GIT", app.selected == Pane::Git))
        .style(Style::default().fg(TEXT))
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

pub fn render_system(frame: &mut Frame, app: &App, area: Rect) {
    let mode = match app.system_layout_mode {
        SystemLayoutMode::Compact => SystemLayoutMode::Compact,
        SystemLayoutMode::Cockpit => SystemLayoutMode::Cockpit,
        SystemLayoutMode::Auto => {
            if area.width < 86 || area.height < 13 {
                SystemLayoutMode::Compact
            } else {
                SystemLayoutMode::Cockpit
            }
        }
    };

    let sys = &app.data.system;
    let mem_pct = if sys.mem_total_gb > 0.0 {
        (sys.mem_used_gb / sys.mem_total_gb).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let flash = app.system_flash_ticks > 0;
    let pulse = app.spinner_index % 8;
    let pulse_char = ["◜", "◠", "◝", "◞", "◡", "◟", "◜", "◠"][pulse];
    let cpu_pct = sys.cpu_usage.clamp(0.0, 100.0);
    let mem_pct_num = (mem_pct * 100.0) as f32;
    let cpu_color = pulse_color(
        utilization_color(
            cpu_pct,
            app.system_alerts.cpu_warn_pct,
            app.system_alerts.cpu_crit_pct,
        ),
        flash,
        pulse,
    );
    let mem_color = pulse_color(
        utilization_color(
            mem_pct_num,
            app.system_alerts.mem_warn_pct,
            app.system_alerts.mem_crit_pct,
        ),
        flash,
        (pulse + 3) % 8,
    );
    let load_color = SECONDARY;
    let fresh_color = TERTIARY;

    let freshness_secs = app
        .last_update()
        .map(|ts| (Utc::now() - ts).num_milliseconds().max(0) as f64 / 1000.0)
        .unwrap_or(999.0);
    let freshness_color = if freshness_secs < app.system_alerts.stale_warn_secs as f64 {
        GOOD
    } else if freshness_secs < app.system_alerts.stale_crit_secs as f64 {
        WARN
    } else {
        BAD
    };

    let cpu_avg = rolling_avg(&app.cpu_history, 12);
    let mem_avg = rolling_avg(&app.mem_history, 12);
    let _disk_avg = rolling_avg(&app.disk_history, 12);
    let cpu_strip = trend_strip(&app.cpu_history, 28, pulse);
    let mem_strip = trend_strip(&app.mem_history, 28, (pulse + 4) % 8);
    let disk_strip = trend_strip(&app.disk_history, 19, (pulse + 2) % 8);
    let health = health_score(cpu_pct, mem_pct_num, sys.load_avg.0 as f32);
    let (health_label, health_color) = health_badge(health);
    let _peak_active = app.peak_hold_ticks > 0;

    let panel = pane_block(
        if flash {
            "SYSTEM // LIVE"
        } else {
            "SYSTEM // METRICS"
        },
        app.selected == Pane::System,
    );
    let inner = panel.inner(area);
    frame.render_widget(panel, area);

    if mode == SystemLayoutMode::Compact {
        render_system_compact(
            frame,
            inner,
            app,
            cpu_pct,
            mem_pct,
            mem_pct_num,
            freshness_secs,
            freshness_color,
            cpu_color,
            mem_color,
            load_color,
            health_label,
            health_color,
            pulse_char,
        );
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top line
            Constraint::Length(7), // gauges + stats cards (increased for 3 gauges)
            Constraint::Length(1), // cpu trend
            Constraint::Length(1), // mem trend
            Constraint::Length(1), // disk trend (NEW)
            Constraint::Length(1), // core meters
            Constraint::Length(1), // trend strips
            Constraint::Length(1), // process line
        ])
        .split(inner);

    let core = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[1]);

    let gauges = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // CPU gauge
            Constraint::Length(2), // MEM gauge
            Constraint::Length(2), // DISK gauge (NEW)
            Constraint::Length(1), // effects
        ])
        .split(core[0]);

    let cards = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(core[1]);

    let top_line = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{pulse_char}{pulse_char} "),
            Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("CPU ", Style::default().fg(MUTED)),
        Span::styled(format!("{:>5.1}%", cpu_pct), Style::default().fg(cpu_color)),
        Span::styled(" ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{:>6}", fmt_delta(app.cpu_delta, "%")),
            Style::default().fg(color_for_delta(app.cpu_delta)),
        ),
        Span::styled("  MEM ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{:>5.1}%", mem_pct_num),
            Style::default().fg(mem_color),
        ),
        Span::styled(" ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{:>6}", fmt_delta(app.mem_delta as f32, "%")),
            Style::default().fg(color_for_delta(app.mem_delta as f32)),
        ),
        Span::styled("  [", Style::default().fg(MUTED)),
        Span::styled(
            health_label.to_string(),
            Style::default()
                .fg(health_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("]", Style::default().fg(MUTED)),
        Span::styled("  Refresh: ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{:.1}s", freshness_secs),
            Style::default().fg(freshness_color),
        ),
    ]));
    frame.render_widget(top_line, rows[0]);

    let cpu_gauge = Gauge::default()
        .gauge_style(Style::default().fg(cpu_color).bg(PANEL_BG_ACTIVE))
        .label(format!("CPU UTIL  {:>5.1}%", cpu_pct))
        .ratio((cpu_pct as f64 / 100.0).clamp(0.0, 1.0));
    frame.render_widget(cpu_gauge, gauges[0]);

    let mem_gauge = Gauge::default()
        .gauge_style(Style::default().fg(mem_color).bg(PANEL_BG_ACTIVE))
        .label(format!(
            "MEM USE   {:>4.1}/{:>4.1} GB",
            sys.mem_used_gb, sys.mem_total_gb
        ))
        .ratio(mem_pct);
    frame.render_widget(mem_gauge, gauges[1]);

    let disk_pct = if sys.disk_total_gb > 0.0 {
        (sys.disk_used_gb / sys.disk_total_gb).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let disk_color = if disk_pct > 0.9 {
        BAD_BRIGHT
    } else if disk_pct > 0.75 {
        WARN_BRIGHT
    } else {
        GOOD_BRIGHT
    };
    let disk_gauge = Gauge::default()
        .gauge_style(Style::default().fg(disk_color).bg(PANEL_BG_ACTIVE))
        .label(format!(
            "DISK USE  {:>4.1}/{:>4.1} GB",
            sys.disk_used_gb, sys.disk_total_gb
        ))
        .ratio(disk_pct);
    frame.render_widget(disk_gauge, gauges[2]);

    let mini_fx = Paragraph::new(Line::from(vec![
        Span::styled("wave ", Style::default().fg(fresh_color)),
        Span::styled(
            aurora_wave(12, app.spinner_index),
            Style::default().fg(fresh_color),
        ),
        Span::styled("  orbit ", Style::default().fg(health_color)),
        Span::styled(
            orbit_dots(8, app.spinner_index),
            Style::default().fg(health_color),
        ),
    ]));
    frame.render_widget(mini_fx, gauges[3]);

    let card_lines = [
        Line::from(vec![
            Span::styled("LOAD ", Style::default().fg(MUTED)),
            Span::styled(
                format!(
                    "{:>4.2} {:>4.2} {:>4.2}",
                    sys.load_avg.0, sys.load_avg.1, sys.load_avg.2
                ),
                Style::default().fg(load_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("CPU ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>5.1}%", cpu_avg),
                Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  pk ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>5.1}%", app.cpu_peak),
                Style::default().fg(cpu_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("MEM ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>5.1}%", mem_avg),
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" avl ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>4.1}G", sys.mem_available_gb),
                Style::default().fg(ACCENT_BRIGHT),
            ),
        ]),
        Line::from(vec![
            Span::styled("DISK ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>5.1}%", disk_pct * 100.0),
                Style::default().fg(disk_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  pk ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>5.1}%", app.disk_peak),
                Style::default().fg(disk_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("CORES ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>2}", sys.cpu_cores.len()),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  PROCS ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:>4}", sys.process_count),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("NET RX ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:.1}M", sys.network_rx_mb),
                Style::default()
                    .fg(ACCENT_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  TX ", Style::default().fg(MUTED)),
            Span::styled(
                format!("{:.1}M", sys.network_tx_mb),
                Style::default()
                    .fg(ACCENT_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("UP ", Style::default().fg(MUTED)),
            Span::styled(
                format_duration_short(sys.uptime_secs),
                Style::default().fg(GLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [", Style::default().fg(MUTED)),
            Span::styled(
                health_label.to_string(),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("]", Style::default().fg(MUTED)),
        ]),
    ];
    for (idx, line) in card_lines.into_iter().enumerate() {
        frame.render_widget(Paragraph::new(line), cards[idx]);
    }

    let cpu_trend = Sparkline::default()
        .data(&app.cpu_history)
        .max(100)
        .style(Style::default().fg(cpu_color));
    frame.render_widget(cpu_trend, rows[2]);

    let mem_trend = Sparkline::default()
        .data(&app.mem_history)
        .max(100)
        .style(Style::default().fg(mem_color));
    frame.render_widget(mem_trend, rows[3]);

    let disk_trend = Sparkline::default()
        .data(&app.disk_history)
        .max(100)
        .style(Style::default().fg(disk_color));
    frame.render_widget(disk_trend, rows[4]);

    let core_line = Paragraph::new(Line::from(vec![
        Span::styled("cores ", Style::default().fg(MUTED)),
        Span::styled(
            core_meter(&sys.cpu_cores, 22),
            Style::default().fg(cpu_color),
        ),
        Span::styled("  swap ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{:.1}/{:.1} GB", sys.swap_used_gb, sys.swap_total_gb),
            Style::default().fg(load_color),
        ),
    ]));
    frame.render_widget(core_line, rows[5]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("C ", Style::default().fg(cpu_color)),
        Span::styled(cpu_strip, Style::default().fg(cpu_color)),
        Span::styled("  M ", Style::default().fg(mem_color)),
        Span::styled(mem_strip, Style::default().fg(mem_color)),
        Span::styled("  D ", Style::default().fg(disk_color)),
        Span::styled(disk_strip, Style::default().fg(disk_color)),
    ]));
    frame.render_widget(footer, rows[6]);

    let proc_line = Paragraph::new(render_top_process_line(
        &sys.top_processes,
        area.width.saturating_sub(4) as usize,
        app.current_list_cursor(Pane::System),
        app.selected == Pane::System,
    ));
    frame.render_widget(proc_line, rows[7]);
}

#[allow(clippy::too_many_arguments)]
fn render_system_compact(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    cpu_pct: f32,
    mem_pct: f64,
    mem_pct_num: f32,
    freshness_secs: f64,
    freshness_color: Color,
    cpu_color: Color,
    mem_color: Color,
    load_color: Color,
    health_label: &str,
    health_color: Color,
    pulse_char: &str,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let top = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{pulse_char} "),
            Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("[{health_label}]"),
            Style::default().fg(health_color),
        ),
        Span::styled("  fr ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{:>4.1}s", freshness_secs),
            Style::default().fg(freshness_color),
        ),
        Span::styled("  L ", Style::default().fg(MUTED)),
        Span::styled(
            format!(
                "{:.2}/{:.2}",
                app.data.system.load_avg.0, app.data.system.load_avg.1
            ),
            Style::default().fg(load_color),
        ),
    ]));
    frame.render_widget(top, rows[0]);

    let cpu = Gauge::default()
        .gauge_style(Style::default().fg(cpu_color).bg(PANEL_BG_ACTIVE))
        .label(format!(
            "CPU {:>5.1}% {:>6}",
            cpu_pct,
            fmt_delta(app.cpu_delta, "%")
        ))
        .ratio((cpu_pct as f64 / 100.0).clamp(0.0, 1.0));
    frame.render_widget(cpu, rows[1]);

    let mem = Gauge::default()
        .gauge_style(Style::default().fg(mem_color).bg(PANEL_BG_ACTIVE))
        .label(format!(
            "MEM {:>5.1}% {:>6}",
            mem_pct_num,
            fmt_delta(app.mem_delta as f32, "%")
        ))
        .ratio(mem_pct);
    frame.render_widget(mem, rows[2]);

    let details = Paragraph::new(Line::from(vec![
        Span::styled("up ", Style::default().fg(MUTED)),
        Span::styled(
            format_duration_short(app.data.system.uptime_secs),
            Style::default().fg(TEXT),
        ),
        Span::styled("  p ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{}", app.data.system.process_count),
            Style::default().fg(TEXT),
        ),
        Span::styled("  sw ", Style::default().fg(MUTED)),
        Span::styled(
            format!(
                "{:.1}/{:.1}G",
                app.data.system.swap_used_gb, app.data.system.swap_total_gb
            ),
            Style::default().fg(load_color),
        ),
    ]));
    frame.render_widget(details, rows[3]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("C ", Style::default().fg(cpu_color)),
        Span::styled(
            trend_strip(&app.cpu_history, 14, app.spinner_index),
            Style::default().fg(cpu_color),
        ),
        Span::styled("  M ", Style::default().fg(mem_color)),
        Span::styled(
            trend_strip(&app.mem_history, 14, app.spinner_index + 3),
            Style::default().fg(mem_color),
        ),
    ]));
    frame.render_widget(footer, rows[4]);
}

pub fn render_docker(frame: &mut Frame, app: &App, area: Rect) {
    render_status_list(
        frame,
        area,
        pane_block("DOCKER", app.selected == Pane::Docker),
        app.data.docker.running.iter().map(|s| s.as_str()).collect(),
        app.data.docker.error.as_deref(),
        None,
        app.current_list_cursor(Pane::Docker),
    );
}

pub fn render_aws(frame: &mut Frame, app: &App, area: Rect) {
    render_status_list(
        frame,
        area,
        pane_block("AWS EC2", app.selected == Pane::Aws),
        app.data.aws.instances.iter().map(|s| s.as_str()).collect(),
        app.data.aws.error.as_deref(),
        Some(app.data.aws.source.as_str()),
        app.current_list_cursor(Pane::Aws),
    );
}

pub fn render_prs(frame: &mut Frame, app: &App, area: Rect) {
    render_status_list(
        frame,
        area,
        pane_block("OPEN PRS", app.selected == Pane::Prs),
        app.data.prs.open.iter().map(|s| s.as_str()).collect(),
        app.data.prs.error.as_deref(),
        Some(app.data.prs.source.as_str()),
        app.current_list_cursor(Pane::Prs),
    );
}

pub fn render_plugins(frame: &mut Frame, app: &App, area: Rect) {
    let block = pane_block("PLUGINS", app.selected == Pane::Plugins);

    if app.data.plugins.is_empty() {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  [X]  ",
                    Style::default()
                        .fg(ACCENT_BRIGHT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("No plugins configured", Style::default().fg(TEXT_DIM)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("-> ", Style::default().fg(MUTED)),
                Span::styled("Add [[plugins]] entries in ", Style::default().fg(TEXT_DIM)),
                Span::styled("devdash.toml", Style::default().fg(SECONDARY)),
            ]),
        ])
        .block(block)
        .style(Style::default().fg(TEXT));
        frame.render_widget(p, area);
        return;
    }

    let items = app
        .data
        .plugins
        .iter()
        .map(|plugin| {
            let (icon, color) = if plugin.error.is_some() {
                ("[X]", BAD_BRIGHT)
            } else {
                ("[+]", GOOD_BRIGHT)
            };
            let sample = plugin.lines.first().cloned().unwrap_or_default();
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{icon} "), Style::default().fg(color)),
                    Span::styled(
                        &plugin.name,
                        Style::default()
                            .fg(ACCENT_BRIGHT)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(sample, Style::default().fg(TEXT_DIM)),
                ]),
            ])
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(app.current_list_cursor(Pane::Plugins));

    let list = List::new(items)
        .block(block)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(TEXT).bg(HIGHLIGHT_BG));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_status_list(
    frame: &mut Frame,
    area: Rect,
    block: ratatui::widgets::Block<'_>,
    items: Vec<&str>,
    err: Option<&str>,
    source: Option<&str>,
    selected: Option<usize>,
) {
    let mut list_items = Vec::new();
    if let Some(s) = source {
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("source: ", Style::default().fg(MUTED)),
            Span::styled(s.to_string(), Style::default().fg(TEXT)),
        ])));
    }

    if let Some(msg) = err {
        list_items.push(ListItem::new(Line::from(vec![Span::styled(
            format!("error: {msg}"),
            Style::default().fg(BAD),
        )])));
    }

    let offset = list_items.len();
    list_items.extend(items.into_iter().map(|line| {
        ListItem::new(Line::from(vec![
            Span::styled("- ", Style::default().fg(ACCENT)),
            Span::styled(line.to_string(), Style::default().fg(TEXT)),
        ]))
    }));

    let mut state = ListState::default();
    state.select(selected.map(|v| v + offset));

    let list = List::new(list_items)
        .block(block)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(TEXT).bg(HIGHLIGHT_BG));
    frame.render_stateful_widget(list, area, &mut state);
}

fn blank_to_na(s: &str) -> &str {
    if s.trim().is_empty() { "n/a" } else { s }
}

fn fmt_delta(v: f32, unit: &str) -> String {
    if v >= 0.0 {
        format!("+{v:.1}{unit}")
    } else {
        format!("{v:.1}{unit}")
    }
}

fn color_for_delta(v: f32) -> Color {
    if v.abs() < 0.2 {
        MUTED
    } else if v > 0.0 {
        WARN
    } else {
        GOOD
    }
}

fn utilization_color(v: f32, warn: f32, crit: f32) -> Color {
    if v < warn {
        GOOD_BRIGHT
    } else if v < crit {
        WARN_BRIGHT
    } else {
        BAD_BRIGHT
    }
}

fn pulse_color(base: Color, flash: bool, phase: usize) -> Color {
    if !flash {
        return base;
    }
    if phase.is_multiple_of(2) {
        match base {
            Color::Rgb(r, g, b) => Color::Rgb(
                r.saturating_add(25),
                g.saturating_add(25),
                b.saturating_add(15),
            ),
            _ => ACCENT_BRIGHT,
        }
    } else {
        base
    }
}

fn rolling_avg(values: &[u64], tail: usize) -> f32 {
    let n = values.len().min(tail);
    if n == 0 {
        return 0.0;
    }
    let sum: u64 = values[values.len() - n..].iter().sum();
    sum as f32 / n as f32
}

fn trend_strip(values: &[u64], width: usize, phase: usize) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if width == 0 {
        return String::new();
    }

    let slice = if values.len() > width {
        &values[values.len() - width..]
    } else {
        values
    };

    let mut chars = slice
        .iter()
        .map(|v| {
            let idx = ((*v as f32 / 100.0) * 7.0).round().clamp(0.0, 7.0) as usize;
            BARS[idx]
        })
        .collect::<Vec<_>>();

    if chars.len() < width {
        let pad = vec![' '; width - chars.len()];
        let mut joined = pad;
        joined.extend(chars);
        chars = joined;
    }

    if !chars.is_empty() {
        let marker_idx = phase % chars.len();
        chars[marker_idx] = '•';
    }

    chars.into_iter().collect()
}

#[allow(dead_code)]
fn neon_meter(value_pct: f32, width: usize, phase: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let fill = ((value_pct.clamp(0.0, 100.0) / 100.0) * width as f32).round() as usize;
    let head = if fill == 0 {
        0
    } else {
        (phase % fill.max(1)).min(width - 1)
    };

    let mut out = String::with_capacity(width);
    for i in 0..width {
        let ch = if i < fill {
            if i == head { '◆' } else { '━' }
        } else {
            '·'
        };
        out.push(ch);
    }
    out
}

fn aurora_wave(width: usize, phase: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let glyphs = [' ', '.', '·', '˙', '°', '•', '◦', '◌'];
    let mut out = String::with_capacity(width);
    let p = phase as f32 * 0.45;
    for x in 0..width {
        let t = x as f32 * 0.55 + p;
        let y = ((t.sin() + (t * 0.5).cos()) * 0.5 + 0.5).clamp(0.0, 1.0);
        let idx = (y * (glyphs.len() as f32 - 1.0)).round() as usize;
        let idx = idx.min(glyphs.len() - 1);
        out.push(glyphs[idx]);
    }
    out
}

fn health_score(cpu: f32, mem: f32, load1: f32) -> u8 {
    let c = (cpu.clamp(0.0, 100.0) * 0.45) as u8;
    let m = (mem.clamp(0.0, 100.0) * 0.35) as u8;
    let l = ((load1.clamp(0.0, 8.0) / 8.0) * 20.0) as u8;
    c.saturating_add(m).saturating_add(l).min(100)
}

fn health_badge(score: u8) -> (&'static str, Color) {
    if score < 38 {
        ("CALM", GOOD)
    } else if score < 68 {
        ("BUSY", WARN)
    } else {
        ("HOT", BAD)
    }
}

fn orbit_dots(width: usize, phase: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = vec!['.'; width];
    let p1 = phase % width;
    let p2 = (phase + (width / 2).max(1)) % width;
    out[p1] = 'O';
    out[p2] = 'o';
    out.into_iter().collect()
}

fn core_meter(cores: &[f32], width: usize) -> String {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if width == 0 {
        return String::new();
    }
    if cores.is_empty() {
        return "n/a".to_string();
    }

    let mut out = String::new();
    let step = (cores.len() as f32 / width as f32).max(1.0);
    let mut idx = 0.0;
    while out.len() < width {
        let cidx = idx as usize;
        let v = *cores.get(cidx).unwrap_or(&0.0);
        let b = ((v.clamp(0.0, 100.0) / 100.0) * 7.0).round() as usize;
        out.push(BARS[b.min(7)]);
        idx += step;
        if cidx + 1 >= cores.len() && out.len() < width {
            out.push(' ');
        }
    }
    out.chars().take(width).collect()
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

fn render_top_process_line(
    processes: &[crate::collectors::ProcessStat],
    width: usize,
    selected: Option<usize>,
    focused: bool,
) -> Line<'static> {
    let mut spans = vec![Span::styled("top ", Style::default().fg(MUTED))];

    if processes.is_empty() {
        spans.push(Span::styled("n/a", Style::default().fg(TEXT)));
        return Line::from(spans);
    }

    for (idx, p) in processes.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled("  ", Style::default().fg(MUTED)));
        }
        let is_sel = focused && selected == Some(idx);
        spans.push(Span::styled(
            if is_sel {
                format!("▶{}.", idx + 1)
            } else {
                format!("{}.", idx + 1)
            },
            Style::default().fg(if is_sel { ACCENT } else { WARN }),
        ));
        spans.push(Span::styled(
            format!(
                "{} {:>4.1}% {:>4.0}M",
                truncate_name(&p.name, width / 5),
                p.cpu_pct,
                p.mem_mb
            ),
            Style::default().fg(if is_sel { ACCENT } else { TEXT }),
        ));
    }

    Line::from(spans)
}

fn truncate_name(name: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    for ch in name.chars().take(max) {
        out.push(ch);
    }
    if name.chars().count() > max && max > 2 {
        out.pop();
        out.push('…');
    }
    out
}
