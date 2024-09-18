use crate::App;
use ratatui::layout::Constraint;
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table};

pub fn build_metrics_table(app: &App) -> Table {
    let top_bar_block = Block::default()
        .title("Metrics")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    // Example Metrics Data
    let metrics = vec![
        vec!["10 req/s", "312312", "123", "123"],
    ];

    Table::new(metrics.iter().map(|r| Row::new(r.clone())).collect::<Vec<Row>>(), [Constraint::Percentage(100)])
        .block(top_bar_block)
        .header(Row::new(vec!["Scylla (req /s)", "Twitch Total Events", "P99", "Total"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        ))
        .widths(&[Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ")
}