use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Request, State},
    routing::get,
};
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    trace::{RandomIdGenerator, Sampler, Tracer},
};
use serde::{Deserialize, Serialize};
use tokio::{
    join,
    net::TcpListener,
    time::{Instant, sleep},
};
use tracing::{info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    Layer,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct User {
    name: String,
    age: u8,
    skills: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily("/tmp/axum_logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let log_layer = fmt::Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);
    let file_layer = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    let tracer = init_tracer()?;
    let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(log_layer)
        .with(file_layer)
        .with(opentelemetry_layer)
        .init();

    let user = User {
        name: "Larry Wall".to_string(),
        age: 30,
        skills: vec!["Raku".to_string(), "Perl 6".to_string()],
    };
    let user = Arc::new(Mutex::new(user));

    let addr = "0.0.0.0:8080";
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/user", get(get_user))
        .with_state(user);
    let listener = TcpListener::bind(addr).await?;
    info!("Starting server on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[instrument(fields(http.uri = req.uri().path(), http.method = req.method().as_str()))]
async fn index_handler(req: Request) -> &'static str {
    sleep(Duration::from_millis(11)).await;
    let ret = long_task().await;
    info!(http.status = 200, "index handler completed");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    sleep(Duration::from_millis(100)).await;
    let t1 = task1();
    let t2 = task2();
    let t3 = task3();
    let _ = join!(t1, t2, t3);
    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, world!"
}

fn init_tracer() -> anyhow::Result<Tracer> {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .with_timeout(Duration::from_secs(3))
        .build()?;

    // Create a tracer provider with the exporter
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(
            Resource::builder()
                .with_service_name("axum.service")
                .build(),
        )
        .build();

    let tracer = provider.tracer("my_service");

    Ok(tracer)
}

#[instrument]
async fn task1() {
    sleep(Duration::from_millis(10)).await;
}

#[instrument]
async fn task2() {
    sleep(Duration::from_millis(50)).await;
}

#[instrument]
async fn task3() {
    sleep(Duration::from_millis(300)).await;
}

#[axum::debug_handler]
#[instrument]
async fn get_user(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    (*user.lock().unwrap()).clone().into()
}
