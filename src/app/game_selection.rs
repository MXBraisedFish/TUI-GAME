use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{symbols, widgets::Wrap};
use unicode_width::UnicodeWidthStr;

use crate::app::i18n;
use crate::app::stats::{self, GameStats, LightsOutBest, MazeEscapeBest, MemoryFlipBest, MinesweeperBest};
use crate::lua_bridge::script_loader::GameMeta;
use crate::terminal::renderer;

pub struct GameSelection {
    games: Vec<GameMeta>,
    stats: HashMap<String, GameStats>,
    lights_out_best: Option<LightsOutBest>,
    memory_flip_best: Option<MemoryFlipBest>,
    minesweeper_best: Option<MinesweeperBest>,
    maze_escape_best: Option<MazeEscapeBest>,
    list_state: ListState,
    page_state: PageState,
    launch_placeholder: bool,
}

#[derive(Clone, Copy)]
struct PageState {
    current_page: usize,
    page_size: usize,
    total_pages: usize,
}

pub enum GameSelectionAction {
    BackToMenu,
    LaunchGame(GameMeta),
}

impl GameSelection {
    /// Creates a game selection state from scanned games and local stats.
    pub fn new(games: Vec<GameMeta>) -> Self {
        let stats = stats::load_stats();
        let lights_out_best = stats::load_lights_out_best();
        let memory_flip_best = stats::load_memory_flip_best();
        let minesweeper_best = stats::load_minesweeper_best();
        let maze_escape_best = stats::load_maze_escape_best();
        let initial_page_size = games.len().max(1);

        let mut list_state = ListState::default();
        if !games.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            games,
            stats,
            lights_out_best,
            memory_flip_best,
            minesweeper_best,
            maze_escape_best,
            list_state,
            page_state: PageState {
                current_page: 0,
                page_size: initial_page_size,
                total_pages: 1,
            },
            launch_placeholder: false,
        }
    }

    /// Handles keyboard events and returns high-level actions.
    pub fn handle_event(&mut self, key: KeyEvent) -> Option<GameSelectionAction> {
        if self.launch_placeholder {
            self.launch_placeholder = false;
            return None;
        }

        match key.code {
            KeyCode::Esc => Some(GameSelectionAction::BackToMenu),
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.prev_page();
                None
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.next_page();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_prev();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                None
            }
            KeyCode::Enter => {
                if let Some(game) = self.selected_game_cloned() {
                    if game.id == "2048"
                        || game.id == "lights_out"
                        || game.id == "memory_flip"
                        || game.id == "sliding_puzzle"
                        || game.id == "color_memory"
                        || game.id == "minesweeper"
                        || game.id == "rock_paper_scissors"
                        || game.id == "blackjack"
                        || game.id == "maze_escape"
                        || game.id == "pacman"
                        || game.id == "shooter"
                    {
                        return Some(GameSelectionAction::LaunchGame(game));
                    }
                    self.launch_placeholder = true;
                }
                None
            }
            _ => None,
        }
    }

    /// Renders the game selection UI.
    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        if self.launch_placeholder {
            self.render_launch_placeholder(frame, area);
            return;
        }

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(root[0]);

        self.render_list_panel(frame, columns[0]);
        self.render_detail_panel(frame, columns[1]);

        let hints = i18n::t("game_selection.hint.controls");
        let hint_widget = Paragraph::new(hints)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint_widget, root[1]);
    }

    /// Returns the minimum terminal size needed for stable layout.
    pub fn minimum_size(&self) -> (u16, u16) {
        let list_title = i18n::t("game_selection.panel.games");
        let detail_title = i18n::t("game_selection.panel.details");
        let hint = i18n::t("game_selection.hint.controls");
        let list_title_w = UnicodeWidthStr::width(list_title.as_str());
        let detail_title_w = UnicodeWidthStr::width(detail_title.as_str());
        let hint_w = UnicodeWidthStr::width(hint.as_str());

        let max_name_w = self
            .games
            .iter()
            .map(|g| UnicodeWidthStr::width(self.localized_game_name(g).as_str()))
            .max()
            .unwrap_or(12);

        let left_w = (max_name_w.max(list_title_w) as u16 + 8).max(26);
        let right_w = (detail_title_w as u16 + 36).max(46);
        let min_w = (left_w + right_w + 4).max(hint_w as u16 + 2).max(60);
        let min_h = 12;
        (min_w, min_h)
    }

    fn render_list_panel(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::DOUBLE)
            .border_style(Style::default().fg(Color::White))
            .title(format!(" {} ", i18n::t("game_selection.panel.games")));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        self.sync_paging(rows[0].height as usize);

        let page_games: Vec<String> = self
            .current_page_games()
            .iter()
            .map(|g| self.localized_game_name(g))
            .collect();
        if page_games.is_empty() {
            let p = Paragraph::new(i18n::t("game_selection.empty"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            frame.render_widget(p, rows[0]);
            return;
        }

        let items: Vec<ListItem<'_>> = page_games
            .iter()
            .map(|name| ListItem::new(Line::from(name.clone())))
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().fg(Color::Black).bg(Color::LightBlue))
            .highlight_symbol("");
        frame.render_stateful_widget(list, rows[0], &mut self.list_state);

        let has_prev = self.page_state.current_page > 0;
        let has_next = self.page_state.current_page + 1 < self.page_state.total_pages;
        let left = if has_prev {
            i18n::t("game_selection.pager.prev")
        } else {
            String::new()
        };
        let center = format!(
            "{}/{}",
            self.page_state.current_page + 1,
            self.page_state.total_pages
        );
        let right = if has_next {
            i18n::t("game_selection.pager.next")
        } else {
            String::new()
        };

        let left_w = UnicodeWidthStr::width(left.as_str()) as u16;
        let right_w = UnicodeWidthStr::width(right.as_str()) as u16;
        let pager_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_w),
                Constraint::Min(0),
                Constraint::Length(right_w),
            ])
            .split(rows[1]);

        let left_widget = Paragraph::new(left)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        frame.render_widget(left_widget, pager_chunks[0]);

        let center_widget = Paragraph::new(center)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(center_widget, pager_chunks[1]);

        let right_widget = Paragraph::new(right)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Right);
        frame.render_widget(right_widget, pager_chunks[2]);
    }

    fn render_detail_panel(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::DOUBLE)
            .border_style(Style::default().fg(Color::White))
            .title(format!(" {} ", i18n::t("game_selection.panel.details")));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(game) = self.selected_game() else {
            let p = Paragraph::new(i18n::t("game_selection.empty"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            frame.render_widget(p, inner);
            return;
        };

        let s = self.stats.get(&game.id).copied().unwrap_or_default();
        let sep_len = inner.width.saturating_sub(1) as usize;
        let separator = "─".repeat(sep_len.max(1));
        let name = self.localized_game_name(game);
        let description = self.localized_game_description(game);

        let mut lines = vec![Line::from(Span::styled(
            name,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))];

        lines.push(Line::from(separator.clone()));
        if game.id == "lights_out" {
            if let Some(best) = self.lights_out_best {
                lines.push(Line::from(format!(
                    "{} {}x{}",
                    i18n::t("game.lights_out.best_size"),
                    best.max_size,
                    best.max_size
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.lights_out.best_steps"),
                    best.min_steps
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.lights_out.best_time"),
                    stats::format_duration(best.min_time_sec)
                )));
            } else {
                lines.push(Line::from(i18n::t("game.lights_out.best_none")));
            }
        } else if game.id == "memory_flip" {
            if let Some(best) = self.memory_flip_best {
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.memory_flip.best_difficulty"),
                    best.difficulty
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.memory_flip.best_steps"),
                    best.min_steps
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.memory_flip.best_time"),
                    stats::format_duration(best.min_time_sec)
                )));
            } else {
                lines.push(Line::from(i18n::t("game.memory_flip.best_none")));
            }
        } else if game.id == "minesweeper" {
            if let Some(best) = self.minesweeper_best {
                let fmt = |v: Option<u64>| -> String {
                    match v {
                        Some(sec) => stats::format_duration(sec),
                        None => "-".to_string(),
                    }
                };
                lines.push(Line::from(i18n::t("game.minesweeper.best_title")));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.minesweeper.best_d1"),
                    fmt(best.d1_min_time_sec)
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.minesweeper.best_d2"),
                    fmt(best.d2_min_time_sec)
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.minesweeper.best_d3"),
                    fmt(best.d3_min_time_sec)
                )));
            } else {
                lines.push(Line::from(i18n::t("game.minesweeper.best_none")));
            }
        } else if game.id == "maze_escape" {
            if let Some(best) = self.maze_escape_best {
                let size = if best.max_cols > 0 && best.max_rows > 0 {
                    format!("{}x{}", best.max_cols, best.max_rows)
                } else if best.max_area > 0 {
                    best.max_area.to_string()
                } else {
                    "-".to_string()
                };
                let fastest = best
                    .min_time_sec
                    .map(stats::format_duration)
                    .unwrap_or_else(|| "-".to_string());
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.maze_escape.best_max_size"),
                    size
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.maze_escape.best_max_mode"),
                    best.max_mode
                )));
                lines.push(Line::from(format!(
                    "{} {}",
                    i18n::t("game.maze_escape.best_fastest"),
                    fastest
                )));
            } else {
                lines.push(Line::from(i18n::t("game.maze_escape.best_none")));
            }
        } else if game.id == "pacman" {
            lines.push(Line::from(format!(
                "{} {}",
                i18n::t("game_selection.label.high_score"),
                s.high_score
            )));
        } else if game.id == "rock_paper_scissors" {
            lines.push(Line::from(format!(
                "{} {}",
                i18n::t("game.rock_paper_scissors.best_streak"),
                s.high_score
            )));
        } else if game.id == "blackjack" {
            lines.push(Line::from(format!(
                "{} {}",
                i18n::t("game_selection.label.high_net_profit"),
                s.high_score
            )));
        } else {
            lines.push(Line::from(format!(
                "{} {}",
                i18n::t("game_selection.label.high_score"),
                s.high_score
            )));
            lines.push(Line::from(format!(
                "{} {}",
                i18n::t("game_selection.label.longest_play"),
                stats::format_duration(s.max_duration_sec)
            )));
        }
        lines.push(Line::from(separator));
        lines.push(Line::from(i18n::t("game_selection.label.how_to_play")));

        let wrapped = renderer::wrap_text(&description, inner.width.saturating_sub(1) as usize);
        for w in wrapped {
            lines.push(Line::from(w));
        }

        let paragraph = Paragraph::new(lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    fn render_launch_placeholder(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let width = 32u16.min(area.width.saturating_sub(2));
        let height = 5u16.min(area.height.saturating_sub(2));
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        let rect = Rect::new(x, y, width.max(1), height.max(1));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::DOUBLE)
            .border_style(Style::default().fg(Color::White));
        let inner = block.inner(rect);
        frame.render_widget(block, rect);

        let msg = Paragraph::new(format!(
            "{}\n{}",
            i18n::t("game_selection.placeholder.title"),
            i18n::t("game_selection.placeholder.back")
        ))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
    }

    fn selected_game(&self) -> Option<&GameMeta> {
        let selected_in_page = self.list_state.selected()?;
        let global = self.page_state.current_page * self.page_state.page_size + selected_in_page;
        self.games.get(global)
    }

    fn selected_game_cloned(&self) -> Option<GameMeta> {
        self.selected_game().cloned()
    }

    fn current_page_games(&self) -> &[GameMeta] {
        let start = self.page_state.current_page * self.page_state.page_size;
        let end = (start + self.page_state.page_size).min(self.games.len());
        &self.games[start..end]
    }

    fn select_prev(&mut self) {
        let page_len = self.current_page_games().len();
        if page_len == 0 {
            self.list_state.select(None);
            return;
        }

        let selected = self.list_state.selected().unwrap_or(0);
        if selected > 0 {
            self.list_state.select(Some(selected - 1));
        }
    }

    fn select_next(&mut self) {
        let page_len = self.current_page_games().len();
        if page_len == 0 {
            self.list_state.select(None);
            return;
        }

        let selected = self.list_state.selected().unwrap_or(0);
        if selected + 1 < page_len {
            self.list_state.select(Some(selected + 1));
        }
    }

    fn prev_page(&mut self) {
        if self.page_state.current_page > 0 {
            self.page_state.current_page -= 1;
            self.list_state.select(Some(0));
        }
    }

    fn next_page(&mut self) {
        if self.page_state.current_page + 1 < self.page_state.total_pages {
            self.page_state.current_page += 1;
            self.list_state.select(Some(0));
        }
    }

    fn localized_game_name(&self, game: &GameMeta) -> String {
        i18n::t_or(&format!("game.{}.name", game.id), &game.name)
    }

    fn localized_game_description(&self, game: &GameMeta) -> String {
        i18n::t_or(&format!("game.{}.description", game.id), &game.description)
    }

    fn selected_global_index(&self) -> Option<usize> {
        let selected_in_page = self.list_state.selected()?;
        let global = self.page_state.current_page * self.page_state.page_size + selected_in_page;
        if global < self.games.len() {
            Some(global)
        } else {
            None
        }
    }

    fn sync_paging(&mut self, visible_rows: usize) {
        let page_size = visible_rows.max(1);
        let selected_global = self.selected_global_index().unwrap_or(0);

        self.page_state.page_size = page_size;
        self.page_state.total_pages = ((self.games.len() + page_size.saturating_sub(1)) / page_size).max(1);

        if self.games.is_empty() {
            self.page_state.current_page = 0;
            self.list_state.select(None);
            return;
        }

        let clamped_global = selected_global.min(self.games.len() - 1);
        self.page_state.current_page =
            (clamped_global / page_size).min(self.page_state.total_pages.saturating_sub(1));

        let start = self.page_state.current_page * page_size;
        let page_len = (self.games.len() - start).min(page_size);
        let selected_in_page = (clamped_global - start).min(page_len.saturating_sub(1));
        self.list_state.select(Some(selected_in_page));
    }
}


