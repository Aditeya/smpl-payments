//! Run with
//!
//! ```not_rust
//! cargo run
//! kill or ctrl-c
//! ```

use std::{sync::Arc, time::Duration};

use axum::{routing::{get, post, put}, Router};
use db::SmplDB;
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handler;
mod db;
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


    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
    let state = AppState {
        smpldb: SmplDB::new(&db_url).expect("Failed to initialise DB connections to DB").into()
    };
    // Create a regular axum app.
    let app = Router::new()
        .route("/sign_up", post(handler::sign_up::sign_up))
        .route("/sign_in", post(handler::sign_in::sign_in))
        .route("/profile", get(handler::profile::get_profile))
        .route("/profile", put(handler::profile::update_profile))
        // .route("/transactions", todo!())
        // .route("/transactions/:id", todo!())
        // .route("/wallet", todo!())
        .with_state(state)
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(listener, app)
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