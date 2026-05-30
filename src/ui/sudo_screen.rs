use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use super::helpers::centered_rect;
use crate::app::AppState;

pub fn render(frame: &mut Frame, state: &AppState) {
    // Render main screen behind
    super::main_screen::render(frame, state);

    let area = centered_rect(50, 10, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Senha necessária ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // info text
            Constraint::Length(1), // input label
            Constraint::Length(1), // input field
            Constraint::Length(1), // spacer
            Constraint::Length(1), // buttons
        ])
        .split(inner);

    let info = Paragraph::new("Alguns comandos precisam de sudo.\nA senha fica apenas em memória.")
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(info, chunks[0]);

    let label = Paragraph::new("Senha:")
        .style(Style::default().fg(Color::Yellow).bold());
    frame.render_widget(label, chunks[1]);

    // Show masked password
    let masked: String = "*".repeat(state.sudo_input.len());
    let input = Paragraph::new(format!("> {}_", masked))
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    frame.render_widget(input, chunks[2]);

    let buttons = Paragraph::new(
        Line::from(vec![
            Span::styled(" [Enter] Confirmar ", Style::default().bg(Color::Blue).fg(Color::White)),
            Span::raw("  "),
            Span::styled(" [Esc] Cancelar ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        ])
    );
    frame.render_widget(buttons, chunks[4]);
}
