#![feature(iterator_try_collect)]

use std::sync::Arc;
use scylla::{CachingSession, SessionBuilder};
use twitch::start_twitch_workload;
use crate::ui::start_terminal;

mod ui;
mod twitch;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let session = SessionBuilder::new()
        .known_node("127.0.0.1:9042")
        .build()
        .await?;

    session.use_keyspace("twitch", true).await?;

    let session = Arc::new(CachingSession::from(session, 20));
    let twitch_session = Arc::clone(&session);
    let terminal_session = Arc::clone(&session);



    tokio::spawn(async {
        start_twitch_workload(twitch_session).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    start_terminal(terminal_session).await.unwrap();

    Ok(())
}
