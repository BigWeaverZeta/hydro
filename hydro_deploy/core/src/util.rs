use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use futures::{Future, Stream, StreamExt};
use tokio::sync::{mpsc, oneshot};

pub async fn async_retry<T, E, F: Future<Output = Result<T, E>>>(
    mut thunk: impl FnMut() -> F,
    count: usize,
    delay: Duration,
) -> Result<T, E> {
    for _ in 1..count {
        let result = thunk().await;
        if result.is_ok() {
            return result;
        } else {
            tokio::time::sleep(delay).await;
        }
    }

    thunk().await
}

#[derive(Clone)]
pub struct PriorityBroadcast(Weak<Mutex<PriorityBroadcastInternal>>);

struct PriorityBroadcastInternal {
    priority_sender: Option<oneshot::Sender<String>>,
    senders: Vec<(Option<String>, mpsc::UnboundedSender<String>)>,
}

impl PriorityBroadcast {
    pub fn receive_priority(&self) -> oneshot::Receiver<String> {
        let (sender, receiver) = oneshot::channel::<String>();

        if let Some(internal) = self.0.upgrade() {
            let mut internal = internal.lock().unwrap();
            let prev_sender = internal.priority_sender.replace(sender);
            if prev_sender.is_some() {
                panic!("Only one deploy stdout receiver is allowed at a time");
            }
        }

        receiver
    }

    pub fn receive(&self, prefix: Option<String>) -> mpsc::UnboundedReceiver<String> {
        let (sender, receiver) = mpsc::unbounded_channel::<String>();

        if let Some(internal) = self.0.upgrade() {
            let mut internal = internal.lock().unwrap();
            internal.senders.push((prefix, sender));
        }

        receiver
    }
}

pub fn prioritized_broadcast<T: Stream<Item = std::io::Result<String>> + Send + Unpin + 'static>(
    mut lines: T,
    fallback_receiver: impl Fn(String) + Send + 'static,
) -> PriorityBroadcast {
    let internal = Arc::new(Mutex::new(PriorityBroadcastInternal {
        priority_sender: None,
        senders: Vec::new(),
    }));

    let weak_internal = Arc::downgrade(&internal);

    // TODO(mingwei): eliminate the need for a separate task.
    tokio::spawn(async move {
        while let Some(Ok(line)) = lines.next().await {
            let mut internal = internal.lock().unwrap();

            // Priority receiver
            if let Some(priority_sender) = internal.priority_sender.take()
                && priority_sender.send(line.clone()).is_ok()
            {
                continue; // Skip regular receivers if successfully sent to the priority receiver.
            }

            // Regular receivers
            internal.senders.retain(|receiver| !receiver.1.is_closed());

            let mut successful_send = false;
            for (prefix_filter, sender) in internal.senders.iter() {
                // Send to specific receivers if the filter prefix matches
                if prefix_filter
                    .as_ref()
                    .is_none_or(|prefix| line.starts_with(prefix))
                {
                    successful_send |= sender.send(line.clone()).is_ok();
                }
            }

            // If no receivers successfully received the line, use the fallback receiver.
            if !successful_send {
                (fallback_receiver)(line);
            }
        }
        // Dropping `internal` will close all senders because it is the only strong `Arc` reference.
    });

    PriorityBroadcast(weak_internal)
}

#[cfg(test)]
mod test {
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::UnboundedReceiverStream;

    use super::*;

