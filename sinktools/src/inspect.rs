//! [`Inspect`] and related items.
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project_lite::pin_project;

use crate::{Sink, SinkBuild};

pin_project! {
    /// Same as [`core::iterator::Inspect`] but as a [`Sink`].
    ///
    /// Synchronously inspects items before sending them to the following sink.
    #[must_use = "sinks do nothing unless polled"]
    pub struct Inspect<Si, Func> {
        #[pin]
        sink: Si,
        func: Func,
    }
}

impl<Si, Func> Inspect<Si, Func> {
    /// Creates with inspecting `func` and next `sink`.
    pub fn new<Item>(func: Func, sink: Si) -> Self
    where
        Self: Sink<Item>,
    {
        Self { sink, func }
    }
}

impl<Si, Func, Item> Sink<Item> for Inspect<Si, Func>
where
    Si: Sink<Item>,
    Func: FnMut(&Item),
{
    type Error = Si::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_ready(cx)
    }
    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        let this = self.project();
        (this.func)(&item);
        this.sink.start_send(item)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_close(cx)
    }
}

/// [`SinkBuild`] for [`Inspect`].
pub struct InspectBuilder<Prev, Func> {
    pub(crate) prev: Prev,
    pub(crate) func: Func,
}
impl<Prev, Func> SinkBuild for InspectBuilder<Prev, Func>
where
    Prev: SinkBuild,
    Func: FnMut(&Prev::Item),
{
    type Item = Prev::Item;

    type Output<Next: Sink<Prev::Item>> = Prev::Output<Inspect<Next, Func>>;

    fn send_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: Sink<Prev::Item>,
    {
        self.prev.send_to(Inspect::new(self.func, next))
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
    fn test_inspect_passthrough() {
        let inspected = RefCell::new(Vec::new());
        let results = RefCell::new(Vec::new());
        
        let sink = Inspect::new(
            |x: &i32| {
                inspected.borrow_mut().push(*x);
            },
            ForEach::new(|item| {
                results.borrow_mut().push(item);
            })
        );
        
        let mut sink = pin!(sink);
        let cx = &mut Context::from_waker(noop_waker_ref());
        
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_ready(cx));
        assert_eq!(Ok(()), sink.as_mut().start_send(1));
        assert_eq!(Ok(()), sink.as_mut().start_send(2));
        assert_eq!(Ok(()), sink.as_mut().start_send(3));
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_flush(cx));
        
        // Both should have the same values
        assert_eq!(*inspected.borrow(), vec![1, 2, 3]);
        assert_eq!(*results.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn test_inspect_side_effects() {
        let mut counter = 0;
        let counter_ref = &mut counter;
        let results = RefCell::new(Vec::new());
        
        let sink = Inspect::new(
            move |_: &i32| {
                *counter_ref += 1;
            },
            ForEach::new(|item| {
                results.borrow_mut().push(item);
            })
        );
        
        let mut sink = pin!(sink);
        let cx = &mut Context::from_waker(noop_waker_ref());
        
        assert_eq!(Ok(()), sink.as_mut().start_send(10));
        assert_eq!(Ok(()), sink.as_mut().start_send(20));
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_flush(cx));
        
        assert_eq!(counter, 2);
        assert_eq!(*results.borrow(), vec![10, 20]);
    }
}
