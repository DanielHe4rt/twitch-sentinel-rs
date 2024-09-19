use crate::models::message::Message;
use crate::ui::{App, Focus};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::ops::Deref;

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

    let chat_content = app
        .chat_messages
        .iter()
        .map(|message| {
            let content = message.content.as_deref().unwrap_or("[No content]");
            let chatter = message.chatter_username.as_deref().unwrap_or("[Unknown]");
            let sent_at = message
                .sent_at
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or("[Unknown time]".to_string());

            format!("[{}] {}: {}", sent_at, chatter, content)
        })
        .collect::<Vec<String>>()
        .join("\n");

    Paragraph::new(chat_content).block(chat_block)
}
