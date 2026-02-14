use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub const MENU_MIN_WIDTH: u16 = 60;
pub const MENU_MIN_HEIGHT: u16 = 15;
pub const MAIN_CONTENT_WIDTH: u16 = 72;
pub const MENU_LIST_WIDTH: u16 = 30;

pub struct MainMenuAreas {
    pub logo: Rect,
    pub menu: Rect,
    pub version: Rect,
}

/// Returns centered layout areas for the main menu.
pub fn main_menu_areas(area: Rect) -> MainMenuAreas {
    let content = centered_rect(area, MAIN_CONTENT_WIDTH.min(area.width), 15.min(area.height));
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(content);

    let menu_width = MENU_LIST_WIDTH.min(rows[2].width);
    let menu_x = rows[2].x + rows[2].width.saturating_sub(menu_width) / 2;

    MainMenuAreas {
        logo: rows[0],
        menu: Rect {
            x: menu_x,
            y: rows[2].y,
            width: menu_width,
            height: rows[2].height,
        },
        version: rows[4],
    }
}

/// Creates a centered rectangle within a parent area.
pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width.min(area.width)),
            Constraint::Min(0),
        ])
        .split(area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height.min(area.height)),
            Constraint::Min(0),
        ])
        .split(horizontal[1]);

    vertical[1]
}
