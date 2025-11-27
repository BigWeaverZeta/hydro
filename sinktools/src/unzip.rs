//! [`Unzip`].
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project_lite::pin_project;

use crate::{Sink, ready_both};

pin_project! {
    /// Same as [`core::iterator::Unzip`] but as a [`Sink`].
    ///
    /// Synchronously maps items and sends the output to the following sink.
    #[must_use = "sinks do nothing unless polled"]
    pub struct Unzip<Si0, Si1> {
        #[pin]
        sink_0: Si0,
        #[pin]
        sink_1: Si1,
    }
}

impl<Si0, Si1> Unzip<Si0, Si1> {
    /// Creates with next sinks `sink_0` and `sink_1`.
    pub fn new<Item>(sink_0: Si0, sink_1: Si1) -> Self
    where
        Self: Sink<Item>,
    {
        Self { sink_0, sink_1 }
    }
}

impl<Si0, Si1, Item0, Item1> Sink<(Item0, Item1)> for Unzip<Si0, Si1>
where
    Si0: Sink<Item0>,
    Si1: Sink<Item1>,
    Si0::Error: From<Si1::Error>,
{
    type Error = Si0::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        ready_both!(this.sink_0.poll_ready(cx)?, this.sink_1.poll_ready(cx)?,);
        Poll::Ready(Ok(()))
    }
    fn start_send(self: Pin<&mut Self>, item: (Item0, Item1)) -> Result<(), Self::Error> {
        let this = self.project();
        this.sink_0.start_send(item.0)?;
        this.sink_1.start_send(item.1)?;
        Ok(())
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        ready_both!(this.sink_0.poll_flush(cx)?, this.sink_1.poll_flush(cx)?,);
        Poll::Ready(Ok(()))
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        ready_both!(this.sink_0.poll_close(cx)?, this.sink_1.poll_close(cx)?,);
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_each::ForEach;
    use futures_util::SinkExt;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_unzip_basic() {
        let left = RefCell::new(Vec::new());
        let right = RefCell::new(Vec::new());
        
        let mut sink = Unzip::new(
            ForEach::new(|x| left.borrow_mut().push(x)),
            ForEach::new(|x| right.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, (1, "a")).await.unwrap();
        SinkExt::send(&mut sink, (2, "b")).await.unwrap();
        SinkExt::send(&mut sink, (3, "c")).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*left.borrow(), vec![1, 2, 3]);
        assert_eq!(*right.borrow(), vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn test_unzip_same_types() {
        let left = RefCell::new(Vec::new());
        let right = RefCell::new(Vec::new());
        
        let mut sink = Unzip::new(
            ForEach::new(|x| left.borrow_mut().push(x)),
            ForEach::new(|x| right.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, (1, 10)).await.unwrap();
        SinkExt::send(&mut sink, (2, 20)).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*left.borrow(), vec![1, 2]);
        assert_eq!(*right.borrow(), vec![10, 20]);
    }

    #[tokio::test]
    async fn test_unzip_empty() {
        let left = RefCell::new(Vec::new());
        let right = RefCell::new(Vec::new());
        
        let mut sink = Unzip::new(
            ForEach::new(|x: i32| left.borrow_mut().push(x)),
            ForEach::new(|x: i32| right.borrow_mut().push(x))
        );
        
        SinkExt::flush(&mut sink).await.unwrap();
        SinkExt::close(&mut sink).await.unwrap();
        
        assert_eq!(*left.borrow(), Vec::<i32>::new());
        assert_eq!(*right.borrow(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_unzip_with_builder() {
        use crate::{SinkBuild, SinkBuilder};
        let left = RefCell::new(Vec::new());
        let right = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<(i32, String)>::new()
            .unzip(
                ForEach::new(|x| left.borrow_mut().push(x)),
                ForEach::new(|x| right.borrow_mut().push(x))
            );
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), (1, "hello".to_string())).await.unwrap();
        SinkExt::send(sink.as_mut(), (2, "world".to_string())).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*left.borrow(), vec![1, 2]);
        assert_eq!(*right.borrow(), vec!["hello".to_string(), "world".to_string()]);
    }

    #[tokio::test]
    async fn test_unzip_complex_types() {
        let left = RefCell::new(Vec::new());
        let right = RefCell::new(Vec::new());
        
        let mut sink = Unzip::new(
            ForEach::new(|x| left.borrow_mut().push(x)),
            ForEach::new(|x| right.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, (vec![1, 2], Some("test"))).await.unwrap();
        SinkExt::send(&mut sink, (vec![3], None)).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*left.borrow(), vec![vec![1, 2], vec![3]]);
        assert_eq!(*right.borrow(), vec![Some("test"), None]);
    }
}
