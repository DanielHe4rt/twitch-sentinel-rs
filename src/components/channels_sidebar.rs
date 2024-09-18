use ratatui::layout::Constraint;
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table};
use crate::{App, Focus};

pub fn build_sidebar(app: &App) -> Table {

    // Draw the Sidebar with Active Channels as Table
    let sidebar_block = Block::default()
        .title("Active Channels")
        .borders(Borders::ALL)
        .border_style(if app.focused == Focus::Sidebar {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });

    let channels: Vec<Row> = app
        .active_channels
        .iter()
        .map(|(_, formatted, idx)| {
            let style = if *idx == app.active_channel {
                Style::default()
                    .underline_color(Color::Yellow)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![formatted.as_str()]).style(style)
        })
        .collect();

    Table::new(channels, [Constraint::Percentage(100)])
        .block(sidebar_block)
        .header(Row::new(vec!["Channel"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        ))
        .widths(&[Constraint::Percentage(100)])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ")
}