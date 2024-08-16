use std::time::Duration;

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    Extension,
};
use axum_extra::{headers, TypedHeader};
use chat_core::utils::UserCliams;
use futures::Stream;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::info;

use crate::{notify_event::NotifyEvent, NotifyState};

pub(crate) async fn sse_handler(
    Extension(user): Extension<UserCliams>,
    State(state): State<NotifyState>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    info!("user {} connected: {}", user.uid, user_agent);
    let users = state.users.clone();

    let rx = if let Some(tx) = users.get(&user.uid) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(10);
        users.insert(user.uid, tx);
        rx
    };

    let stream = BroadcastStream::new(rx).filter_map(Result::ok).map(|ev| {
        let name = match &ev {
            NotifyEvent::NewChat(_) => "NewChat",
            NotifyEvent::AddToChat(_) => "AddToChat",
            NotifyEvent::NewMessage(_) => "NewMessage",
            NotifyEvent::RemoveFromChat(_) => "RemoveFromChat",
        };
        Event::default().event(name).json_data(ev)
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
}
