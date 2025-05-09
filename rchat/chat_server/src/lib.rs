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
use axum::handler::Handler;
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

#[cfg(test)]
impl AppState {
    pub async fn new_for_test(config: AppConfig) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx_db_tester::TestPg;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pos = config.server.db_url.rfind('/').expect("invalid db_url");
        let server_url = &config.server.db_url[..pos];

        let tdb = TestPg::new(server_url.to_string(), std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        Ok((
            tdb, Self {
                inner: Arc::new(AppStateInner { config, dk, ek, pool }),
            }
        ))
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
        .route("/chat/:id/messages", get(list_message_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layer(app))
}

#[cfg(feature = "test-util")]
mod test_util {
    use super::*;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:possword@localhost:5432".to_string(),
        };
        let tdb = TestPg::new(url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test data
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }

    impl AppState {
        pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
            let config = AppConfig::load()?;
            let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
            let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
            let post = config.server.db_url.rfind("/").expect("invalid db_url");
            let server_url = &config.server.db_url[..post];
            let (tdb, pool) = get_test_pool(Some(server_url)).await;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    pool,
                })
            };
            Ok((tdb, state))
        }
    }
}
