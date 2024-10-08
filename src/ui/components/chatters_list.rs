
use ratatui::layout::Constraint;
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table};
use crate::ui::App;

pub fn build_chatters_list(app: &App) -> Table {
    let users_block = Block::default()
        .title("Connected Users")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let users: Vec<Row> = app
        .connected_users
        .iter().enumerate()
        .map(|(idx, data)| {
            let chatter_id = data.chatter_id.as_ref().unwrap();
            
            Row::new(vec![chatter_id.to_string(), idx.to_string()])
        })
        .collect();

    Table::new(users, [Constraint::Percentage(100)])
        .block(users_block)
        .header(
            Row::new(vec!["Username", "Messages Sent"]).style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ")
}