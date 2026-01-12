use crate::api::rest::sse_adapter::SseUserEventPublisher;
use crate::api::rest::{dto, routes};
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::EventPublisher;
use futures_util::StreamExt;
use modkit::api::{OpenApiInfo, OpenApiRegistryImpl};
use modkit::SseBroadcaster;
use time::OffsetDateTime;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

#[tokio::test]
async fn openapi_has_users_sse_content() {
    // Create a mock OpenAPI registry (using api_ingress)
    let api = OpenApiRegistryImpl::default();
    let router: axum::Router<()> = axum::Router::new();
    let sse_broadcaster = SseBroadcaster::<dto::UserEvent>::new(4);

    let _router = routes::register_users_sse_route(router, &api, sse_broadcaster);

    let doc = api.build_openapi(&OpenApiInfo::default()).expect("openapi");
    let v = serde_json::to_value(&doc).expect("json");

    // UserEvent schema is materialized
    let schema = v
        .pointer("/components/schemas/UserEvent")
        .expect("UserEvent missing");
    assert!(schema.get("$ref").is_none());

    // content is text/event-stream with $ref to our schema
    // Path is /users-info/v1/users/events, JSON pointer escapes / as ~1
    let refp = v
        .pointer(
            "/paths/~1users-info~1v1~1users~1events/get/responses/200/content/text~1event-stream/schema/$ref",
        )
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    assert_eq!(refp, "#/components/schemas/UserEvent");
}

#[tokio::test]
async fn sse_broadcaster_delivers_events() {
    let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
    let mut stream = Box::pin(broadcaster.subscribe_stream());

    let test_event = dto::UserEvent {
        kind: "created".to_owned(),
        id: Uuid::new_v4(),
        at: OffsetDateTime::now_utc(),
    };

    // Send event
    broadcaster.send(test_event.clone());

    // Receive event
    let received = timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("timeout")
        .expect("event received");

    assert_eq!(received.kind, test_event.kind);
    assert_eq!(received.id, test_event.id);
    assert_eq!(received.at, test_event.at);
}

#[tokio::test]
async fn sse_adapter_publishes_domain_events() {
    let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
    let adapter = SseUserEventPublisher::new(broadcaster.clone());
    let mut stream = Box::pin(broadcaster.subscribe_stream());

    let user_id = Uuid::new_v4();
    let timestamp = OffsetDateTime::now_utc();
    let domain_event = UserDomainEvent::Created {
        id: user_id,
        at: timestamp,
    };

    // Publish domain event through adapter
    adapter.publish(&domain_event);

    // Receive converted event
    let received = timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("timeout")
        .expect("event received");

    assert_eq!(received.kind, "created");
    assert_eq!(received.id, user_id);
    assert_eq!(received.at, timestamp);
}

#[tokio::test]
async fn sse_adapter_handles_all_event_types() {
    let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
    let adapter = SseUserEventPublisher::new(broadcaster.clone());
    let mut stream = Box::pin(broadcaster.subscribe_stream());

    let user_id = Uuid::new_v4();
    let timestamp = OffsetDateTime::now_utc();

    // Test Created event
    adapter.publish(&UserDomainEvent::Created {
        id: user_id,
        at: timestamp,
    });
    let event = timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("timeout")
        .expect("event received");
    assert_eq!(event.kind, "created");

    // Test Updated event
    adapter.publish(&UserDomainEvent::Updated {
        id: user_id,
        at: timestamp,
    });
    let event = timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("timeout")
        .expect("event received");
    assert_eq!(event.kind, "updated");

    // Test Deleted event
    adapter.publish(&UserDomainEvent::Deleted {
        id: user_id,
        at: timestamp,
    });
    let event = timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("timeout")
        .expect("event received");
    assert_eq!(event.kind, "deleted");
}

#[tokio::test]
async fn sse_broadcaster_handles_multiple_subscribers() {
    let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
    let mut stream1 = Box::pin(broadcaster.subscribe_stream());
    let mut stream2 = Box::pin(broadcaster.subscribe_stream());

    let test_event = dto::UserEvent {
        kind: "created".to_owned(),
        id: Uuid::new_v4(),
        at: OffsetDateTime::now_utc(),
    };

    // Send event
    broadcaster.send(test_event.clone());

    // Both subscribers should receive the event
    let received1 = timeout(Duration::from_millis(100), stream1.next())
        .await
        .expect("timeout")
        .expect("event received");
    let received2 = timeout(Duration::from_millis(100), stream2.next())
        .await
        .expect("timeout")
        .expect("event received");

    assert_eq!(received1.kind, test_event.kind);
    assert_eq!(received2.kind, test_event.kind);
    assert_eq!(received1.id, received2.id);
}

#[tokio::test]
async fn sse_response_stream_works() {
    let broadcaster = SseBroadcaster::<dto::UserEvent>::new(10);
    let sse_response = broadcaster.sse_response();

    // The response should be created successfully
    // This test mainly ensures the type system works correctly
    drop(sse_response);
}
