mod chrome;
mod layout;
mod modal;
mod panes;
mod theme;

use ratatui::{Frame, layout::Margin, widgets::Block};

use crate::app::App;

use self::chrome::{render_footer, render_header};
use self::layout::compute_layout;
use self::modal::render_modal;
use self::panes::{
    render_aws, render_docker, render_git, render_plugins, render_prs, render_system,
};
use self::theme::BG;

pub use self::layout::pane_at;

pub fn render(frame: &mut Frame, app: &App) {
    let root = Block::default().style(ratatui::style::Style::default().bg(BG));
    frame.render_widget(root, frame.area());

    let shell = frame.area().inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    let layout = compute_layout(shell, &app.layout);

    render_header(frame, app, layout.header);
    render_git(frame, app, layout.git);
    render_system(frame, app, layout.system);
    render_prs(frame, app, layout.prs);
    render_docker(frame, app, layout.docker);
    render_aws(frame, app, layout.aws);
    render_plugins(frame, app, layout.plugins);
    render_footer(frame, app, layout.footer);

    if let Some(detail) = &app.detail_modal {
        render_modal(frame, detail);
    }
}
