use crate::models::channels::{ConnectedUsersToChannel, ConnectedUsersToChannelHistory};
use charybdis::operations::{Delete, Insert};
use charybdis::types::Timestamp;
use chrono::Utc;
use scylla::CachingSession;
use std::fmt::Display;
use std::sync::Arc;

pub enum TransitState {
    JOIN,
    LEAVE,
}

impl Display for TransitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitState::JOIN => write!(f, "JOIN"),
            TransitState::LEAVE => write!(f, "LEAVE"),
        }
    }
}

pub async fn handle_transit(
    state: TransitState,
    session: Arc<CachingSession>,
    streamer_id: String,
    chatter_id: String,
)
{
    let connection = ConnectedUsersToChannel {
        chatter_id: Some(chatter_id.clone()),
        streamer_id: streamer_id.clone(),
        joined_at: Some(Timestamp::from(Utc::now())),
    };

    match state {
        TransitState::JOIN => {
            connection.insert().execute(&session).await.unwrap();
        }
        TransitState::LEAVE => {
            connection.delete().execute(&session).await.unwrap();
        }
    };

    let connection_history = ConnectedUsersToChannelHistory {
        chatter_id,
        streamer_id,
        event: state.to_string(),
        created_at: Timestamp::from(Utc::now()),
    };

    connection_history.insert().execute(&session).await.unwrap();
}