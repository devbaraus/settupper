use ratatui::{layout::Rect, style::Color};
use crate::core::AppStatus;
use crate::app::{Action, AppState};

pub struct StatusLabel {
    pub text: &'static str,
    pub color: Color,
    pub bold: bool,
}

pub fn status_label(status: &AppStatus) -> StatusLabel {
    if !status.supported {
        return StatusLabel { text: "SKIP", color: Color::DarkGray, bold: false };
    }
    let has_update = !status.app.update.is_empty();
    if status.installed {
        if has_update {
            StatusLabel { text: "OK  ", color: Color::Green, bold: true }
        } else {
            StatusLabel { text: "OK ~", color: Color::Yellow, bold: true }
        }
    } else {
        let has_install = !status.app.install.is_empty();
        if has_install {
            StatusLabel { text: "MISS", color: Color::Red, bold: false }
        } else {
            StatusLabel { text: "MISS!", color: Color::Red, bold: true }
        }
    }
}

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

pub fn smart_action_label(state: &AppState, idx: usize) -> &'static str {
    match state.smart_action(idx) {
        Some(Action::Install) => "install",
        Some(Action::Update) => "update",
        _ => "nenhuma",
    }
}