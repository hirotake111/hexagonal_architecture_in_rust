use std::{env, sync::Arc};

use anyhow::Context;
use axum::routing::post;
use dotenvy::dotenv;
use hexagonal_architecture_in_rust::{
    config, handler,
    repository::Postgres,
    service::{EmailClient, Prometheus, Service},
    state::AppState,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let config = config::get_config()?;
    tracing_subscriber::fmt::init();
    let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
        |req: &axum::extract::Request| {
            let uri = req.uri().to_string();
            tracing::info_span!("http_request", method = ?req.method(), uri)
        },
    );

    let database_url =
        env::var("DATABASE_URL").expect("Expected environment variable DATABASE_URL");
    let postgres = Postgres::new(&database_url).await?;
    let metrics = Prometheus {};
    let email_client = EmailClient {};
    let author_service = Service::new(postgres, metrics, email_client);
    let app_state = AppState {
        author_service: Arc::new(author_service),
    };
    let router = axum::Router::new()
        .route("/authors", post(handler::crate_author))
        .layer(trace_layer)
        .with_state(app_state);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", &config.port))
        .await
        .with_context(|| format!("failed to listen on port {}", &config.port))?;
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .await
        .context("received error from running server")?;

    Ok(())
}
