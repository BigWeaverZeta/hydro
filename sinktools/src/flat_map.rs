//! [`FlatMap`] and related items.
use core::pin::Pin;
use core::task::{Context, Poll, ready};

use pin_project_lite::pin_project;

use crate::{Sink, SinkBuild};

pin_project! {
    /// Same as [`core::iterator::FlatMap`] but as a [`Sink`].
    ///
    /// Synchronously maps and flattens items, and sends the outputs to the following sink.
    #[must_use = "sinks do nothing unless polled"]
    pub struct FlatMap<Si, Func, IntoIter>
    where
        IntoIter: IntoIterator,
    {
        #[pin]
        sink: Si,
        func: Func,
        // Current iterator and the next item.
        iter_next: Option<(IntoIter::IntoIter, IntoIter::Item)>,
    }
}

impl<Si, Func, IntoIter> FlatMap<Si, Func, IntoIter>
where
    IntoIter: IntoIterator,
{
    /// Create with flat-mapping function `func` and next `sink`.
    pub fn new<Item>(func: Func, sink: Si) -> Self
    where
        Self: Sink<Item>,
    {
        Self {
            sink,
            func,
            iter_next: None,
        }
    }

    fn poll_ready_impl(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Si::Error>>
    where
        Si: Sink<IntoIter::Item>,
    {
        let mut this = self.project();

        while this.iter_next.is_some() {
            // Ensure following sink is ready.
            ready!(this.sink.as_mut().poll_ready(cx))?;

            // Send the item.
            let (mut iter, next) = this.iter_next.take().unwrap();
            this.sink.as_mut().start_send(next)?;

            // Replace the iterator and next item (if any).
            *this.iter_next = iter.next().map(|next| (iter, next));
        }

        Poll::Ready(Ok(()))
    }
}

impl<Si, Func, Item, IntoIter> Sink<Item> for FlatMap<Si, Func, IntoIter>
where
    Si: Sink<IntoIter::Item>,
    Func: FnMut(Item) -> IntoIter,
    IntoIter: IntoIterator,
{
    type Error = Si::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_ready_impl(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        let this = self.project();

        assert!(
            this.iter_next.is_none(),
            "Sink not ready: `poll_ready` must be called and return `Ready` before `start_send` is called."
        );
        let mut iter = (this.func)(item).into_iter();
        *this.iter_next = iter.next().map(|next| (iter, next));
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_ready_impl(cx)?);
        self.project().sink.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_ready_impl(cx)?);
        self.project().sink.poll_close(cx)
    }
}

/// [`SinkBuild`] for [`FlatMap`].
pub struct FlatMapBuilder<Prev, Func> {
    pub(crate) prev: Prev,
    pub(crate) func: Func,
}
impl<Prev, Func, IntoIter> SinkBuild for FlatMapBuilder<Prev, Func>
where
    Prev: SinkBuild,
    Func: FnMut(Prev::Item) -> IntoIter,
    IntoIter: IntoIterator,
{
    type Item = IntoIter::Item;

    type Output<Next: Sink<IntoIter::Item>> = Prev::Output<FlatMap<Next, Func, IntoIter>>;

    fn send_to<Next>(self, next: Next) -> Self::Output<Next>
    where
        Next: Sink<IntoIter::Item>,
    {
        self.prev.send_to(FlatMap::new(self.func, next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_each::ForEach;
    use futures_util::SinkExt;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_flat_map_basic() {
        let result = RefCell::new(Vec::new());
        let mut sink = FlatMap::new(
            |x: i32| vec![x, x * 2, x * 3],
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 2, 3, 2, 4, 6]);
    }

    #[tokio::test]
    async fn test_flat_map_string_chars() {
        let result = RefCell::new(Vec::new());
        let mut sink = FlatMap::new(
            |s: String| s.chars().collect::<Vec<_>>(),
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, "hi".to_string()).await.unwrap();
        SinkExt::send(&mut sink, "go".to_string()).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec!['h', 'i', 'g', 'o']);
    }

    #[tokio::test]
    async fn test_flat_map_empty_iterators() {
        let result = RefCell::new(Vec::new());
        let mut sink = FlatMap::new(
            |_x: i32| Vec::<i32>::new(),
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_flat_map_mixed_sizes() {
        let result = RefCell::new(Vec::new());
        let mut sink = FlatMap::new(
            |x: i32| {
                if x % 2 == 0 {
                    vec![x]
                } else {
                    vec![x, x + 10, x + 20]
                }
            },
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::send(&mut sink, 3).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 11, 21, 2, 3, 13, 23]);
    }

    #[tokio::test]
    async fn test_flat_map_with_builder() {
        use crate::{SinkBuild, SinkBuilder};
        let result = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<i32>::new()
            .flat_map(|x| vec![x, x * 2])
            .for_each(|x| result.borrow_mut().push(x));
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 3).await.unwrap();
        SinkExt::send(sink.as_mut(), 5).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![3, 6, 5, 10]);
    }

    #[tokio::test]
    async fn test_flat_map_with_range() {
        let result = RefCell::new(Vec::new());
        let mut sink = FlatMap::new(
            |x: i32| 0..x,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 3).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![0, 1, 2, 0, 1]);
    }
}
