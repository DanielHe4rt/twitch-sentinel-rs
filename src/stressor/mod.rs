use crate::models::streams_metrics::handle_event;
use crate::stressor::payload::{MESSAGES, STREAMERS, USERS};
use crate::twitch::actions::channel_transit::{handle_transit, TransitState};
use crate::twitch::actions::new_message::handle_message;
use log::trace;
use rand::random;
use scylla::CachingSession;
use std::sync::Arc;
use twitch_irc::message::{IRCMessage, PrivmsgMessage, TwitchUserBasics};
use uuid::Uuid;

pub mod payload;

enum EventTypes {
    Join,
    Part,
    Privmsg,
}

impl EventTypes {
    pub fn random() -> Self {
        match rand::random::<u8>() % 3 {
            0 => Self::Join,
            1 => Self::Part,
            _ => Self::Privmsg,
        }
    }
}

pub async fn handle_stressor(instance: usize, session: Arc<CachingSession>) -> anyhow::Result<()> {
    println!("Handling stressor {}", instance);

    let online_streamers = STREAMERS.clone();
    let chatters = USERS;
    let possible_messages = MESSAGES;

    loop {
        let streamer = online_streamers[random::<usize>() % online_streamers.len()].to_string();
        let chatter = chatters[random::<usize>() % chatters.len()].to_string();
        let message = possible_messages[random::<usize>() % possible_messages.len()].to_string();

        let message = generate_payload(streamer.clone(), chatter.clone(), message.clone());

        match EventTypes::random() {
            EventTypes::Join => {
                trace!("{} joined at {} channel", chatter, streamer);
                handle_transit(
                    TransitState::JOIN,
                    Arc::clone(&session),
                    streamer.to_string(),
                    chatter.to_string(),
                )
                .await;
                handle_event(streamer.to_string(), Arc::clone(&session)).await;
            }
            EventTypes::Part => {
                trace!("{} left {}", chatter, streamer);
                handle_transit(
                    TransitState::LEAVE,
                    Arc::clone(&session),
                    streamer.to_string(),
                    chatter.to_string(),
                )
                .await;
                handle_event(streamer.to_string(), Arc::clone(&session)).await;
            }
            EventTypes::Privmsg => {
                handle_event(streamer.to_string(), Arc::clone(&session)).await;
                handle_message(Arc::clone(&session), message).await;
            },
        };
    }
    
}

fn generate_payload(
    streamer: String,
    chatter: String,
    message: String,
) -> PrivmsgMessage {

   PrivmsgMessage {
        channel_login: streamer.clone(),
        channel_id: streamer,
        message_text: message, // Example message
        is_action: false,
        sender: TwitchUserBasics {
            id: chatter.to_string(),
            login: chatter.to_string(),
            name: chatter.to_string(),
        },
        badge_info: vec![],     // Fake badge info (subscriber for 6 months)
        badges: vec![],         // Example badge (broadcaster)
        bits: None,             // Example of bits donated
        name_color: None,       // Example color (dodger blue)
        emotes: vec![],         // Example emote
        message_id: Uuid::new_v4().to_string(), // Generates a new random UUID
        server_timestamp: Default::default(), // Use default timestamp value
        source: IRCMessage {
            tags: Default::default(),
            prefix: None,
            command: "".to_string(),
            params: vec![],
        }, // Placeholder for IRC message
    }
}
