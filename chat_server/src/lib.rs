mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use std::{ops::Deref, sync::Arc};

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
pub use config::AppConfig;
use handlers::*;
use middlewares::verify_token;
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

    let api = Router::new()
        .route("/users", get(list_ws_users_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .post(send_message_handler)
                .delete(delete_chat_handler),
        )
        .route("/chat/:id/message", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_token));

    let root = Router::new()
        .route("/signup", post(singup_handler))
        .route("/signin", post(signin_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(middlewares::set_global_layer(root))
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> anyhow::Result<Self> {
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

#[cfg(test)]
impl AppState {
    fn new_for_test(pool: PgPool) -> AppState {
        let pk = include_str!("../fixtures/private.pem");
        let sk = include_str!("../fixtures/public.pem");
        let ek = JwtEncodingKey::load(pk.as_bytes()).unwrap();
        let dk = JwtDecodingKey::load(sk.as_bytes()).unwrap();

        AppState {
            inner: Arc::new(AppStateInner {
                config: AppConfig {
                    server: config::ServerConfig {
                        db_url: "".to_string(),
                        host: "0.0.0.0".to_string(),
                        port: 8080,
                    },
                    auth: config::AuthConfig {
                        sk: sk.to_string(),
                        pk: pk.to_string(),
                    },
                },
                dk,
                ek,
                db: pool,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    pub(crate) static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
}
