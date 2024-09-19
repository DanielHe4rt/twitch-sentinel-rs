use crate::models::channels::ConnectedUsersToChannel;
use crate::models::message::Message;
use crate::models::streams_metrics::StreamersEventsLeaderboard;
use crate::ui::App;
use charybdis::operations::Find;
use charybdis::types::Date;
use chrono::Utc;
use scylla::transport::PagingState;
use scylla::CachingSession;
use std::sync::{Arc, Mutex};

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
            .await.unwrap();
    
    let connected_users: Vec<ConnectedUsersToChannel> = connected_users.try_collect().unwrap_or_default();

    let paging_state = PagingState::start();
    let chat_messages = Message::find_by_partition_key_value((channel.streamer_id.clone(),))
        .page_size(30)
        .paging_state(paging_state)
        .execute(&session)
        .await?
        .try_collect()
        .await
        .unwrap_or_default();
        
    
    

    let paging_state = PagingState::start();
    let (mut active_channels, a) = StreamersEventsLeaderboard::find_by_partition_key_value_paged((
        Date::from(Utc::now().date_naive()),
    ))
    .paging_state(paging_state)
    .page_size(30)
    .execute(&session)
    .await?;

    let mut active_channels: Vec<StreamersEventsLeaderboard> = active_channels.try_collect().unwrap();

    active_channels.push(StreamersEventsLeaderboard {
        streamer_id: "danielhe4rt".to_string(),
        day: Utc::now().date_naive(),
        events_count: 0,
    });

    _ = {
        let mut app = app.lock().unwrap();
        app.connected_users = connected_users;
        app.chat_messages = chat_messages;
        if app.active_channels.len() != active_channels.len() {
            app.active_channel = 0;
        }
        app.active_channels = active_channels;
    };

    Ok(())
}
