use ratatui::layout::Constraint;
use crate::{App, Focus};
use ratatui::prelude::{Color, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Row, Table, Tabs};

pub fn build_right_bar(app: &App) -> Tabs {
    let right_bar_block = Block::default()
        .title("Details")
        .borders(Borders::ALL)
        .border_style(if app.focused == Focus::RightBar {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });
    
    // Tabs for the right bar
    let tabs = Tabs::new(app.tabs.iter().cloned().map(Span::from).collect::<Vec<Span>>())
        .select(app.active_tab as usize)
        .block(right_bar_block)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .style(Style::default().fg(Color::White))
        .divider(Span::raw("|"));
    tabs
}

pub fn build_ranking(app: &App) -> Table {
    // Rankings as Table
    let rankings_block = Block::default()
        .title("Rankings")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let rankings: Vec<Row> = app
        .rankings
        .iter()
        .map(|(user, msg)| Row::new(vec![user.as_str(), "msg.to_string()"]))
        .collect();

     Table::new(rankings, [Constraint::Percentage(100)])
        .block(rankings_block)
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