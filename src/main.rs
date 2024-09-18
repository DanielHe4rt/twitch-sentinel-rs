use crate::ui::start_terminal;

mod ui;
mod twitch;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::spawn(async {
        twitch::start_twitch_workload().await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    start_terminal().await.unwrap();

    Ok(())
}
