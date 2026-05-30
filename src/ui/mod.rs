pub mod main_screen;
pub mod reboot_screen;
pub mod helpers;

use ratatui::Frame;
use crate::app::{AppState, Screen};

pub fn render(frame: &mut Frame, state: &AppState) {
    match &state.screen {
        Screen::RebootRequired { app_name, pending } => {
            reboot_screen::render(frame, state, app_name, pending)
        }
        Screen::Main => main_screen::render(frame, state),
    }
}
