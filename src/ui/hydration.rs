use std::sync::{Arc, Mutex};
use anyhow::Context;
use charybdis::scylla::PagingState;
use charybdis::types::Timestamp;
use chrono::Utc;
use scylla::Session;
use crate::ui::App;

pub async fn fetch_data(app: Arc<Mutex<App>>, session: Arc<Session>) -> anyhow::Result<()> {
    let mut prepared_connected_users = session
        .prepare("SELECT chatter_id FROM twitch.connected_users_to_channel WHERE streamer_id = ? LIMIT 30")
        .await
        .context("Failed to prepare connected users query")?;

    let mut prepared_chat = session
        .prepare("SELECT sent_at, chatter_username, content, chatter_color FROM twitch.messages WHERE streamer_id = ? LIMIT 40")
        .await
        .context("Failed to prepare chat messages query")?;

    let mut prepared_streamers_events_leaderboard = session
        .prepare("SELECT streamer_id, events_count FROM twitch.streamers_events_leaderboard WHERE day = ? LIMIT 50")
        .await
        .context("Failed to prepare streamers events leaderboard query")?;

    let channel = {
        let app = app.lock().unwrap();
        app.active_channels[app.active_channel].0.clone()
    };

    // Fetch connected users
    let result = session.execute_unpaged(&prepared_connected_users, (channel.clone(),)).await
        .context("Failed to fetch connected users")?;
    let connected_users: Vec<(String, u32)> = result
        .rows_typed::<(String,)>()
        .context("Failed to parse connected users")?
        .enumerate()
        .map(|(idx, row)| (row.unwrap().0, idx as u32))
        .collect();

    // Fetch chat messages
    let (result, _) = session.execute_single_page(&prepared_chat, (channel,), PagingState::start()).await
        .context("Failed to fetch chat messages")?;
    let chat_messages: Vec<String> = result
        .rows_typed::<(Timestamp, String, String, Option<String>)>()
        .context("Failed to parse chat messages")?
        .map(|row| {
            let (sent_at, chatter_id, message, chatter_color) = row.unwrap();
            let _ = chatter_color.unwrap_or_else(|| "white".to_string());
            format!("[{}] {}: {}", sent_at, chatter_id, message)
        })
        .collect();

    // Fetch active channels
    let epoch_naive = Utc::now().date_naive();
    let (result, _) = session
        .execute_single_page(&prepared_streamers_events_leaderboard, (epoch_naive,), PagingState::start())
        .await
        .context("Failed to fetch active channels")?;
    let active_channels: Vec<(String, String, usize)> = result
        .rows_typed::<(String, i32)>()
        .context("Failed to parse active channels")?
        .enumerate()
        .map(|(idx, row)| {
            let (streamer_id, events_count) = row.unwrap();
            let formatted = format!("{} ({})", streamer_id, events_count);
            (streamer_id, formatted, idx)
        })
        .collect();


    {
        let mut app = app.lock().unwrap();
        app.connected_users = connected_users;
        app.chat_messages = chat_messages;
        if app.active_channels.len() != active_channels.len() {
            app.active_channel = 0;
        }
        app.active_channels = active_channels;
    }

    Ok(())
}
