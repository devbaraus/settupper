use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use super::helpers::centered_rect;
use crate::app::AppState;

pub fn render(frame: &mut Frame, state: &AppState, app_name: &str, pending: &[String]) {
    super::main_screen::render(frame, state);

    let height = (6 + pending.len() as u16).min(20);
    let area = centered_rect(55, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Reboot necessário ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(app_name, Style::default().fg(Color::Yellow).bold()),
            Span::raw(" requer reinicialização."),
        ]),
        Line::from(""),
    ];

    if !pending.is_empty() {
        lines.push(Line::from(Span::styled("Itens pendentes:", Style::default().bold())));
        for item in pending {
            lines.push(Line::from(format!("  • {}", item)));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        " [Enter/Esc] OK ",
        Style::default().bg(Color::Blue).fg(Color::White),
    )));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
