//! [`SendStream`] and related items.
use core::pin::Pin;
use core::task::{Context, Poll, ready};

use futures_util::Stream;
use pin_project_lite::pin_project;

use crate::{Sink, SinkBuild};

pin_project! {
    /// [`Future`] for pulling from an [`Iterator`] and pushing to a [`Sink`].
    #[must_use = "futures do nothing unless polled"]
    pub struct SendStream<Pull, Push> {
        #[pin]
        pull: Pull,
        #[pin]
        push: Push,
    }
}
impl<Pull, Push> SendStream<Pull, Push>
where
    Self: Future,
{
    /// Create a new [`SendStream`] from the given `pull` and `push` sides.
    pub fn new(pull: Pull, push: Push) -> Self {
        Self { pull, push }
    }
}
impl<Pull, Push> Future for SendStream<Pull, Push>
where
    Pull: Stream,
    Push: Sink<Pull::Item>,
{
    type Output = Result<(), Push::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            ready!(this.push.as_mut().poll_ready(cx)?);
            if let Some(item) = ready!(this.pull.as_mut().poll_next(cx)) {
                let () = this.push.as_mut().start_send(item)?;
            } else {
                break;
            }
        }
        this.push.as_mut().poll_flush(cx)
    }
}

/// [`SinkBuild`] for [`SendStream`].
pub struct SendStreamBuild<St> {
    pub(crate) stream: St,
}
impl<St> SinkBuild for SendStreamBuild<St>
where
    St: Stream,
{
    type Item = St::Item;

    type Output<Next: Sink<Self::Item>> = SendStream<St, Next>;
    fn send_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: Sink<Self::Item>,
    {
        SendStream::new(self.stream, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_each::ForEach;
    use crate::ToSinkBuild;
    use futures_util::stream;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_send_stream_basic() {
        let result = RefCell::new(Vec::new());
        let stream = stream::iter(vec![1, 2, 3, 4, 5]);
        let sink = ForEach::new(|x| result.borrow_mut().push(x));
        
        let send_stream = SendStream::new(stream, sink);
        send_stream.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_send_stream_empty() {
        let result = RefCell::new(Vec::new());
        let stream = stream::iter(Vec::<i32>::new());
        let sink = ForEach::new(|x| result.borrow_mut().push(x));
        
        let send_stream = SendStream::new(stream, sink);
        send_stream.await.unwrap();
        
        assert_eq!(*result.borrow(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_send_stream_strings() {
        let result = RefCell::new(Vec::new());
        let stream = stream::iter(vec!["hello".to_string(), "world".to_string()]);
        let sink = ForEach::new(|x| result.borrow_mut().push(x));
        
        let send_stream = SendStream::new(stream, sink);
        send_stream.await.unwrap();
        
        assert_eq!(*result.borrow(), vec!["hello".to_string(), "world".to_string()]);
    }

    #[tokio::test]
    async fn test_send_stream_with_builder() {
        let result = RefCell::new(Vec::new());
        let stream = stream::iter(vec![1, 2, 3]);
        
        let fut = stream
            .stream_to_sink_build()
            .map(|x| x * 2)
            .for_each(|x| result.borrow_mut().push(x));
        
        fut.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_send_stream_with_filter() {
        let result = RefCell::new(Vec::new());
        let stream = stream::iter(vec![1, 2, 3, 4, 5, 6]);
        
        let fut = stream
            .stream_to_sink_build()
            .filter(|x| *x % 2 == 0)
            .for_each(|x| result.borrow_mut().push(x));
        
        fut.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_send_stream_with_filter_map() {
        let result = RefCell::new(Vec::new());
        let stream = stream::iter(vec!["1", "not", "2", "invalid", "3"]);
        
        let fut = stream
            .stream_to_sink_build()
            .filter_map(|s| s.parse::<i32>().ok())
            .for_each(|x| result.borrow_mut().push(x));
        
        fut.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 2, 3]);
    }
}
