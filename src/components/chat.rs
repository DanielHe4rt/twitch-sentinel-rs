
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::{App, Focus};

pub fn build_chat(app: &App) -> Paragraph {
    // Draw the Main Chat as Paragraph (Could be replaced with a Table if needed)
    let chat_block = Block::default()
        .title("Chat")
        .borders(Borders::ALL)
        .border_style(if app.focused == Focus::MainChat {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });

    let chat_content = app.chat_messages.join("\n");

    Paragraph::new(chat_content)
        .block(chat_block)
}