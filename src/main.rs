//! Run with
//!
//! ```not_rust
//! cargo run
//! kill or ctrl-c
//! ```

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    routing::{get, post, put},
    Router,
};
use db::SmplDB;
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tokio::signal;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod handler;
mod utils;

#[derive(Clone)]
struct AppState {
    smpldb: Arc<SmplDB>,
}

#[tokio::main]
async fn main() {
    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let governor = GovernorConfigBuilder::default()
        .per_second(5) // Allow 5 requests per second
        .burst_size(10) // Allow bursts of up to 10 requests
        .finish()
        .unwrap();

    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
    let state = AppState {
        smpldb: SmplDB::new(&db_url)
            .expect("Failed to initialise DB connections to DB")
            .into(),
    };
    // Create a regular axum app.
    let app = Router::new()
        .route("/sign_up", post(handler::sign_up::sign_up))
        .route("/sign_in", post(handler::sign_in::sign_in))
        .route("/profile", get(handler::profile::get_profile))
        .route("/profile", put(handler::profile::update_profile))
        .route("/wallet", get(handler::wallet::get_wallet))
        .route("/wallet", put(handler::wallet::update_wallet))
        .route(
            "/transactions/:id",
            get(handler::transaction::get_transaction_by_id),
        )
        .route(
            "/transactions",
            post(handler::transaction::create_transaction),
        )
        .route(
            "/transactions",
            get(handler::transaction::list_transactions),
        )
        .with_state(state)
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ))
        .layer(GovernorLayer {
            config: governor.into(),
        });

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Startng server: listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
