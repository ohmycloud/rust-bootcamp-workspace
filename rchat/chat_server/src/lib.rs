#[allow(dead_code)]
mod config;
mod error;
mod handlers;
mod models;
mod utils;

use std::fmt;
use axum::routing::{get, patch, post};
use axum::Router;
pub use config::AppConfig;
pub use error::AppError;
use handlers::*;
pub use models::User;
use std::ops::Deref;
use std::sync::Arc;
use anyhow::Context;
use sqlx::PgPool;
use crate::utils::{DecodingKey, EncodingKey};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
#[derive(Clone)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: PgPool,
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pool = PgPool::connect(&config.server.db_url).await.context("create pool failed")?;

        Ok(Self {
            inner: Arc::new(AppStateInner { config, dk, ek, pool }),
        })
    }
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(signup_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/message", get(list_message_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(app)
}
