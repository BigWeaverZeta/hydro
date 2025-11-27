//! [`TryForEach`] consuming sink.
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project_lite::pin_project;

use crate::Sink;

pin_project! {
    /// Same as [`crate::ForEach`] but the closure returns `Result<(), Error>` instead of `()`.
    ///
    /// This is useful when you want to handle errors in the closure.
    ///
    /// Synchronously consumes items and always returns `Poll::Ready(Ok(())`.
    #[must_use = "sinks do nothing unless polled"]
    pub struct TryForEach<Func> {
        func: Func,
    }
}
impl<Func> TryForEach<Func> {
    /// Create with consuming `func`.
    pub fn new<Item>(func: Func) -> Self
    where
        Self: Sink<Item>,
    {
        Self { func }
    }
}
impl<Func, Item, Error> Sink<Item> for TryForEach<Func>
where
    Func: FnMut(Item) -> Result<(), Error>,
{
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        (self.project().func)(item)
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
    async fn test_try_for_each_success() {
        let result = RefCell::new(Vec::new());
        let mut sink = TryForEach::new(|x: i32| {
            result.borrow_mut().push(x);
            Ok::<(), String>(())
        });
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::send(&mut sink, 3).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_try_for_each_error() {
        let result = RefCell::new(Vec::new());
        let mut sink = TryForEach::new(|x: i32| {
            result.borrow_mut().push(x);
            if x == 2 {
                Err("error at 2")
            } else {
                Ok(())
            }
        });
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        let err = SinkExt::send(&mut sink, 2).await;
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), "error at 2");
        
        assert_eq!(*result.borrow(), vec![1, 2]);
    }

    #[tokio::test]
    async fn test_try_for_each_validation() {
        let result = RefCell::new(Vec::new());
        let mut sink = TryForEach::new(|x: i32| {
            if x < 0 {
                Err("negative value")
            } else {
                result.borrow_mut().push(x);
                Ok(())
            }
        });
        
        SinkExt::send(&mut sink, 5).await.unwrap();
        SinkExt::send(&mut sink, 10).await.unwrap();
        let err = SinkExt::send(&mut sink, -1).await;
        assert!(err.is_err());
        
        assert_eq!(*result.borrow(), vec![5, 10]);
    }

    #[tokio::test]
    async fn test_try_for_each_all_errors() {
        let mut sink = TryForEach::new(|_x: i32| {
            Err::<(), &str>("always fails")
        });
        
        let err1 = SinkExt::send(&mut sink, 1).await;
        assert!(err1.is_err());
        
        let err2 = SinkExt::send(&mut sink, 2).await;
        assert!(err2.is_err());
    }

    #[tokio::test]
    async fn test_try_for_each_with_builder() {
        use crate::{SinkBuild, SinkBuilder};
        let result = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<i32>::new()
            .map(|x| x * 2)
            .try_for_each(|x| {
                result.borrow_mut().push(x);
                Ok::<(), String>(())
            });
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 1).await.unwrap();
        SinkExt::send(sink.as_mut(), 2).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![2, 4]);
    }
}
