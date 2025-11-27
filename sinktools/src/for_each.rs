//! [`ForEach`] consuming sink.
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project_lite::pin_project;

use crate::Sink;

pin_project! {
    /// Same as [`core::iterator::ForEach`] but as a [`Sink`].
    ///
    /// Synchronously consumes items and always returns `Poll::Ready(Ok(())`.
    #[must_use = "sinks do nothing unless polled"]
    pub struct ForEach<Func> {
        func: Func,
    }
}
impl<Func> ForEach<Func> {
    /// Create with consuming `func`.
    pub fn new<Item>(func: Func) -> Self
    where
        Self: Sink<Item>,
    {
        Self { func }
    }
}
impl<Func, Item> Sink<Item> for ForEach<Func>
where
    Func: FnMut(Item),
{
    type Error = core::convert::Infallible;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        (self.project().func)(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::SinkExt;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_for_each_basic() {
        let result = RefCell::new(Vec::new());
        let mut sink = ForEach::new(|x: i32| result.borrow_mut().push(x));
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::send(&mut sink, 3).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_for_each_side_effects() {
        let sum = RefCell::new(0);
        let mut sink = ForEach::new(|x: i32| *sum.borrow_mut() += x);
        
        SinkExt::send(&mut sink, 5).await.unwrap();
        SinkExt::send(&mut sink, 10).await.unwrap();
        SinkExt::send(&mut sink, 15).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*sum.borrow(), 30);
    }

    #[tokio::test]
    async fn test_for_each_empty() {
        let result = RefCell::new(Vec::new());
        let mut sink = ForEach::new(|x: i32| result.borrow_mut().push(x));
        
        SinkExt::flush(&mut sink).await.unwrap();
        SinkExt::close(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_for_each_strings() {
        let result = RefCell::new(String::new());
        let mut sink = ForEach::new(|s: String| result.borrow_mut().push_str(&s));
        
        SinkExt::send(&mut sink, "hello".to_string()).await.unwrap();
        SinkExt::send(&mut sink, " ".to_string()).await.unwrap();
        SinkExt::send(&mut sink, "world".to_string()).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), "hello world".to_string());
    }

    #[tokio::test]
    async fn test_for_each_with_close() {
        let count = RefCell::new(0);
        let mut sink = ForEach::new(|_: i32| *count.borrow_mut() += 1);
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::close(&mut sink).await.unwrap();
        
        assert_eq!(*count.borrow(), 2);
    }
}
