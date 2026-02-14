mod app;
mod lua_bridge;
mod terminal;
mod updater;
mod utils;

use std::io::{self, Stdout};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::game_selection::{GameSelection, GameSelectionAction};
use crate::app::i18n;
use crate::app::layout::{MENU_MIN_HEIGHT, MENU_MIN_WIDTH};
use crate::app::menu::{Menu, MenuAction};
use crate::app::placeholder_pages::{self, PlaceholderPage};
use crate::app::settings;
use crate::lua_bridge::api::{
    LaunchMode, clear_active_game_save, latest_saved_game_id, run_game_script,
    take_terminal_dirty_from_lua,
};
use crate::lua_bridge::script_loader::{GameMeta, scan_scripts};
use crate::terminal::size_watcher;
use crate::updater::github::{
    CURRENT_VERSION_TAG, UpdateNotification, Updater, UpdaterEvent, run_external_update_script,
};

pub enum AppState {
    MainMenu { menu: Menu },
    GameSelection { ui: GameSelection },
    Settings { selected: usize },
    About,
    Continue,
    Exiting,
}

struct PendingNewGameStart {
    target_game: GameMeta,
    saved_game_name: String,
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen, Hide)?;
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn install_panic_hook() {
    let old = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut out = io::stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen);
        old(panic_info);
    }));
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
    }
}

fn run() -> Result<()> {
    install_panic_hook();
    i18n::init("us-en")?;

    let mut session = TerminalSession::new()?;
    let updater = Updater::spawn(CURRENT_VERSION_TAG);

    let mut update_notification: Option<UpdateNotification> = None;
    let mut state = AppState::MainMenu { menu: Menu::new() };
    let mut pending_new_game_start: Option<PendingNewGameStart> = None;

    let frame_budget = Duration::from_millis(16);

    loop {
        let frame_start = Instant::now();

        while let Some(event) = updater.try_recv() {
            if let UpdaterEvent::NewVersion(notification) = event {
                update_notification = Some(notification);
            }
        }

        if let AppState::MainMenu { menu } = &mut state {
            sync_continue_item(menu);
        }

        if event::poll(Duration::from_millis(0))? {
            let ev = event::read()?;
            if let Event::Key(key) = ev {
                handle_key_event(
                    &mut state,
                    &mut pending_new_game_start,
                    key,
                    update_notification.as_ref(),
                )?;
            }
        }

        if take_terminal_dirty_from_lua() {
            session.terminal.clear()?;
        }

        if matches!(state, AppState::Exiting) {
            break;
        }

        let (min_width, min_height) = minimum_size_for_state(&state);
        let size_state = size_watcher::check_size(min_width, min_height)?;

        if size_state.size_ok {
            session.terminal.draw(|frame| match &mut state {
                AppState::MainMenu { menu } => {
                    let version_hint = update_notification
                        .as_ref()
                        .map(|update| update.latest_version.as_str());
                    app::menu::render_main_menu(frame, menu, CURRENT_VERSION_TAG, version_hint);
                }
                AppState::GameSelection { ui } => {
                    if let Some(pending) = pending_new_game_start.as_ref() {
                        render_new_game_confirm(frame, &pending.saved_game_name);
                    } else {
                        ui.render(frame, frame.area());
                    }
                }
                AppState::Settings { selected } => {
                    settings::render_language_selector(frame, *selected);
                }
                AppState::About => {
                    placeholder_pages::render_placeholder(
                        frame,
                        PlaceholderPage::About,
                        CURRENT_VERSION_TAG,
                    );
                }
                AppState::Continue => {
                    placeholder_pages::render_placeholder(
                        frame,
                        PlaceholderPage::Continue,
                        CURRENT_VERSION_TAG,
                    );
                }
                AppState::Exiting => {}
            })?;
        } else {
            size_watcher::draw_size_warning(&size_state, min_width, min_height)?;
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_budget {
            thread::sleep(frame_budget - elapsed);
        }
    }

    Ok(())
}

fn minimum_size_for_state(state: &AppState) -> (u16, u16) {
    match state {
        AppState::MainMenu { .. } => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::GameSelection { ui } => ui.minimum_size(),
        AppState::Settings { .. } => settings::minimum_size(),
        AppState::About | AppState::Continue => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
        AppState::Exiting => (MENU_MIN_WIDTH, MENU_MIN_HEIGHT),
    }
}

fn handle_key_event(
    state: &mut AppState,
    pending_new_game_start: &mut Option<PendingNewGameStart>,
    key: KeyEvent,
    update_notification: Option<&UpdateNotification>,
) -> Result<()> {
    if !matches!(key.kind, KeyEventKind::Press) {
        return Ok(());
    }

    if matches!(key.code, KeyCode::Char('u') | KeyCode::Char('U')) {
        if let Some(notification) = update_notification {
            if run_external_update_script(notification).unwrap_or(false) {
                *state = AppState::Exiting;
                return Ok(());
            }
        }
    }

    if !matches!(state, AppState::GameSelection { .. }) {
        *pending_new_game_start = None;
    }

    match state {
        AppState::MainMenu { menu } => match key.code {
            KeyCode::Up | KeyCode::Char('k') => menu.previous(),
            KeyCode::Down | KeyCode::Char('j') => menu.next(),
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(index) = c
                    .to_digit(10)
                    .map(|v| v as usize)
                    .and_then(|v| v.checked_sub(1))
                {
                    menu.set_selected(index);
                }
            }
            KeyCode::Esc => {
                let _ = menu.select_by_shortcut(KeyCode::Esc);
            }
            KeyCode::Enter => {
                if let Some(action) = menu.selected_action() {
                    if matches!(action, MenuAction::Continue) && !menu.can_continue() {
                        return Ok(());
                    }
                    *state = apply_menu_action(action, menu.continue_game_id());
                }
            }
            _ => {}
        },
        AppState::GameSelection { ui } => {
            if pending_new_game_start.is_some() {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        let pending = pending_new_game_start.take();
                        if let Some(pending) = pending {
                            if let Err(err) = clear_active_game_save() {
                                eprintln!("Failed to clear active save slot: {err:#}");
                            }
                            if let Err(err) =
                                run_game_script(&pending.target_game.script_path, LaunchMode::New)
                            {
                                eprintln!(
                                    "Failed to run game '{}': {err:#}",
                                    pending.target_game.id
                                );
                            }
                            let games = scan_scripts().unwrap_or_default();
                            *ui = GameSelection::new(games);
                        }
                    }
                    KeyCode::Char('n')
                    | KeyCode::Char('N')
                    | KeyCode::Char('q')
                    | KeyCode::Char('Q')
                    | KeyCode::Esc => {
                        *pending_new_game_start = None;
                    }
                    _ => {}
                }
                return Ok(());
            }

            if let Some(action) = ui.handle_event(key) {
                match action {
                    GameSelectionAction::BackToMenu => {
                        *pending_new_game_start = None;
                        *state = AppState::MainMenu { menu: Menu::new() };
                    }
                    GameSelectionAction::LaunchGame(game) => {
                        if let Some(saved_game_id) = latest_saved_game_id() {
                            let saved_game_name =
                                i18n::t_or(&format!("game.{}.name", saved_game_id), &saved_game_id);
                            *pending_new_game_start = Some(PendingNewGameStart {
                                target_game: game,
                                saved_game_name,
                            });
                            return Ok(());
                        }
                        if let Err(err) = clear_active_game_save() {
                            eprintln!("Failed to clear active save slot: {err:#}");
                        }
                        if let Err(err) = run_game_script(&game.script_path, LaunchMode::New) {
                            eprintln!("Failed to run game '{}': {err:#}", game.id);
                        }
                        let games = scan_scripts().unwrap_or_default();
                        *ui = GameSelection::new(games);
                    }
                }
            }
        }
        AppState::Settings { selected } => {
            let languages = i18n::available_languages();
            if languages.is_empty() {
                return Ok(());
            }
            if *selected >= languages.len() {
                *selected = languages.len() - 1;
            }
            let (term_width, _) = crossterm::terminal::size().unwrap_or((80, 24));
            let metrics = settings::grid_metrics(term_width, &languages);

            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                    *selected = settings::move_selection(*selected, key.code, metrics, languages.len());
                }
                KeyCode::Enter => {
                    if let Some(pack) = languages.get(*selected) {
                        let _ = i18n::set_language(&pack.code);
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    *state = AppState::MainMenu { menu: Menu::new() };
                }
                _ => {}
            }
        }
        AppState::About | AppState::Continue => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                *state = AppState::MainMenu { menu: Menu::new() }
            }
            _ => {}
        },
        AppState::Exiting => {}
    }

    Ok(())
}

