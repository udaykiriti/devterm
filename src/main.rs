mod app;
mod collectors;
mod config;
mod plugin;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
        KeyModifiers, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures_util::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use std::io;
use tokio::sync::mpsc;

use app::{App, NavDir, PaletteCommand, Pane};
use collectors::{DataCache, apply_cache, collect_all};
use config::Config;
use plugin::PluginManager;

/// Messages for controlling data collection
enum ControlMsg {
    RefreshNow,
    ReloadRuntime { cfg: Config, plugins: PluginManager },
}

/// Main application entry point
#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::load()?;
    let plugins = PluginManager::from_config(&cfg.plugins);

    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to initialize terminal")?;

    let run_result = run_app(&mut terminal, cfg, plugins).await;

    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    run_result
}

/// Main application loop handling UI rendering and event processing
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cfg: Config,
    plugins: PluginManager,
) -> Result<()> {
    let mut app = App::new();
    app.apply_config(&cfg);

    // Channels for data updates, loading status, and control messages
    let (data_tx, mut data_rx) = mpsc::channel(8);
    let (status_tx, mut status_rx) = mpsc::channel(8);
    let (ctrl_tx, mut ctrl_rx) = mpsc::channel(8);

    // Background task for periodic data collection
    let mut collector_cfg = cfg.clone();
    let mut plugin_mgr = plugins.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            collector_cfg.refresh_seconds.max(1),
        ));
        let mut cache = DataCache::default();

        loop {
            tokio::select! {
                _ = interval.tick() => {}
                cmd = ctrl_rx.recv() => {
                    match cmd {
                        Some(ControlMsg::RefreshNow) => {}
                        Some(ControlMsg::ReloadRuntime { cfg, plugins }) => {
                            collector_cfg = cfg;
                            plugin_mgr = plugins;
                            interval = tokio::time::interval(std::time::Duration::from_secs(
                                collector_cfg.refresh_seconds.max(1),
                            ));
                        }
                        None => break,
                    }
                }
            }

            let _ = status_tx.send(true).await;
            let mut data = collect_all(&collector_cfg, &plugin_mgr).await;
            apply_cache(&mut data, &mut cache, collector_cfg.cache_seconds.max(1));
            let _ = status_tx.send(false).await;

            if data_tx.send(data).await.is_err() {
                break;
            }
        }
    });

    if ctrl_tx.send(ControlMsg::RefreshNow).await.is_err() {
        eprintln!("Warning: Failed to send initial refresh signal");
    }

    let mut reader = EventStream::new();
    let mut spinner_tick = tokio::time::interval(std::time::Duration::from_millis(120));

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        tokio::select! {
            _ = spinner_tick.tick() => {
                app.tick_spinner();
            }
            maybe_loading = status_rx.recv() => {
                if let Some(loading) = maybe_loading {
                    app.loading = loading;
                }
            }
            maybe_data = data_rx.recv() => {
                if let Some(data) = maybe_data {
                    app.update_data(data);
                }
            }
            maybe_event = reader.next() => {
                if let Some(Ok(event)) = maybe_event {
                    match event {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            if handle_key(key.code, key.modifiers, &mut app, &ctrl_tx).await {
                                break;
                            }
                        }
                        Event::Mouse(mouse) => {
                            if matches!(mouse.kind, MouseEventKind::Down(_)) {
                                if let Ok(size) = terminal.size() {
                                    let area = Rect::new(0, 0, size.width, size.height);
                                    if let Some(pane) =
                                        ui::pane_at(area, &app.layout, mouse.column, mouse.row)
                                    {
                                        app.selected = pane;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle keyboard input, returns true if app should quit
async fn handle_key(
    code: KeyCode,
    modifiers: KeyModifiers,
    app: &mut App,
    ctrl_tx: &mpsc::Sender<ControlMsg>,
) -> bool {
    // Handle modal dialog keys
    if app.detail_modal.is_some() {
        match code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => app.close_details(),
            _ => {}
        }
        return false;
    }

    // Handle command palette
    if app.command_mode {
        return handle_command_mode(code, app, ctrl_tx).await;
    }

    // Handle normal navigation and control keys
    match code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(10) => return true,
        KeyCode::Char(':') => app.enter_command_mode(),
        KeyCode::Tab => app.select_next(),
        KeyCode::BackTab => app.select_prev(),
        KeyCode::Char('h') => app.select_directional(NavDir::Left),
        KeyCode::Char('l') => app.select_directional(NavDir::Right),
        KeyCode::Char('j') => {
            if app.can_scroll_list() {
                app.move_list_cursor(1);
            } else {
                app.select_directional(NavDir::Down);
            }
        }
        KeyCode::Char('k') => {
            if app.can_scroll_list() {
                app.move_list_cursor(-1);
            } else {
                app.select_directional(NavDir::Up);
            }
        }
        KeyCode::Left => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                app.resize_focused(-1);
            } else {
                app.select_directional(NavDir::Left);
            }
        }
        KeyCode::Right => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                app.resize_focused(1);
            } else {
                app.select_directional(NavDir::Right);
            }
        }
        KeyCode::Up => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                app.resize_rows(1);
            } else if app.can_scroll_list() {
                app.move_list_cursor(-1);
            } else {
                app.select_directional(NavDir::Up);
            }
        }
        KeyCode::Down => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                app.resize_rows(-1);
            } else if app.can_scroll_list() {
                app.move_list_cursor(1);
            } else {
                app.select_directional(NavDir::Down);
            }
        }
        KeyCode::Enter => app.open_details_for_selected(),
        KeyCode::Char('+') | KeyCode::Char('=') => app.resize_focused(1),
        KeyCode::Char('-') => app.resize_focused(-1),
        KeyCode::Char('r') | KeyCode::F(5) => {
            if ctrl_tx.send(ControlMsg::RefreshNow).await.is_err() {
                app.set_error("refresh channel closed");
            } else {
                app.loading = true;
            }
        }
        KeyCode::Char('1') => app.selected = Pane::Git,
        KeyCode::Char('2') => app.selected = Pane::System,
        KeyCode::Char('3') => app.selected = Pane::Prs,
        KeyCode::Char('4') => app.selected = Pane::Docker,
        KeyCode::Char('5') => app.selected = Pane::Aws,
        KeyCode::Char('6') => app.selected = Pane::Plugins,
        _ => {}
    }

    false
}

