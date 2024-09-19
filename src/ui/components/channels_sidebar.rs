use std::ops::Deref;
use ratatui::layout::Constraint;
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table};
use crate::ui::{App, Focus};

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
        .enumerate()
        .map(|(idx, data)| {
            let style = if idx == app.active_channel {
                Style::default()
                    .underline_color(Color::Yellow)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let fodase = data.events_count.clone().to_string();
            Row::new(vec![format!("{} ({}) ", data.streamer_id.to_string(), fodase)]).style(style)
        })
        .collect();

    Table::new(channels, [Constraint::Percentage(100)])
        .block(sidebar_block)
        .header(Row::new(vec!["Channel", "Events Count"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        ))
        .widths(&[Constraint::Percentage(100)])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ")
}