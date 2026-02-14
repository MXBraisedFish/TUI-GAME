use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Clears the whole terminal viewport.
pub fn clear() -> Result<()> {
    let mut out = stdout();
    queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;
    out.flush()?;
    Ok(())
}

/// Draws text at absolute terminal coordinates.
pub fn draw_text(x: u16, y: u16, text: &str) -> Result<()> {
    let mut out = stdout();
    queue!(out, MoveTo(x, y), Print(text))?;
    out.flush()?;
    Ok(())
}

/// Wraps text by display width using Unicode width rules.
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if UnicodeWidthStr::width(raw_line) <= max_width {
            lines.push(raw_line.to_string());
            continue;
        }

        let mut current = String::new();
        let mut width = 0;

        for ch in raw_line.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if width + w > max_width && !current.is_empty() {
                lines.push(current.clone());
                current.clear();
                width = 0;
            }
            current.push(ch);
            width += w;
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}
