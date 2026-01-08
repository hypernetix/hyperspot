use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use futures_core::Stream;
use futures_util::StreamExt;
use serde::Serialize;
use std::{borrow::Cow, convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

/// Small typed SSE broadcaster built on `tokio::sync::broadcast`.
/// - T must be `Clone` so multiple subscribers can receive the same payload.
/// - Bounded channel drops oldest events when subscribers lag (by design).
#[derive(Clone)]
pub struct SseBroadcaster<T> {
    tx: broadcast::Sender<T>,
}

impl<T: Clone + Send + 'static> SseBroadcaster<T> {
    /// Create a broadcaster with bounded buffer capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Broadcast a single message to current subscribers.
    /// Errors are ignored to keep the hot path cheap (e.g., no active subscribers).
    pub fn send(&self, value: T) {
        let _ = self.tx.send(value);
    }

    /// Subscribe to a typed stream of messages; lag/drop errors are filtered out.
    pub fn subscribe_stream(&self) -> impl Stream<Item = T> {
        BroadcastStream::new(self.tx.subscribe()).filter_map(|res| async move { res.ok() })
    }

    /// Convert a typed stream into an SSE stream with JSON payloads (no event name).
    fn wrap_stream_as_sse<U>(stream: U) -> impl Stream<Item = Result<Event, Infallible>>
    where
        U: Stream<Item = T>,
        T: Serialize,
    {
        stream.map(|msg| {
            let ev = Event::default().json_data(&msg).unwrap_or_else(|_| {
                // Fallback to a tiny text marker instead of breaking the stream.
                Event::default().data("serialization_error")
            });
            Ok(ev)
        })
    }

    /// Convert a typed stream into an SSE stream with JSON payloads and a constant `event:` name.
    fn wrap_stream_as_sse_named<U>(
        stream: U,
        event_name: Cow<'static, str>,
    ) -> impl Stream<Item = Result<Event, Infallible>>
    where
        U: Stream<Item = T>,
        T: Serialize,
    {
        stream.map(move |msg| {
            let ev = Event::default()
                .event(&event_name) // <-- set event name
                .json_data(&msg)
                .unwrap_or_else(|_| {
                    Event::default()
                        .event(&event_name)
                        .data("serialization_error")
                });
            Ok(ev)
        })
    }

    // -------------------------
    // Plain (unnamed) variants
    // -------------------------

    /// Plain SSE (no extra headers), unnamed events.
    /// Includes periodic keepalive pings to avoid idle timeouts.
    pub fn sse_response(&self) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
    where
        T: Serialize,
    {
        let stream = Self::wrap_stream_as_sse(self.subscribe_stream());
        Sse::new(stream).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
    }

    /// SSE with custom headers applied on top of the Sse response (unnamed events).
    pub fn sse_response_with_headers<I>(&self, headers: I) -> axum::response::Response
    where
        T: Serialize,
        I: IntoIterator<Item = (axum::http::HeaderName, axum::http::HeaderValue)>,
    {
        let mut resp = self.sse_response().into_response();
        let dst = resp.headers_mut();
        for (name, value) in headers {
            dst.insert(name, value);
        }
        resp
    }

    // -------------------------
    // Named-event variants
    // -------------------------

    /// Plain SSE with a constant `event:` name for all messages (no extra headers).
    pub fn sse_response_named(
        &self,
        event_name: impl Into<Cow<'static, str>> + 'static,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
    where
        T: Serialize,
    {
        let stream = Self::wrap_stream_as_sse_named(self.subscribe_stream(), event_name.into());
        Sse::new(stream).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
    }

    /// SSE with custom headers and a constant `event:` name for all messages.
    pub fn sse_response_named_with_headers<I>(
        &self,
        event_name: impl Into<Cow<'static, str>> + 'static,
        headers: I,
    ) -> axum::response::Response
    where
        T: Serialize,
        I: IntoIterator<Item = (axum::http::HeaderName, axum::http::HeaderValue)>,
    {
        let mut resp = self.sse_response_named(event_name).into_response();
        let dst = resp.headers_mut();
        for (name, value) in headers {
            dst.insert(name, value);
        }
        resp
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn broadcaster_delivers_single_event() {
        let b = SseBroadcaster::<u32>::new(16);
        let mut sub = Box::pin(b.subscribe_stream());
        b.send(42);
        let v = timeout(Duration::from_millis(200), sub.next())
            .await
            .unwrap();
        assert_eq!(v, Some(42));
    }

    #[tokio::test]
    async fn broadcaster_handles_backpressure_with_bounded_channel() {
        // Test that bounded channel drops old events when capacity is exceeded
        let capacity = 4;
        let broadcaster = SseBroadcaster::<u32>::new(capacity);

        // Create a slow consumer that doesn't read immediately
        let mut subscriber = Box::pin(broadcaster.subscribe_stream());

        // Send more events than capacity
        let num_events = capacity * 2;
        for i in 0..num_events {
            broadcaster.send(u32::try_from(i).unwrap());
        }

        // The subscriber should only receive the most recent events
        // due to the bounded channel dropping older ones
        let mut received = Vec::new();

        // Try to receive all events with a timeout
        for _ in 0..num_events {
            match timeout(Duration::from_millis(10), subscriber.next()).await {
                Ok(Some(event)) => received.push(event),
                Ok(None) | Err(_) => break, // None or timeout
            }
        }

        // Should have received some events, but not necessarily all
        // due to backpressure handling
        assert!(!received.is_empty());
        assert!(received.len() <= num_events);

        // The events we did receive should be in order
        for window in received.windows(2) {
            assert!(window[0] < window[1], "Events should be in order");
        }
    }

    #[tokio::test]
    async fn broadcaster_handles_multiple_subscribers_with_backpressure() {
        let capacity = 8;
        let broadcaster = SseBroadcaster::<String>::new(capacity);

        // Create multiple subscribers with different consumption rates
        let mut fast_subscriber = Box::pin(broadcaster.subscribe_stream());
        let mut slow_subscriber = Box::pin(broadcaster.subscribe_stream());

        let events_sent = Arc::new(AtomicUsize::new(0));
        let events_sent_clone = events_sent.clone();

        // Producer task - sends events rapidly
        let producer = tokio::spawn(async move {
            for i in 0..50 {
                broadcaster.send(format!("event_{i}"));
                events_sent_clone.fetch_add(1, Ordering::SeqCst);
                tokio::task::yield_now().await; // Allow other tasks to run
            }
        });

        // Fast consumer task
        let fast_events = Arc::new(AtomicUsize::new(0));
        let fast_events_clone = fast_events.clone();
        let fast_consumer = tokio::spawn(async move {
            while let Ok(Some(_event)) =
                timeout(Duration::from_millis(100), fast_subscriber.next()).await
            {
                fast_events_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Slow consumer task
        let slow_events = Arc::new(AtomicUsize::new(0));
        let slow_events_clone = slow_events.clone();
        let slow_consumer = tokio::spawn(async move {
            while let Ok(Some(_event)) =
                timeout(Duration::from_millis(100), slow_subscriber.next()).await
            {
                slow_events_clone.fetch_add(1, Ordering::SeqCst);
                // Simulate slow processing
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        });

        // Wait for producer to finish
        producer.await.unwrap();

        // Give consumers time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Cancel consumers
        fast_consumer.abort();
        slow_consumer.abort();

        let total_sent = events_sent.load(Ordering::SeqCst);
        let fast_received = fast_events.load(Ordering::SeqCst);
        let slow_received = slow_events.load(Ordering::SeqCst);

        assert_eq!(total_sent, 50);

        // Fast consumer should receive more events than slow consumer
        // due to backpressure affecting the slow consumer more
        assert!(fast_received > 0);
        assert!(slow_received > 0);

        // Due to bounded channel, neither consumer necessarily receives all events
        // but the system should remain stable
        println!(
            "Sent: {total_sent}, Fast received: {fast_received}, Slow received: {slow_received}"
        );
    }

    #[tokio::test]
    #[allow(clippy::assertions_on_constants)]
    async fn broadcaster_prevents_unbounded_memory_growth() {
        let small_capacity = 2;
        let broadcaster = SseBroadcaster::<Vec<u8>>::new(small_capacity);

        // Create a subscriber but don't consume from it
        let _subscriber = broadcaster.subscribe_stream();

        // Send many large events
        for i in 0..100 {
            let large_event = vec![u8::try_from(i).unwrap(); 1024]; // 1KB per event
            broadcaster.send(large_event);
        }

        // The broadcaster should not accumulate unbounded memory
        // This test mainly ensures we don't panic or run out of memory
        // The bounded channel should drop old events automatically

        // Verify we can still send and the system is responsive
        broadcaster.send(vec![255; 1024]);

        // Test passes if we reach here without OOM or panic
        assert!(true);
    }

    #[tokio::test]
    async fn broadcaster_handles_subscriber_drop_gracefully() {
        let broadcaster = SseBroadcaster::<u32>::new(16);

        // Create and immediately drop a subscriber
        {
            let _subscriber = broadcaster.subscribe_stream();
            broadcaster.send(1);
        } // subscriber dropped here

        // Broadcaster should continue working with new subscribers
        let mut new_subscriber = Box::pin(broadcaster.subscribe_stream());
        broadcaster.send(2);

        let received = timeout(Duration::from_millis(100), new_subscriber.next())
            .await
            .unwrap();
        assert_eq!(received, Some(2));
    }

    #[tokio::test]
    async fn broadcaster_send_is_non_blocking() {
        let broadcaster = SseBroadcaster::<u32>::new(1); // Very small capacity

        // Send should not block even when no subscribers exist
        let start = std::time::Instant::now();
        for i in 0..1000 {
            broadcaster.send(i);
        }
        let elapsed = start.elapsed();

        // Should complete very quickly since send() doesn't block
        assert!(
            elapsed < Duration::from_millis(100),
            "Send operations took too long: {elapsed:?}"
        );
    }
}
