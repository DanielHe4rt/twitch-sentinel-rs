use charybdis::macros::{charybdis_model, charybdis_view_model};
use charybdis::operations::{Find, Insert};
use charybdis::types::{Counter, Date, Int, Text};
use chrono::Utc;
use scylla::CachingSession;
use std::sync::Arc;

#[charybdis_model(
    table_name=streamers_events_counter,
    partition_keys=[day, streamer_id],
    clustering_keys = [],
)]
pub struct StreamersEventsCounter {
    pub day: Date,
    pub streamer_id: Text,
    pub events_count: Counter,
}

#[charybdis_model(
    table_name=streamers_events_count,
    partition_keys=[day, streamer_id],
    clustering_keys = [],
)]
pub struct StreamersEventsCount {
    pub day: Date,
    pub streamer_id: Text,
    pub events_count: Int,
}

#[derive(Clone)]
#[charybdis_view_model(
    table_name=streamers_events_leaderboard,
    base_table=streamers_events_count,
    partition_keys=[day],
    clustering_keys = [events_count, streamer_id],
    table_options = r#"
      CLUSTERING ORDER BY (events_count DESC, streamer_id ASC)
    "#
)]
pub struct StreamersEventsLeaderboard {
    pub streamer_id: Text,
    pub day: Date,
    pub events_count: Int,
}

pub async fn handle_event(streamer_id: String, session: Arc<CachingSession>) {
    // Prepare and Increment 'Count' Type
    let counter = StreamersEventsCounter {
        day: Utc::now().date_naive(),
        streamer_id: streamer_id.clone(),
        events_count: Default::default(),
    };
    counter
        .increment_events_count(1)
        .execute(&session)
        .await
        .unwrap();
    let counter = counter
        .find_by_primary_key()
        .page_size(1)
        .execute(&session)
        .await
        .unwrap();

    let count = StreamersEventsCount {
        day: Utc::now().date_naive(),
        streamer_id,
        events_count: counter.events_count.0 as i32,
    };
    count.insert().execute(&session).await.unwrap();
}
