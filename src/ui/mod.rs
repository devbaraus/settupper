pub mod main_screen;
pub mod sudo_screen;
pub mod reboot_screen;
pub mod helpers;

use ratatui::{Frame, layout::{Layout, Constraint, Direction, Rect}};
use crate::app::{AppState, Screen};

pub fn render(frame: &mut Frame, state: &AppState) {
    match &state.screen {
        Screen::SudoPassword => sudo_screen::render(frame, state),
        Screen::RebootRequired { app_name, pending } => {
            reboot_screen::render(frame, state, app_name, pending)
        }
        Screen::Main | Screen::Confirm { .. } => main_screen::render(frame, state),
    }
}
