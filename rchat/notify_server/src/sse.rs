use axum::response::Sse;
use axum::response::sse::Event;
use axum_extra::{TypedHeader, headers};
use futures::{Stream, stream};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;

#[axum::debug_handler]
pub(crate) async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every seconds
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
