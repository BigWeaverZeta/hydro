//! [`Map`] and related items.
use core::pin::Pin;

use pin_project_lite::pin_project;

use crate::{Sink, SinkBuild, forward_sink};

pin_project! {
    /// Same as [`core::iterator::Map`] but as a [`Sink`].
    ///
    /// Synchronously maps items and sends the output to the following sink.
    #[must_use = "sinks do nothing unless polled"]
    pub struct Map<Si, Func> {
        #[pin]
        sink: Si,
        func: Func,
    }
}

impl<Si, Func> Map<Si, Func> {
    /// Creates with mapping `func` and next `sink`.
    pub fn new<Item>(func: Func, sink: Si) -> Self
    where
        Self: Sink<Item>,
    {
        Self { sink, func }
    }
}

impl<Si, Func, Item, ItemOut> Sink<Item> for Map<Si, Func>
where
    Si: Sink<ItemOut>,
    Func: FnMut(Item) -> ItemOut,
{
    type Error = Si::Error;

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        let this = self.project();
        let item = (this.func)(item);
        this.sink.start_send(item)
    }

    forward_sink!(poll_ready, poll_flush, poll_close);
}

/// [`SinkBuild`] for [`Map`].
pub struct MapBuilder<Prev, Func> {
    pub(crate) prev: Prev,
    pub(crate) func: Func,
}
impl<Prev, ItemOut, Func> SinkBuild for MapBuilder<Prev, Func>
where
    Prev: SinkBuild,
    Func: FnMut(Prev::Item) -> ItemOut,
{
    type Item = ItemOut;

    type Output<Next: Sink<ItemOut>> = Prev::Output<Map<Next, Func>>;

    fn send_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: Sink<ItemOut>,
    {
        self.prev.send_to(Map::new(self.func, next))
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
    fn test_map_transform() {
        let results = RefCell::new(Vec::new());
        
        let sink = Map::new(
            |x: i32| x * 2,
            ForEach::new(|item| {
                results.borrow_mut().push(item);
            })
        );
        
        let mut sink = pin!(sink);
        let cx = &mut Context::from_waker(noop_waker_ref());
        
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_ready(cx));
        assert_eq!(Ok(()), sink.as_mut().start_send(5));
        assert_eq!(Ok(()), sink.as_mut().start_send(10));
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_flush(cx));
        
        assert_eq!(*results.borrow(), vec![10, 20]);
    }

    #[test]
    fn test_map_type_conversion() {
        let results = RefCell::new(Vec::new());
        
        let sink = Map::new(
            |x: i32| x.to_string(),
            ForEach::new(|item: String| {
                results.borrow_mut().push(item);
            })
        );
        
        let mut sink = pin!(sink);
        let cx = &mut Context::from_waker(noop_waker_ref());
        
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_ready(cx));
        assert_eq!(Ok(()), sink.as_mut().start_send(42));
        assert_eq!(Ok(()), sink.as_mut().start_send(100));
        assert_eq!(Poll::Ready(Ok(())), sink.as_mut().poll_flush(cx));
        
        assert_eq!(*results.borrow(), vec!["42".to_string(), "100".to_string()]);
    }
}
