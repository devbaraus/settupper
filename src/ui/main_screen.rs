use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
};
use crate::app::AppState;
use crate::core::select_commands;
use super::helpers::{status_label, smart_action_label};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    // Header + body + footer layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer
        ])
        .split(area);

    render_header(frame, state, chunks[0]);
    render_body(frame, state, chunks[1]);
    render_footer(frame, chunks[2]);
}

fn render_header(frame: &mut Frame, state: &AppState, area: Rect) {
    let title = format!(" Settupper — {} ", state.distro.as_str());
    let mode = if state.dry_run { " [DRY-RUN]" } else { "" };
    let busy = if state.busy { " ⟳" } else { "" };
    let text = format!("{}{}{}", title, mode,  busy);

    let header = Paragraph::new(text)
        .style(Style::default().fg(Color::White).bg(Color::Blue));
    frame.render_widget(header, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let keys = " q:sair  i:instalar  u:atualizar  d:desinstalar  a:smart  A:smart-all  r:refresh  space:selecionar  esc:limpar  e:exportar";
    let footer = Paragraph::new(keys)
        .style(Style::default().fg(Color::White).bg(Color::Blue));
    frame.render_widget(footer, area);
}

fn render_body(frame: &mut Frame, state: &AppState, area: Rect) {
    let split_x = (area.width as f64 * state.split_ratio) as u16;
    let left_width = split_x.max(20).min(area.width.saturating_sub(2));
    let right_width = area.width.saturating_sub(left_width + 1);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_width),
            Constraint::Length(1),
            Constraint::Length(right_width),
        ])
        .split(area);

    render_left_pane(frame, state, chunks[0]);
    render_divider(frame, chunks[1]);
    render_right_pane(frame, state, chunks[2]);
}

fn render_divider(frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = (0..area.height)
        .map(|_| Line::from(Span::styled("│", Style::default().fg(Color::DarkGray))))
        .collect();
    let widget = Paragraph::new(Text::from(lines));
    frame.render_widget(widget, area);
}

fn render_left_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    // Group filter row (height 1 if groups exist, else 0) + summary + table + buttons
    let has_groups = !state.config.groups.is_empty();
    let group_h = if has_groups { 1 } else { 0 };
    let button_h = 3u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(group_h),
            Constraint::Length(1), // summary
            Constraint::Min(0),    // table
            // Constraint::Length(button_h), // buttons
        ])
        .split(area);

    if has_groups {
        render_group_filter(frame, state, chunks[0]);
    }
    render_summary(frame, state, chunks[1]);
    render_table(frame, state, chunks[2]);
    // render_buttons(frame, state, chunks[3]);
}

fn render_group_filter(frame: &mut Frame, state: &AppState, area: Rect) {
    let groups = state.groups();
    let current = if state.active_group.is_empty() {
        "Todos".to_string()
    } else {
        groups
            .iter()
            .find(|g| g.id == state.active_group)
            .map(|g| g.name.clone())
            .unwrap_or_else(|| state.active_group.clone())
    };

    let text = format!(" Grupo: [{}] (tab)", current);
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(widget, area);
}

fn render_summary(frame: &mut Frame, state: &AppState, area: Rect) {
    let installed = state.statuses.iter().filter(|s| s.installed && s.supported).count();
    let missing = state.statuses.iter().filter(|s| !s.installed && s.supported).count();
    let skip = state.statuses.iter().filter(|s| !s.supported).count();
    let sel = state.selected_indices.len();

    let text = format!(
        " OK:{} MISS:{} SKIP:{}{}",
        installed,
        missing,
        skip,
        if sel > 0 { format!("  sel:{}", sel) } else { String::new() }
    );

    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::White));
    frame.render_widget(widget, area);
}

