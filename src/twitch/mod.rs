use std::collections::HashMap;
use std::sync::Arc;
use log::trace;
use scylla::{CachingSession, Session, SessionBuilder};
use twitch_irc::{irc, ClientConfig, MetricsConfig, SecureTCPTransport, TwitchIRCClient};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use crate::models::streams_metrics::handle_event;
use crate::twitch::actions::channel_transit::{handle_transit, TransitState};
use crate::twitch::actions::new_message::handle_message;
use crate::twitch::server::prometheus_api;

mod server;
mod actions;

pub async fn start_twitch_workload(session: Arc<CachingSession>) -> anyhow::Result<()> {
    
    prometheus_api();
    let config = ClientConfig {
        // Enable metrics collection.
        metrics_config: MetricsConfig::Enabled {
            constant_labels: {
                let mut labels = HashMap::new();
                labels.insert("app".to_owned(), "metrics-example".to_owned());
                labels.insert("version".to_owned(), env!("CARGO_PKG_VERSION").to_owned());
                labels
            },
            metrics_registry: None,
        },
        // rest of the config is default
        ..ClientConfig::default()
    };
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);


    let handler_db = Arc::clone(&session);
    let message_handler = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Join(payload) => {
                    trace!("{} joined {}", payload.user_login, payload.channel_login);
                    handle_transit(TransitState::JOIN, Arc::clone(&handler_db), payload.channel_login.clone(), payload.user_login.clone()).await;
                    handle_event(payload.channel_login.clone(), Arc::clone(&handler_db)).await;
                }
                ServerMessage::Part(payload) => {
                    trace!("{} left {}", payload.user_login, payload.channel_login);
                    handle_transit(TransitState::LEAVE, Arc::clone(&handler_db), payload.channel_login.clone(), payload.user_login.clone()).await;
                    handle_event(payload.channel_login.clone(), Arc::clone(&handler_db)).await;
                }
                ServerMessage::Privmsg(message) => {
                    handle_event(message.channel_login.clone(), Arc::clone(&handler_db)).await;
                    handle_message(Arc::clone(&handler_db), message).await;
                }
                _ => {}
            }
        }
    });

    client.send_message(irc!["CAP", "REQ", "twitch.tv/membership"]).await?;
    client.join("danielhe4rt".to_owned()).unwrap();


    let result = session.get_session().query_unpaged("SELECT streamer_username FROM twitch.channels", []).await?;
    let mut result = result.rows_typed::<(String,)>()?;
    while let Some(row) = result.next().transpose()? {
        client.join(row.0.to_owned())?;
    }


    message_handler.await?;
    Ok(())
}