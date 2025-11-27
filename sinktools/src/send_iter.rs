//! [`SendIter`] and related items.
use core::pin::Pin;
use core::task::{Context, Poll, ready};

use pin_project_lite::pin_project;

use crate::{Sink, SinkBuild};

pin_project! {
    /// [`Future`] for pulling from an [`Iterator`] and pushing to a [`Sink`].
    #[must_use = "futures do nothing unless polled"]
    pub struct SendIter<Pull, Push> {
        pull: Pull,
        #[pin]
        push: Push,
    }
}
impl<Pull, Push> SendIter<Pull, Push>
where
    Self: Future,
{
    /// Create a new [`SendIter`] from the given `pull` and `push` sides.
    pub fn new(pull: Pull, push: Push) -> Self {
        Self { pull, push }
    }
}
impl<Pull, Push> Future for SendIter<Pull, Push>
where
    Pull: Iterator,
    Push: Sink<Pull::Item>,
{
    type Output = Result<(), Push::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            ready!(this.push.as_mut().poll_ready(cx)?);
            if let Some(item) = this.pull.next() {
                let () = this.push.as_mut().start_send(item)?;
            } else {
                break;
            }
        }
        this.push.as_mut().poll_flush(cx)
    }
}

/// [`SinkBuild`] for [`SendIter`]s.
pub struct SendIterBuild<Iter> {
    pub(crate) iter: Iter,
}
impl<Iter> SinkBuild for SendIterBuild<Iter>
where
    Iter: Iterator,
{
    type Item = Iter::Item;

    type Output<Next: Sink<Self::Item>> = SendIter<Iter, Next>;
    fn send_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: Sink<Self::Item>,
    {
        SendIter::new(self.iter, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_each::ForEach;
    use crate::ToSinkBuild;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_send_iter_basic() {
        let result = RefCell::new(Vec::new());
        let iter = vec![1, 2, 3, 4, 5].into_iter();
        let sink = ForEach::new(|x| result.borrow_mut().push(x));
        
        let send_iter = SendIter::new(iter, sink);
        send_iter.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_send_iter_empty() {
        let result = RefCell::new(Vec::new());
        let iter = Vec::<i32>::new().into_iter();
        let sink = ForEach::new(|x| result.borrow_mut().push(x));
        
        let send_iter = SendIter::new(iter, sink);
        send_iter.await.unwrap();
        
        assert_eq!(*result.borrow(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_send_iter_range() {
        let result = RefCell::new(Vec::new());
        let iter = 0..5;
        let sink = ForEach::new(|x| result.borrow_mut().push(x));
        
        let send_iter = SendIter::new(iter, sink);
        send_iter.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_send_iter_strings() {
        let result = RefCell::new(Vec::new());
        let iter = vec!["hello".to_string(), "world".to_string()].into_iter();
        let sink = ForEach::new(|x| result.borrow_mut().push(x));
        
        let send_iter = SendIter::new(iter, sink);
        send_iter.await.unwrap();
        
        assert_eq!(*result.borrow(), vec!["hello".to_string(), "world".to_string()]);
    }

    #[tokio::test]
    async fn test_send_iter_with_builder() {
        let result = RefCell::new(Vec::new());
        let iter = vec![1, 2, 3].into_iter();
        
        let fut = iter
            .iter_to_sink_build()
            .map(|x| x * 2)
            .for_each(|x| result.borrow_mut().push(x));
        
        fut.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_send_iter_with_filter() {
        let result = RefCell::new(Vec::new());
        let iter = vec![1, 2, 3, 4, 5, 6].into_iter();
        
        let fut = iter
            .iter_to_sink_build()
            .filter(|x| *x % 2 == 0)
            .for_each(|x| result.borrow_mut().push(x));
        
        fut.await.unwrap();
        
        assert_eq!(*result.borrow(), vec![2, 4, 6]);
    }
}
