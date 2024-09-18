use axum::routing::get;
use axum::Router;
use prometheus::TextEncoder;

pub fn prometheus_api() {
    tracing_subscriber::fmt::init();

    let web_app = Router::new().route("/metrics", get(get_metrics));
    // run it

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
            .await
            .unwrap();
        println!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, web_app).await.unwrap();
    });
}

// Web request handler for GET /metrics
pub async fn get_metrics() -> String {
    // Export all metrics from the global registry from the prometheus crate
    TextEncoder.encode_to_string(&prometheus::gather()).unwrap()
}