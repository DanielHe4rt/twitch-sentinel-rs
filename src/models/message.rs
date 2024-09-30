use crate::models::streams_metrics::StreamersEventsLeaderboard;
use charybdis::macros::{charybdis_model, charybdis_view_model};
use charybdis::types::{Set, Text, Timestamp, Uuid};
use chrono::Utc;
use scylla::query::Query;
use scylla::CachingSession;
use std::str::FromStr;
use futures::StreamExt;
use twitch_irc::message::PrivmsgMessage;

#[derive(Default, Clone)]
#[charybdis_model(
    table_name = messages,
    partition_keys = [streamer_id],
    clustering_keys = [sent_at, chatter_username],
    table_options = r#"
          CLUSTERING ORDER BY (sent_at DESC, chatter_username ASC)
      "#
)]
pub struct Message {
    pub message_id: Option<Uuid>,
    pub streamer_id: Text,
    pub chatter_id: Option<Text>,
    pub content: Option<Text>,
    pub chatter_username: Option<Text>,
    pub chatter_badges: Option<Set<Text>>,
    pub chatter_color: Option<Text>,
    pub sent_at: Option<Timestamp>,
}

impl Message {
    pub async fn get_leaderboard(
        session: &CachingSession,
        streamer: String,
    ) -> anyhow::Result<(Vec<Message>)> {
        let query = "SELECT message_id, streamer_id, chatter_id, content, chatter_username, chatter_badges, chatter_color, sent_at FROM messages WHERE streamer_id = ? LIMIT 50";

        let mut query = Query::new(query);
        query.set_page_size(50);

        let mut response = session
            .get_session()
            .query_iter(query, (streamer,))
            .await?
            .into_typed::<Message>();

        let mut result = Vec::new();

        while let Some(next_row_res) = response.next().await {
            let row = next_row_res?;
            result.push(row.clone());
        }

        Ok(result)
    }
    pub fn from_twitch(message: PrivmsgMessage) -> Self {
        let mut chatter_badges: Set<Text> = Set::new();
        message
            .badges
            .iter()
            .map(|badge| chatter_badges.insert(badge.name.to_string()))
            .for_each(drop);

        let message_id = message.message_id;
        let streamer_id = message.channel_login.to_string();
        let chatter_id = Some(message.sender.id);
        let chatter_username = Some(message.sender.login);
        let chatter_color = if let Some(color) = message.name_color {
            Some(color.to_string())
        } else {
            Some("#FFFFFF".to_string())
        };
        let sent_at = Some(chrono::Utc::now());
        let message_id = Some(Uuid::from_str(message_id.as_str()).unwrap());
        let content = Some(message.message_text);

        Self {
            message_id,
            streamer_id,
            chatter_id,
            content,
            chatter_username,
            chatter_badges: Some(chatter_badges),
            chatter_color,
            sent_at,
        }
    }
}

#[charybdis_view_model(
    table_name=messages_by_user,
    base_table=messages,
    partition_keys=[chatter_username],
    clustering_keys = [sent_at, streamer_id],
    table_options = r#"
          CLUSTERING ORDER BY (sent_at DESC, streamer_id ASC)
      "#
)]
pub struct MessagesByUser {
    pub message_id: Uuid,
    pub streamer_id: Text,
    pub chatter_id: Text,
    pub chatter_username: Text,
    pub chatter_badges: Option<Set<Text>>,
    pub chatter_color: Option<Text>,
    pub sent_at: Timestamp,
}
