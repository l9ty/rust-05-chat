use anyhow::Context;
use chat_server::{get_router, AppConfig, AppState};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt as _, Layer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let addr = format!("0.0.0.0:{}", config.server.port);
    let listener = TcpListener::bind(&addr).await?;

    let console = tracing_subscriber::fmt::Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(console).init();

    let state = AppState::try_new(config)
        .await
        .context("new state failed")?;
    let router = get_router(state).await?;
    info!("listening on {}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
