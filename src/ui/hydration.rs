use crate::models::channels::ConnectedUsersToChannel;
use crate::models::message::Message;
use crate::models::streams_metrics::StreamersEventsLeaderboard;
use crate::ui::App;
use charybdis::operations::Find;
use scylla::transport::PagingState;
use scylla::CachingSession;
use std::sync::{Arc, Mutex};
use crate::models::metrics::StreamLeaderboard;

pub async fn fetch_data(app: Arc<Mutex<App>>, session: Arc<CachingSession>) -> anyhow::Result<()> {
    let channel = {
        let app = app.lock().unwrap();
        app.active_channels[app.active_channel].clone()
    };

    let paging_state = PagingState::start();
    let (mut connected_users, _) =
        ConnectedUsersToChannel::find_by_partition_key_value_paged((channel.streamer_id.clone(),))
            .page_size(30)
            .paging_state(paging_state)
            .execute(&session)
            .await
            .unwrap();

    let connected_users: Vec<ConnectedUsersToChannel> =
        connected_users.try_collect().unwrap_or_default();

    let streamer_ranking = StreamLeaderboard::get_leaderboard(&session, channel.streamer_id.clone()).await?;
    let chat_messages = Message::get_leaderboard(&session, channel.streamer_id.clone()).await?;
    let active_channels = StreamersEventsLeaderboard::get_leaderboard(&session).await?;

    {
        let mut app = app.lock().unwrap();
        app.connected_users = connected_users;
        app.chat_messages = chat_messages;
        if app.active_channels.len() != active_channels.len() {
            app.active_channel = 0;
        }
        
        app.rankings = streamer_ranking;
        app.active_channels = active_channels;
    };

    Ok(())
}
