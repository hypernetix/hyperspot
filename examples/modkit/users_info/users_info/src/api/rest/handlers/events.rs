use axum::response::{IntoResponse, Response};

use super::{SseBroadcaster, UserEvent, info};

pub(super) fn users_events(sse: &SseBroadcaster<UserEvent>) -> Response {
    info!("New SSE connection for user events");
    sse.sse_response_named("users_events").into_response()
}