fn render_new_game_confirm(frame: &mut ratatui::Frame<'_>, saved_game_name: &str) {
    use ratatui::layout::{Alignment, Constraint, Direction, Layout};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    let area = frame.area();
    frame.render_widget(Clear, area);

    let template = i18n::t(
        "confirm.new_game_overwrite",
    );
    let msg = if template.contains("{game}") {
        template.replace("{game}", saved_game_name)
    } else {
        format!("{template} {saved_game_name}")
    };
    let yes = i18n::t("confirm.new_game_yes");
    let no = i18n::t("confirm.new_game_no");

    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4), Constraint::Min(0)])
        .split(area);

    let p = Paragraph::new(vec![
        Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("{yes}  {no}"),
            Style::default().fg(Color::White),
        )),
    ])
    .style(Style::default().bg(Color::Black))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: false });
    frame.render_widget(p, center[1]);
}

fn apply_menu_action(action: MenuAction, continue_game_id: Option<&str>) -> AppState {
    match action {
        MenuAction::Play => {
            let games = match scan_scripts() {
                Ok(found) => found,
                Err(_) => Vec::new(),
            };
            AppState::GameSelection {
                ui: GameSelection::new(games),
            }
        }
        MenuAction::Continue => {
            if let Some(game_id) = continue_game_id {
                let game = scan_scripts()
                    .unwrap_or_default()
                    .into_iter()
                    .find(|g| g.id.eq_ignore_ascii_case(game_id));
                if let Some(game) = game {
                    if let Err(err) = run_game_script(&game.script_path, LaunchMode::Continue) {
                        eprintln!("Failed to continue game '{}': {err:#}", game.id);
                    }
                }
            }
            let games = scan_scripts().unwrap_or_default();
            AppState::GameSelection {
                ui: GameSelection::new(games),
            }
        }
        MenuAction::Settings => AppState::Settings {
            selected: settings::default_selected_index(),
        },
        MenuAction::About => AppState::About,
        MenuAction::Quit => AppState::Exiting,
    }
}

fn sync_continue_item(menu: &mut Menu) {
    let game_id = latest_saved_game_id();
    let game_name = game_id
        .as_deref()
        .map(|id| i18n::t_or(&format!("game.{}.name", id), id));
    menu.set_continue_target(game_id, game_name);
}
