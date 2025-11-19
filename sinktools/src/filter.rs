//! [`Filter`] and related items.
use core::pin::Pin;

use pin_project_lite::pin_project;

use crate::{Sink, SinkBuild, forward_sink};

pin_project! {
    /// Same as [`core::iterator::Filter`] but as a [`Sink`].
    ///
    /// Synchronously filters items and sends the outputs to the following sink.
    #[must_use = "sinks do nothing unless polled"]
    pub struct Filter<Si, Func> {
        #[pin]
        sink: Si,
        func: Func,
    }
}

impl<Si, Func> Filter<Si, Func> {
    /// Creates with filtering `func` and next `sink`.
    pub fn new<Item>(func: Func, sink: Si) -> Self
    where
        Self: Sink<Item>,
    {
        Self { sink, func }
    }
}

impl<Si, Func, Item> Sink<Item> for Filter<Si, Func>
where
    Si: Sink<Item>,
    Func: FnMut(&Item) -> bool,
{
    type Error = Si::Error;

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        let this = self.project();
        if (this.func)(&item) {
            this.sink.start_send(item)
        } else {
            Ok(())
        }
    }

    forward_sink!(poll_ready, poll_flush, poll_close);
}

/// [`SinkBuild`] for [`Filter`].
pub struct FilterBuilder<Prev, Func> {
    pub(crate) prev: Prev,
    pub(crate) func: Func,
}
impl<Prev, Func> SinkBuild for FilterBuilder<Prev, Func>
where
    Prev: SinkBuild,
    Func: FnMut(&Prev::Item) -> bool,
{
    type Item = Prev::Item;

    type Output<Next: Sink<Prev::Item>> = Prev::Output<Filter<Next, Func>>;

    fn send_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: Sink<Prev::Item>,
    {
        self.prev.send_to(Filter::new(self.func, next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_each::ForEach;
    use core::cell::RefCell;
    use core::convert::Infallible;
    use core::pin::pin;
    use core::task::{Context, Poll};
    use futures_task::noop_waker_ref;

    #[test]
    fn test_filter_basic() {
        let results = RefCell::new(Vec::new());
        
        let sink = Filter::new(
            |x: &i32| *x > 10,
            ForEach::new(|item| {
                results.borrow_mut().push(item);
            })
        );
        
        let mut sink = pin!(sink);
        let cx = &mut Context::from_waker(noop_waker_ref());
        
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_ready(cx));
        assert_eq!(Ok(()), sink.as_mut().start_send(5));   // Filtered out
        assert_eq!(Ok(()), sink.as_mut().start_send(15));  // Passes
        assert_eq!(Ok(()), sink.as_mut().start_send(10));  // Filtered out
        assert_eq!(Ok(()), sink.as_mut().start_send(20));  // Passes
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_flush(cx));
        
        assert_eq!(*results.borrow(), vec![15, 20]);
    }

    #[test]
    fn test_filter_none_pass() {
        let results = RefCell::new(Vec::new());
        
        let sink = Filter::new(
            |x: &i32| *x > 100,
            ForEach::new(|item| {
                results.borrow_mut().push(item);
            })
        );
        
        let mut sink = pin!(sink);
        let cx = &mut Context::from_waker(noop_waker_ref());
        
        assert_eq!(Ok(()), sink.as_mut().start_send(5));
        assert_eq!(Ok(()), sink.as_mut().start_send(10));
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_flush(cx));
        
        assert_eq!(*results.borrow(), Vec::<i32>::new());
    }

    #[test]
    fn test_filter_all_pass() {
        let results = RefCell::new(Vec::new());
        
        let sink = Filter::new(
            |_: &i32| true,
            ForEach::new(|item| {
                results.borrow_mut().push(item);
            })
        );
        
        let mut sink = pin!(sink);
        let cx = &mut Context::from_waker(noop_waker_ref());
        
        assert_eq!(Ok(()), sink.as_mut().start_send(1));
        assert_eq!(Ok(()), sink.as_mut().start_send(2));
        assert_eq!(Ok(()), sink.as_mut().start_send(3));
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_flush(cx));
        
        assert_eq!(*results.borrow(), vec![1, 2, 3]);
    }
}
