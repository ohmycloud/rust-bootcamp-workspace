use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use http::{HeaderMap, StatusCode, header::LOCATION};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const LISTEN_ADDRESS: &str = "0.0.0.0:9876";

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let listener = TcpListener::bind(LISTEN_ADDRESS).await?;
    info!("Listening on {}", LISTEN_ADDRESS);
    let app_state =
        AppState::try_new("postgres://postgres:password@127.0.0.1:5432/shortener").await?;

    let app = Router::new()
        .route("/", post(shorten))
        .route("/{id}", get(redirect))
        .with_state(app_state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[axum::debug_handler]
async fn shorten(
    State(app_state): State<AppState>,
    Json(request): Json<ShortenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = app_state
        .shorten(&request.url)
        .await
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let body = Json(ShortenResponse {
        url: format!("http://{}/{}", LISTEN_ADDRESS, id),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = app_state
        .get_url(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::FOUND, headers))
}

#[derive(Debug, Deserialize)]
struct ShortenRequest {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShortenResponse {
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(url).await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )
        "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    async fn shorten(&self, url: &str) -> Result<String, sqlx::Error> {
        let nano_id = nanoid::nanoid!(6);
        let record: UrlRecord = sqlx::query_as(
            "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT DO UPDATE SET url=EXCLUDED.url RETURNING id",
        )
        .bind(&nano_id)
        .bind(url)
        .fetch_one(&self.pool)
        .await?;

        Ok(record.id)
    }

    async fn get_url(&self, id: &str) -> Result<String, sqlx::Error> {
        let record: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(record.url)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}
