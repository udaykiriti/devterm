use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::app::DetailModal;

use super::theme::{ACCENT_BRIGHT, BORDER_FOCUSED, GLOW, PANEL_BG, TEXT, TEXT_DIM};

pub fn render_modal(frame: &mut Frame, detail: &DetailModal) {
    let popup = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, popup);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for line in detail.lines.iter().take(25) {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(line.clone(), Style::default().fg(TEXT)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            "[Esc] ",
            Style::default().fg(GLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Close  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "[Enter] ",
            Style::default()
                .fg(ACCENT_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Confirm", Style::default().fg(TEXT_DIM)),
    ]));

    let modal = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" [[ {} ]] ", detail.title),
                    Style::default()
                        .fg(ACCENT_BRIGHT)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::default()
                        .fg(BORDER_FOCUSED)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(PANEL_BG)),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(TEXT));

    frame.render_widget(modal, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
