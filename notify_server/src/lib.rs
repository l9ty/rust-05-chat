mod config;
mod notify_event;
mod sse;

use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{
    middlewares::{verify_token, VerifyToken},
    utils::{JwtDecodingKey, UserCliams},
    RowID,
};
use config::NotifyConfig;
use dashmap::DashMap;
use notify_event::{setup_pg_listener, NotifyEvent};
use sse::sse_handler;
use tokio::{net::TcpListener, sync::broadcast};
use tracing::info;

#[derive(Clone)]
pub struct NotifyState {
    inner: Arc<NotifyStateInner>,
}

pub type UserMap = Arc<DashMap<RowID, broadcast::Sender<NotifyEvent>>>;

pub struct NotifyStateInner {
    config: NotifyConfig,
    users: UserMap,
    dk: JwtDecodingKey,
}

pub async fn run_server() -> anyhow::Result<()> {
    let config = NotifyConfig::load().context("read notify config")?;
    let addr = format!("{}:{}", &config.server.host, config.server.port);
    info!("listening on {}", addr);

    let state = NotifyState::try_new(config)?;

    setup_pg_listener(state.clone())
        .await
        .context("setup pg listener")?;

    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(
            state.clone(),
            verify_token::<NotifyState>,
        ))
        .route("/", get(index_handler))
        .with_state(state.clone());
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

const INDEX_HTML: &str = include_str!("../index.html");
async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl Deref for NotifyState {
    type Target = NotifyStateInner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NotifyState {
    pub fn try_new(config: NotifyConfig) -> anyhow::Result<Self> {
        let users = Arc::new(DashMap::new());
        let dk = JwtDecodingKey::load(config.auth.pk.as_bytes())?;
        Ok(Self {
            inner: Arc::new(NotifyStateInner { config, users, dk }),
        })
    }
}

impl VerifyToken for NotifyState {
    type Error = anyhow::Error;

    fn verify(&self, token: &str) -> Result<UserCliams, Self::Error> {
        self.dk.verify(token)
    }
}