/// Handle command palette input, returns true if app should quit
async fn handle_command_mode(
    code: KeyCode,
    app: &mut App,
    ctrl_tx: &mpsc::Sender<ControlMsg>,
) -> bool {
    match code {
        KeyCode::Esc => {
            app.exit_command_mode();
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Enter => {
            let parsed = app.parse_command();
            app.exit_command_mode();
            match parsed {
                Ok(cmd) => {
                    return execute_palette_command(cmd, app, ctrl_tx).await;
                }
                Err(e) => app.set_error(e),
            }
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        _ => {}
    }

    false
}

/// Execute a parsed command palette command, returns true if app should quit
async fn execute_palette_command(
    cmd: PaletteCommand,
    app: &mut App,
    ctrl_tx: &mpsc::Sender<ControlMsg>,
) -> bool {
    match cmd {
        PaletteCommand::Refresh => {
            if ctrl_tx.send(ControlMsg::RefreshNow).await.is_err() {
                app.set_error("refresh channel closed");
            } else {
                app.loading = true;
                app.set_status("refresh triggered");
            }
        }
        PaletteCommand::ReloadConfig => match Config::load() {
            Ok(cfg) => {
                app.apply_config(&cfg);
                let plugins = PluginManager::from_config(&cfg.plugins);
                if ctrl_tx
                    .send(ControlMsg::ReloadRuntime { cfg, plugins })
                    .await
                    .is_err()
                {
                    app.set_error("reload channel closed");
                } else {
                    app.loading = true;
                    app.set_status("config reloaded");
                }
            }
            Err(e) => app.set_error(format!("reload failed: {e}")),
        },
        PaletteCommand::ToggleCompact => {
            app.compact_mode = !app.compact_mode;
            app.set_status(if app.compact_mode {
                "compact mode on"
            } else {
                "compact mode off"
            });
        }
        PaletteCommand::Focus(pane) => {
            app.selected = pane;
            app.set_status("focus changed");
        }
        PaletteCommand::Quit => return true,
        PaletteCommand::Help => {
            app.set_status("commands: refresh | reload | compact | focus <pane> | quit");
        }
    }

    false
}
