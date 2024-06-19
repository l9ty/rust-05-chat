mod config;
mod error;
mod handlers;
mod models;
mod utils;

use std::{ops::Deref, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
pub use config::AppConfig;
use error::AppError;
use handlers::*;
use sqlx::PgPool;
use utils::{JwtDecodingKey, JwtEncodingKey};

#[derive(Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: JwtDecodingKey,
    pub(crate) ek: JwtEncodingKey,
    pub(crate) db: PgPool,
}

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::try_new(config).await?;
    let root = Router::new()
        .route("/", get(index_handler))
        .route("/signup", post(singup_handler))
        .route("/signin", post(signin_handler))
        .with_state(state);
    Ok(root)
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> anyhow::Result<Self, AppError> {
        let dk = JwtDecodingKey::load(config.auth.pk.as_bytes())?;
        let ek = JwtEncodingKey::load(config.auth.sk.as_bytes())?;
        let pool = PgPool::connect(&config.server.db_url).await?;
        let inner = Arc::new(AppStateInner {
            config,
            dk,
            ek,
            db: pool,
        });
        Ok(Self { inner })
    }
}
