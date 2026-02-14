use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::i18n::t;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaceholderPage {
    Settings,
    About,
    Continue,
}

/// Renders a simple placeholder page.
pub fn render_placeholder(frame: &mut ratatui::Frame<'_>, page: PlaceholderPage, version: &str) {
    let message = match page {
        PlaceholderPage::Settings => t("placeholder.settings").to_string(),
        PlaceholderPage::About => format!(
            "{}\n{} {}",
            t("placeholder.about"),
            t("placeholder.runtime_version"),
            version
        ),
        PlaceholderPage::Continue => t("placeholder.continue").to_string(),
    };

    let text = format!("{}\n\n{}", message, t("common.back_hint"));
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, frame.area());
}
