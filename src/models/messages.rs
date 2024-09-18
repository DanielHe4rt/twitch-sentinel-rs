use charybdis::macros::charybdis_model;
use charybdis::types::{Set, Text, Timestamp, Uuid};

#[derive(Clone, Default)]
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
    pub chatter_username: Text,
    pub chatter_badges: Option<Set<Text>>,
    pub chatter_color: Option<Text>,
    pub sent_at: Timestamp,
}