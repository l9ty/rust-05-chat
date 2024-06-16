mod config;
mod error;
mod handlers;

use std::{ops::Deref, sync::Arc};

use axum::{routing, Router};
pub use config::AppConfig;
use handlers::index_handler;

#[derive(Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    // pub(crate) dk: DecodingKey,
    // pub(crate) ek: EncodingKey,
    // pub(crate) db: PgPool,
}

pub fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::try_new(config)?;
    let root = Router::new().route("/", routing::get(index_handler).with_state(state));
    Ok(root)
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn try_new(config: AppConfig) -> anyhow::Result<Self> {
        let inner = Arc::new(AppStateInner { config });
        Ok(Self { inner })
    }
}
