use charybdis::macros::charybdis_model;
use charybdis::types::{Text, Timestamp};

#[derive(Clone)]
#[charybdis_model(
    table_name = channels,
    partition_keys = [streamer_id],
    clustering_keys = [],
)]
pub struct Channel {
    pub streamer_id: Text,
    pub streamer_username: Option<Text>,
    pub last_game_info: Option<Text>,
    pub profile_image_url: Option<Text>,
    pub stream_tags: Option<Text>,
    pub created_at: Option<Text>,
}


#[charybdis_model(
    table_name = connected_users_to_channel,
    partition_keys = [streamer_id],
    clustering_keys = [chatter_id],
)]
pub struct ConnectedUsersToChannel {
    pub streamer_id: Text,
    pub chatter_id: Text,
    pub joined_at: Timestamp,
}

#[charybdis_model(
    table_name = channel_connected_users_history,
    partition_keys = [streamer_id],
    clustering_keys = [chatter_id, event, created_at],
)]
pub struct ConnectedUsersToChannelHistory {
    pub streamer_id: Text,
    pub chatter_id: Text,
    pub event: Text,
    pub created_at: Timestamp,
}

