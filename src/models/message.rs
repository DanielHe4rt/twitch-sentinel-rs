use charybdis::macros::{charybdis_model, charybdis_view_model};
use charybdis::types::{Set, Text, Timestamp, Uuid};
use std::str::FromStr;
use twitch_irc::message::PrivmsgMessage;

#[derive(Clone)]
#[charybdis_model(
    table_name = messages,
    partition_keys = [streamer_id],
    clustering_keys = [sent_at, chatter_username],
    table_options = r#"
          CLUSTERING ORDER BY (sent_at DESC, chatter_username ASC)
      "#
)]
pub struct Message {
    pub message_id: Uuid,
    pub streamer_id: Text,
    pub chatter_id: Text,
    pub content: Text,
    pub chatter_username: Text,
    pub chatter_badges: Option<Set<Text>>,
    pub chatter_color: Option<Text>,
    pub sent_at: Timestamp,
}

impl Message {
    pub fn from_twitch(message: PrivmsgMessage) -> Self {
        let mut chatter_badges: Set<Text> = Set::new();
        message.badges.iter().map(|badge| chatter_badges.insert(badge.name.to_string())).for_each(drop);

        let message_id = message.message_id;
        let streamer_id = message.channel_login.to_string();
        let chatter_id = message.sender.id;
        let chatter_username = message.sender.login;
        let chatter_color = if let Some(color) = message.name_color { Some(color.to_string()) } else { Some("#FFFFFF".to_string()) };
        let sent_at = chrono::Utc::now();
        let message_id = Uuid::from_str(message_id.as_str()).unwrap();
        let content = message.message_text;

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
