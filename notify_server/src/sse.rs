use axum::response::{
    sse::{Event, KeepAlive},
    Sse,
};
use futures::{stream, Stream};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

pub(crate) async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        KeepAlive::default()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

// pub(crate) async fn sse_handler(
//     Extension(User): Extension<UserCliams>,
//     State(state): State<AppState>,
//     TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
// ) {
//     todo!()
// }
