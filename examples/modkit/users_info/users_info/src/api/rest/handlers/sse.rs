use axum::response::{IntoResponse, Response};

use super::{info, SseBroadcaster, UserEvent};

pub(super) fn users_events(sse: &SseBroadcaster<UserEvent>) -> Response {
    info!("New SSE connection for user events");
    sse.sse_response_named("users_events").into_response()
}
