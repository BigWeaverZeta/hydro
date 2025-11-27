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
    use futures_util::SinkExt;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_map_basic() {
        let result = RefCell::new(Vec::new());
        let mut sink = Map::new(
            |x: i32| x * 2,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::send(&mut sink, 3).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_map_string_to_length() {
        let result = RefCell::new(Vec::new());
        let mut sink = Map::new(
            |s: String| s.len(),
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, "hello".to_string()).await.unwrap();
        SinkExt::send(&mut sink, "world".to_string()).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![5, 5]);
    }

    #[tokio::test]
    async fn test_map_type_conversion() {
        let result = RefCell::new(Vec::new());
        let mut sink = Map::new(
            |x: i32| x.to_string(),
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 42).await.unwrap();
        SinkExt::send(&mut sink, -10).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec!["42".to_string(), "-10".to_string()]);
    }

    #[tokio::test]
    async fn test_map_tuple_extraction() {
        let result = RefCell::new(Vec::new());
        let mut sink = Map::new(
            |(x, y): (i32, i32)| x + y,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, (1, 2)).await.unwrap();
        SinkExt::send(&mut sink, (3, 4)).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![3, 7]);
    }

    #[tokio::test]
    async fn test_map_empty_stream() {
        let result = RefCell::new(Vec::new());
        let mut sink = Map::new(
            |x: i32| x * 2,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::flush(&mut sink).await.unwrap();
        SinkExt::close(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_map_with_close() {
        let result = RefCell::new(Vec::new());
        let mut sink = Map::new(
            |x: i32| x + 10,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::close(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![11]);
    }

    #[tokio::test]
    async fn test_map_builder() {
        use crate::{SinkBuild, SinkBuilder};
        let result = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<i32>::new()
            .map(|x| x * 3)
            .for_each(|x| result.borrow_mut().push(x));
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 5).await.unwrap();
        SinkExt::send(sink.as_mut(), 7).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![15, 21]);
    }

    #[tokio::test]
    async fn test_map_chain() {
        use crate::{SinkBuild, SinkBuilder};
        let result = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<i32>::new()
            .map(|x| x * 2)
            .map(|x| x + 1)
            .for_each(|x| result.borrow_mut().push(x));
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 1).await.unwrap();
        SinkExt::send(sink.as_mut(), 2).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![3, 5]);
    }
}