    #[tokio::test]
    async fn broadcast_listeners_close_when_source_does() {
        let (tx, rx) = mpsc::unbounded_channel();
        let priority_broadcast = prioritized_broadcast(UnboundedReceiverStream::new(rx), |_| {});

        let mut rx2 = priority_broadcast.receive(None);

        tx.send(Ok("hello".to_string())).unwrap();
        assert_eq!(rx2.recv().await, Some("hello".to_string()));

        let wait_again = tokio::spawn(async move { rx2.recv().await });

        drop(tx);

        assert_eq!(wait_again.await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_async_retry_success_first_try() {
        let mut attempts = 0;
        let result = async_retry(
            || async {
                attempts += 1;
                Ok::<i32, String>(42)
            },
            3,
            Duration::from_millis(10),
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 1);
    }

    #[tokio::test]
    async fn test_async_retry_success_after_failures() {
        let mut attempts = 0;
        let result = async_retry(
            || async {
                attempts += 1;
                if attempts < 3 {
                    Err("not yet")
                } else {
                    Ok(100)
                }
            },
            5,
            Duration::from_millis(10),
        )
        .await;

        assert_eq!(result, Ok(100));
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_async_retry_all_failures() {
        let mut attempts = 0;
        let result = async_retry(
            || async {
                attempts += 1;
                Err::<i32, &str>("always fails")
            },
            3,
            Duration::from_millis(10),
        )
        .await;

        assert_eq!(result, Err("always fails"));
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_async_retry_single_attempt() {
        let mut attempts = 0;
        let result = async_retry(
            || async {
                attempts += 1;
                Err::<i32, &str>("fails")
            },
            1,
            Duration::from_millis(10),
        )
        .await;

        assert_eq!(result, Err("fails"));
        assert_eq!(attempts, 1);
    }

    #[tokio::test]
    async fn test_priority_broadcast_basic() {
        let (tx, rx) = mpsc::unbounded_channel();
        let broadcast = prioritized_broadcast(UnboundedReceiverStream::new(rx), |_| {});

        let mut receiver = broadcast.receive(None);

        tx.send(Ok("message1".to_string())).unwrap();
        tx.send(Ok("message2".to_string())).unwrap();

        assert_eq!(receiver.recv().await, Some("message1".to_string()));
        assert_eq!(receiver.recv().await, Some("message2".to_string()));
    }

    #[tokio::test]
    async fn test_priority_broadcast_with_prefix() {
        let (tx, rx) = mpsc::unbounded_channel();
        let broadcast = prioritized_broadcast(UnboundedReceiverStream::new(rx), |_| {});

        let mut receiver = broadcast.receive(Some("[PREFIX]".to_string()));

        tx.send(Ok("[PREFIX] matched".to_string())).unwrap();
        tx.send(Ok("not matched".to_string())).unwrap();
        tx.send(Ok("[PREFIX] also matched".to_string())).unwrap();

        assert_eq!(receiver.recv().await, Some("[PREFIX] matched".to_string()));
        assert_eq!(
            receiver.recv().await,
            Some("[PREFIX] also matched".to_string())
        );
    }

    #[tokio::test]
    async fn test_priority_broadcast_priority_receiver() {
        let (tx, rx) = mpsc::unbounded_channel();
        let broadcast = prioritized_broadcast(UnboundedReceiverStream::new(rx), |_| {});

        let priority_rx = broadcast.receive_priority();
        let mut normal_rx = broadcast.receive(None);

        tx.send(Ok("priority message".to_string())).unwrap();

        // Priority receiver should get the message
        assert_eq!(
            priority_rx.await,
            Ok("priority message".to_string())
        );

        // Normal receiver should not get it since priority took it
        tx.send(Ok("normal message".to_string())).unwrap();
        assert_eq!(normal_rx.recv().await, Some("normal message".to_string()));
    }

    #[tokio::test]
    async fn test_priority_broadcast_fallback() {
        let (tx, rx) = mpsc::unbounded_channel();
        let fallback_called = Arc::new(Mutex::new(Vec::new()));
        let fallback_called_clone = fallback_called.clone();

        let _broadcast = prioritized_broadcast(UnboundedReceiverStream::new(rx), move |msg| {
            fallback_called_clone.lock().unwrap().push(msg);
        });

        // No receivers, so fallback should be called
        tx.send(Ok("fallback message".to_string())).unwrap();

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        let messages = fallback_called.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "fallback message");
    }

    #[tokio::test]
    async fn test_priority_broadcast_multiple_receivers() {
        let (tx, rx) = mpsc::unbounded_channel();
        let broadcast = prioritized_broadcast(UnboundedReceiverStream::new(rx), |_| {});

        let mut rx1 = broadcast.receive(None);
        let mut rx2 = broadcast.receive(None);

        tx.send(Ok("broadcast".to_string())).unwrap();

        // Both receivers should get the message
        assert_eq!(rx1.recv().await, Some("broadcast".to_string()));
        assert_eq!(rx2.recv().await, Some("broadcast".to_string()));
    }

    #[tokio::test]
    async fn test_priority_broadcast_cloning() {
        let (tx, rx) = mpsc::unbounded_channel();
        let broadcast = prioritized_broadcast(UnboundedReceiverStream::new(rx), |_| {});

        let broadcast_clone = broadcast.clone();
        let mut receiver = broadcast_clone.receive(None);

        tx.send(Ok("clone test".to_string())).unwrap();

        assert_eq!(receiver.recv().await, Some("clone test".to_string()));
    }
}
