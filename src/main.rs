#![feature(iterator_try_collect)]

use crate::args::{Cli, Commands};
use crate::models::channels::Channel;
use crate::stressor::payload::STREAMERS;
use crate::ui::start_terminal;
use charybdis::operations::Insert;
use clap::Parser;
use scylla::{CachingSession, SessionBuilder};
use std::sync::Arc;
use twitch::start_twitch_workload;

mod args;
mod models;
mod stressor;
mod twitch;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    

    let session = SessionBuilder::new()
        .known_node("127.0.0.1:9042")
        .build()
        .await?;

    session.use_keyspace("twitch", true).await?;

    let session = Arc::new(CachingSession::from(session, 30));
    let twitch_session = Arc::clone(&session);
    let terminal_session = Arc::clone(&session);

    match args.command {
        Commands::Twitch => {
            tokio::spawn(async {
                start_twitch_workload(twitch_session).await.unwrap();
            });
        }
        Commands::Stress { stress } => {
            for i in 0..stress.workers {
                let stressor_session = Arc::clone(&session);
                tokio::spawn(async move {
                    stressor::handle_stressor(i, Arc::clone(&stressor_session))
                        .await
                        .unwrap();
                });
            }
        }
    }

    load_channels(Arc::clone(&session)).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    start_terminal(terminal_session).await.unwrap();

    Ok(())
}

async fn load_channels(session: Arc<CachingSession>) -> anyhow::Result<()> {
    for streamer in STREAMERS {
        let channel = Channel {
            streamer_id: streamer.to_string(),
            streamer_username: None,
            last_game_info: None,
            profile_image_url: None,
            stream_tags: None,
            created_at: None,
        };

        channel.insert().execute(&session).await?;
    }

    Ok(())
}
