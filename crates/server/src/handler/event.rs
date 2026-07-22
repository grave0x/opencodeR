use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
};
use futures::stream::{self, Stream, StreamExt};
use std::convert::Infallible;
use crate::SharedState;

pub async fn subscribe(
    State(_state): State<SharedState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Connected event as JSON data
    let connected = Event::default()
        .event("message")
        .json_data(serde_json::json!({
            "id": opencode_r_schema::identifier::ascending(),
            "type": "server.connected",
            "data": {}
        }))
        .unwrap();

    // Start with connected event, follow with heartbeats
    let init = stream::once(async { Ok(connected) });
    let heartbeats = stream::repeat_with(|| {
        Ok(Event::default().data(": heartbeat\n\n"))
    });

    let stream = init.chain(heartbeats);
    Sse::new(stream).keep_alive(KeepAlive::new())
}