fn render_table(frame: &mut Frame, state: &AppState, area: Rect) {
    let header_cells = ["", "Status", "App", "Grupo", "Descrição", "Deps"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().bold().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = state
        .visible_indices
        .iter()
        .enumerate()
        .map(|(row_idx, &real_idx)| {
            let is_cursor = row_idx == state.table_cursor;
            let is_selected = state.selected_indices.contains(&real_idx);

            let status = state.statuses.get(real_idx);
            let app = &state.config.apps[real_idx];

            let check_char = if is_selected { "☑" } else { "○" };
            let check_style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let (status_text, status_style) = if let Some(s) = status {
                let lbl = status_label(s);
                let mut style = Style::default().fg(lbl.color);
                if lbl.bold {
                    style = style.bold();
                }
                (lbl.text.to_string(), style)
            } else {
                ("...".to_string(), Style::default().fg(Color::DarkGray))
            };

            let deps = if app.depends_on.is_empty() {
                "-".to_string()
            } else {
                app.depends_on.join(", ")
            };

            let row_style = if is_cursor {
                Style::default().bg(Color::DarkGray)
            } else if row_idx % 2 == 0 {
                Style::default()
            } else {
                Style::default().bg(Color::Rgb(20, 20, 30))
            };

            Row::new(vec![
                Cell::from(check_char).style(check_style),
                Cell::from(status_text).style(status_style),
                Cell::from(app.name.clone()),
                Cell::from(if app.group.is_empty() { "-".to_string() } else { app.group.clone() }),
                Cell::from(app.description.clone()),
                Cell::from(deps),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // checkbox
        Constraint::Length(6),  // status
        Constraint::Length(18), // name
        Constraint::Length(10), // group
        Constraint::Fill(1),    // description
        Constraint::Length(12), // deps
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Apps "))
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut table_state = TableState::default();
    if !state.visible_indices.is_empty() {
        table_state.select(Some(state.table_cursor));
    }

    frame.render_stateful_widget(table, area, &mut table_state);
}

fn render_buttons(frame: &mut Frame, state: &AppState, area: Rect) {
    let busy = state.busy;
    let style_active = Style::default().fg(Color::White).bg(Color::Blue);
    let style_dim = Style::default().fg(Color::DarkGray);

    let line = Line::from(vec![
        Span::styled(
            " [i]nstalar ",
            if busy { style_dim } else { style_active },
        ),
        Span::raw(" "),
        Span::styled(
            " [u]pdate ",
            if busy { style_dim } else { style_active },
        ),
        Span::raw(" "),
        Span::styled(
            " [a]smart ",
            if busy { style_dim } else { style_active },
        ),
        Span::raw(" "),
        Span::styled(
            " [A]smart-all ",
            if busy { style_dim } else { style_active },
        ),
        Span::raw(" "),
        Span::styled(
            " [d]esinstalar ",
            if busy { style_dim } else { style_active },
        ),
        Span::raw(" "),
        Span::styled(
            " [r]efresh ",
            style_active,
        ),
    ]);

    let widget = Paragraph::new(line)
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(widget, area);
}

fn render_right_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    // Details panel (fixed) + log (fill)
    let detail_h = 13u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(detail_h), Constraint::Min(0)])
        .split(area);

    render_details(frame, state, chunks[0]);
    render_log(frame, state, chunks[1]);
}

fn render_details(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Detalhes ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(&real_idx) = state.visible_indices.get(state.table_cursor) else {
        return;
    };

    let app = &state.config.apps[real_idx];
    let status = state.statuses.get(real_idx);

    let distro = &state.distro;
    let has_install = select_commands(&app.install, distro).is_some();
    let has_update = select_commands(&app.update, distro).is_some();
    let has_uninstall = select_commands(&app.uninstall, distro).is_some();
    let reboot_install = app.reboot_on.get("install").copied().unwrap_or(false);
    let reboot_update = app.reboot_on.get("update").copied().unwrap_or(false);
    let reboot_uninstall = app.reboot_on.get("uninstall").copied().unwrap_or(false);

    let deps = if app.depends_on.is_empty() {
        "nenhum".to_string()
    } else {
        app.depends_on.join(", ")
    };

    let smart = if status.is_some() {
        smart_action_label(state, real_idx)
    } else {
        "..."
    };

    let (installed_str, supported_str) = if let Some(s) = status {
        let inst = if s.installed { "instalado" } else { "não instalado" };
        let supp = if s.supported { "sim" } else { "não" };
        (inst, supp)
    } else {
        ("verificando...", "...")
    };

    let yn = |b: bool| if b { "sim" } else { "não" };
    let conf = |has: bool| if has { "sim" } else { "NÃO CONFIGURADO" };

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("App: ", Style::default().bold()),
            Span::raw(&app.name),
        ]),
        Line::from(vec![
            Span::styled("Grupo: ", Style::default().bold()),
            Span::raw(if app.group.is_empty() { "-" } else { &app.group }),
        ]),
        Line::from(vec![
            Span::styled("Descrição: ", Style::default().bold()),
            Span::raw(if app.description.is_empty() { "-" } else { &app.description }),
        ]),
        Line::from(vec![
            Span::styled("Depende de: ", Style::default().bold()),
            Span::raw(&deps),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().bold()),
            Span::styled(
                installed_str,
                if installed_str == "instalado" {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("Suporte: ", Style::default().bold()),
            Span::raw(supported_str),
        ]),
        Line::from(vec![
            Span::styled("install: ", Style::default().bold()),
            Span::raw(conf(has_install)),
            Span::raw("  reboot: "),
            Span::raw(yn(reboot_install)),
        ]),
        Line::from(vec![
            Span::styled("update: ", Style::default().bold()),
            Span::raw(conf(has_update)),
            Span::raw("  reboot: "),
            Span::raw(yn(reboot_update)),
        ]),
        Line::from(vec![
            Span::styled("uninstall: ", Style::default().bold()),
            Span::raw(conf(has_uninstall)),
            Span::raw("  reboot: "),
            Span::raw(yn(reboot_uninstall)),
        ]),
        Line::from(vec![
            Span::styled("Ação smart: ", Style::default().bold()),
            Span::raw(smart),
        ]),
    ];

    // Warnings
    if let Some(s) = status {
        if s.supported {
            if !has_install {
                lines.push(Line::from(Span::styled(
                    "! sem comando install para esta distro",
                    Style::default().fg(Color::Red),
                )));
            }
            if !has_update {
                lines.push(Line::from(Span::styled(
                    "~ sem comando update para esta distro",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        // Check output
        if !s.message.is_empty() {
            lines.push(Line::from(Span::styled(
                s.message.trim(),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((state.detail_scroll, 0));
    frame.render_widget(paragraph, inner);
}

fn render_log(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Log ");
    let inner = block.inner(area);
    frame.render_widget(&block, area);

    let log_height = inner.height as usize;
    let start = state.log_lines.len().saturating_sub(log_height);
    let visible = &state.log_lines[start..];

    let lines: Vec<Line> = visible
        .iter()
        .map(|l| {
            let style = if l.starts_with("$") {
                Style::default().fg(Color::Cyan).bold()
            } else if l.contains("exit=0") || l.contains("OK") {
                Style::default().fg(Color::Green)
            } else if l.contains("exit=1") || l.contains("ERRO") || l.contains("error") {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            Line::from(Span::styled(l.as_str(), style))
        })
        .collect();

    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, inner);
}
