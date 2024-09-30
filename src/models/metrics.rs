use charybdis::macros::{charybdis_model, charybdis_view_model};
use charybdis::types::{Counter, Int, Text};
use futures::StreamExt;
use scylla::CachingSession;
use scylla::query::Query;
use twitch_irc::message::PrivmsgMessage;
use crate::models::message::Message;

#[charybdis_model(
    table_name=stream_messages_counter_by_user,
    partition_keys=[streamer_id, chatter_id],
    clustering_keys = [],
)]
pub struct StreamMessagesCounterByUser {
    pub streamer_id: Text,
    pub chatter_id: Text,
    pub messages_count: Option<Counter>,
}

impl StreamMessagesCounterByUser {
    pub fn from_twitch(message: PrivmsgMessage) -> Self {
        let streamer_id = message.channel_login.to_string();
        let chatter_id = message.sender.id;
        Self {
            streamer_id,
            chatter_id,
            messages_count: None,
        }
    }
}

#[charybdis_model(
    table_name=stream_messages_count_by_user,
    partition_keys=[streamer_id],
    clustering_keys = [chatter_username],
)]
pub struct StreamMessageCountByUser {
    pub streamer_id: Text,
    pub chatter_id: Text,
    pub chatter_username: Text,
    pub messages_count: Int,
}

impl StreamMessageCountByUser {
    pub fn from_twitch_and_counter(message: PrivmsgMessage, counter: Counter) -> Self {
        let streamer_id = message.channel_login.to_string();
        let chatter_id = message.sender.id;
        let chatter_username = message.sender.login;
        let messages_count = Int::from(counter.0 as i32) + 1;
        Self {
            streamer_id,
            chatter_id,
            chatter_username,
            messages_count,
        }
    }
}


#[derive(Default, Clone)]
#[charybdis_view_model(
    table_name=stream_leaderboard,
    base_table=stream_messages_count_by_user,
    partition_keys=[streamer_id],
    clustering_keys = [messages_count, chatter_username],
    table_options = r#"
      CLUSTERING ORDER BY (messages_count DESC, chatter_username ASC)
    "#
)]
pub struct StreamLeaderboard {
    pub streamer_id: Text,
    pub chatter_id: Text,
    pub chatter_username: Text,
    pub messages_count: Int,
}

impl StreamLeaderboard {
    pub async fn get_leaderboard(
        session: &CachingSession,
        streamer: String,
    ) -> anyhow::Result<(Vec<StreamLeaderboard>)> {
        let query = "SELECT streamer_id, chatter_id, chatter_username, messages_count FROM stream_leaderboard WHERE streamer_id = ? LIMIT 50";

        let mut query = Query::new(query);
        query.set_page_size(50);

        let mut response = session
            .get_session()
            .query_iter(query, (streamer,))
            .await?
            .into_typed::<StreamLeaderboard>();

        let mut result = Vec::new();

        while let Some(next_row_res) = response.next().await {
            let row = next_row_res?;
            result.push(row.clone());
        }

        Ok(result)
    }
}