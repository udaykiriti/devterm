use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::{App, Pane};

use super::theme::{
    ACCENT, ACCENT_BRIGHT, BAD, BAD_BRIGHT, BG, BORDER, BORDER_ACTIVE, BORDER_FOCUSED, 
    GOOD, GOOD_BRIGHT, GLOW, MUTED, PANEL_BG, PANEL_BG_ACTIVE, SECONDARY, TERTIARY, 
    TEXT, TEXT_DIM, WARN_BRIGHT
};

pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let selected = pane_name(app.selected);
    let updated = app
        .last_update()
        .map(|ts| ts.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "──:──:──".to_string());

    let status_color = if app.loading { WARN_BRIGHT } else { GOOD_BRIGHT };
    let status_text = if app.loading { "[*]" } else { "[+]" };
    let mode = if app.compact_mode { "compact" } else { "normal" };

    let spinner = spinner_glyph(app.spinner_index);
    
    // Modern header with gradient-style separators
    let mut lines = vec![Line::from(vec![
        Span::styled(
            " [#] ",
            Style::default().fg(ACCENT_BRIGHT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "DEVDASH",
            Style::default().fg(ACCENT_BRIGHT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " [#]",
            Style::default().fg(ACCENT_BRIGHT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            status_text,
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {spinner} "),
            Style::default().fg(status_color),
        ),
        Span::styled(
            if app.loading { "Refreshing..." } else { "Ready" },
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ", Style::default()),
        Span::styled("/ ", Style::default().fg(BORDER)),
        Span::styled("Focus: ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            selected,
            Style::default().fg(SECONDARY).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  / ", Style::default().fg(BORDER)),
        Span::styled("Mode: ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            mode,
            Style::default().fg(TERTIARY).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  / ", Style::default().fg(BORDER)),
        Span::styled("[T] ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            updated,
            Style::default().fg(GLOW).add_modifier(Modifier::BOLD),
        ),
    ])];

    if area.height >= 3 {
        lines.push(Line::from(build_modern_pane_chips(app.selected)));
    }

    let p = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(BG))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER_ACTIVE)),
        );
    frame.render_widget(p, area);
}

pub fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    if app.command_mode {
        let cmd = Paragraph::new(Line::from(vec![
            Span::styled(
                "> ",
                Style::default().fg(ACCENT_BRIGHT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(app.command_input.clone(), Style::default().fg(TEXT)),
            Span::styled("|", Style::default().fg(ACCENT_BRIGHT)),
        ]))
        .alignment(Alignment::Left)
        .style(Style::default().bg(BG))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER_FOCUSED)),
        );
        frame.render_widget(cmd, area);
        return;
    }

    if app.compact_mode {
        let mut spans = vec![
            Span::styled("> ", Style::default().fg(ACCENT)),
            Span::styled("Commands  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[Enter] ", Style::default().fg(ACCENT_BRIGHT)),
            Span::styled("Details  ", Style::default().fg(TEXT_DIM)),
            Span::styled("F10 ", Style::default().fg(BAD)),
            Span::styled("Quit", Style::default().fg(TEXT_DIM)),
        ];
        if let Some(msg) = &app.status {
            spans.push(Span::styled("  /  ", Style::default().fg(BORDER)));
            spans.push(Span::styled(msg, Style::default().fg(GLOW)));
        }
        let p = Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .style(Style::default().bg(BG))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(BORDER)),
            );
        frame.render_widget(p, area);
        return;
    }

    let mut spans = vec![
        Span::styled("  ", Style::default()),
        Span::styled(">", Style::default().fg(ACCENT_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(" Palette  ", Style::default().fg(TEXT_DIM)),
        Span::styled("[Tab] ", Style::default().fg(SECONDARY)),
        Span::styled("Cycle  ", Style::default().fg(TEXT_DIM)),
        Span::styled("[Arrows] ", Style::default().fg(TERTIARY)),
        Span::styled("Navigate  ", Style::default().fg(TEXT_DIM)),
        Span::styled("[Enter] ", Style::default().fg(ACCENT_BRIGHT)),
        Span::styled("Details  ", Style::default().fg(TEXT_DIM)),
        Span::styled("[+/-]", Style::default().fg(GOOD)),
        Span::styled(" Resize  ", Style::default().fg(TEXT_DIM)),
        Span::styled("F10 ", Style::default().fg(BAD)),
        Span::styled("Exit", Style::default().fg(TEXT_DIM)),
    ];

    if let Some(err) = &app.last_error {
        spans.push(Span::styled("    //  ", Style::default().fg(BORDER)));
        spans.push(Span::styled("[!] ", Style::default().fg(BAD_BRIGHT).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(err, Style::default().fg(WARN_BRIGHT)));
    } else if let Some(msg) = &app.status {
        spans.push(Span::styled("    //  ", Style::default().fg(BORDER)));
        spans.push(Span::styled("[+] ", Style::default().fg(GOOD_BRIGHT)));
        spans.push(Span::styled(msg, Style::default().fg(GLOW)));
    }

    let p = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .style(Style::default().bg(BG))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(BORDER)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

pub fn pane_block(title: &str, selected: bool) -> Block<'_> {
    let (border_style, bg, btype) = if selected {
        (
            Style::default().fg(BORDER_FOCUSED).add_modifier(Modifier::BOLD),
            PANEL_BG_ACTIVE,
            BorderType::Rounded,
        )
    } else {
        (
            Style::default().fg(BORDER),
            PANEL_BG,
            BorderType::Rounded,
        )
    };

    let title_style = if selected {
        Style::default()
            .fg(ACCENT_BRIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD)
    };

    Block::default()
        .title(Span::styled(
            format!(" [[ {} ]] ", title),
            title_style,
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(btype)
        .border_style(border_style)
        .style(Style::default().bg(bg))
}

fn build_modern_pane_chips(selected: Pane) -> Vec<Span<'static>> {
    let mut spans = vec![
        Span::styled(" ", Style::default()),
        Span::styled("[", Style::default().fg(ACCENT_BRIGHT)),
        Span::styled(" Panes ", Style::default().fg(TEXT_DIM)),
        Span::styled("] ", Style::default().fg(ACCENT_BRIGHT)),
    ];
    
    for pane in Pane::ALL.iter() {
        let name = pane_name(*pane);
        let icon = pane_icon(*pane);
        
        if *pane == selected {
            spans.push(Span::styled(" / ", Style::default().fg(BORDER)));
            spans.push(Span::styled(
                icon,
                Style::default().fg(ACCENT_BRIGHT).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {name}"),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(" / ", Style::default().fg(BORDER)));
            spans.push(Span::styled(icon, Style::default().fg(MUTED)));
            spans.push(Span::styled(format!(" {name}"), Style::default().fg(TEXT_DIM)));
        }
    }
    spans.push(Span::styled(" ", Style::default()));
    spans
}

fn pane_name(p: Pane) -> &'static str {
    match p {
        Pane::Git => "Git",
        Pane::System => "System",
        Pane::Prs => "Open PRs",
        Pane::Docker => "Docker",
        Pane::Aws => "AWS EC2",
        Pane::Plugins => "Plugins",
    }
}

fn pane_icon(p: Pane) -> &'static str {
    match p {
        Pane::Git => "[G]",
        Pane::System => "[S]",
        Pane::Prs => "[P]",
        Pane::Docker => "[D]",
        Pane::Aws => "[A]",
        Pane::Plugins => "[X]",
    }
}

fn spinner_glyph(idx: usize) -> &'static str {
    const GLYPHS: [&str; 10] = ["[|]", "[/]", "[-]", "[\\]", "[|]", "[/]", "[-]", "[\\]", "[|]", "[/]"];
    GLYPHS[idx % GLYPHS.len()]
}
