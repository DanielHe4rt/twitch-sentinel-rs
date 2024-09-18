use crate::models::message::Message;
use crate::models::metrics::{StreamMessageCountByUser, StreamMessagesCounterByUser};
use charybdis::operations::{Find, Insert};
use scylla::CachingSession;
use std::sync::Arc;
use twitch_irc::message::PrivmsgMessage;

pub async fn handle_message(session: Arc<CachingSession>, payload: PrivmsgMessage) {
    // Store Message
    let message = Message::from_twitch(payload.clone());
    message.insert().execute(&session).await.unwrap();

    // Prepare and Increment 'Count' Type
    let counter = StreamMessagesCounterByUser::from_twitch(payload.clone());
    counter.increment_messages_count(1).execute(&session).await.unwrap();
    let counter = counter.find_by_primary_key().execute(&session).await.unwrap();

    let count =  StreamMessageCountByUser::from_twitch_and_counter(payload.clone(), counter.messages_count);
    count.insert().execute(&session).await.unwrap();
}