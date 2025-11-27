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
    use futures_util::SinkExt;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_inspect_basic() {
        let inspected = RefCell::new(Vec::new());
        let result = RefCell::new(Vec::new());
        let mut sink = Inspect::new(
            |x: &i32| inspected.borrow_mut().push(*x),
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::send(&mut sink, 3).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*inspected.borrow(), vec![1, 2, 3]);
        assert_eq!(*result.borrow(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_inspect_count() {
        let count = RefCell::new(0);
        let result = RefCell::new(Vec::new());
        let mut sink = Inspect::new(
            |_x: &i32| *count.borrow_mut() += 1,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 10).await.unwrap();
        SinkExt::send(&mut sink, 20).await.unwrap();
        SinkExt::send(&mut sink, 30).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*count.borrow(), 3);
        assert_eq!(*result.borrow(), vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn test_inspect_no_modify() {
        let result = RefCell::new(Vec::new());
        let mut sink = Inspect::new(
            |x: &String| { let _ = x.len(); }, // Access but don't modify
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, "hello".to_string()).await.unwrap();
        SinkExt::send(&mut sink, "world".to_string()).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec!["hello".to_string(), "world".to_string()]);
    }

    #[tokio::test]
    async fn test_inspect_with_builder() {
        use crate::{SinkBuild, SinkBuilder};
        let inspected = RefCell::new(Vec::new());
        let result = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<i32>::new()
            .inspect(|x| inspected.borrow_mut().push(*x))
            .map(|x| x * 2)
            .for_each(|x| result.borrow_mut().push(x));
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 1).await.unwrap();
        SinkExt::send(sink.as_mut(), 2).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*inspected.borrow(), vec![1, 2]);
        assert_eq!(*result.borrow(), vec![2, 4]);
    }

    #[tokio::test]
    async fn test_inspect_multiple() {
        let first = RefCell::new(Vec::new());
        let second = RefCell::new(Vec::new());
        let result = RefCell::new(Vec::new());
        
        use crate::{SinkBuild, SinkBuilder};
        let sink = SinkBuilder::<i32>::new()
            .inspect(|x| first.borrow_mut().push(*x))
            .map(|x| x + 10)
            .inspect(|x| second.borrow_mut().push(*x))
            .for_each(|x| result.borrow_mut().push(x));
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 1).await.unwrap();
        SinkExt::send(sink.as_mut(), 2).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*first.borrow(), vec![1, 2]);
        assert_eq!(*second.borrow(), vec![11, 12]);
        assert_eq!(*result.borrow(), vec![11, 12]);
    }
}
