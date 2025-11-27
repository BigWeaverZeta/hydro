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
    use futures_util::SinkExt;
    use std::cell::RefCell;

    #[tokio::test]
    async fn test_filter_even_numbers() {
        let result = RefCell::new(Vec::new());
        let mut sink = Filter::new(
            |x: &i32| x % 2 == 0,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::send(&mut sink, 3).await.unwrap();
        SinkExt::send(&mut sink, 4).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![2, 4]);
    }

    #[tokio::test]
    async fn test_filter_all_pass() {
        let result = RefCell::new(Vec::new());
        let mut sink = Filter::new(
            |_: &i32| true,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![1, 2]);
    }

    #[tokio::test]
    async fn test_filter_none_pass() {
        let result = RefCell::new(Vec::new());
        let mut sink = Filter::new(
            |_: &i32| false,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, 1).await.unwrap();
        SinkExt::send(&mut sink, 2).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), Vec::<i32>::new());
    }

    #[tokio::test]
    async fn test_filter_strings() {
        let result = RefCell::new(Vec::new());
        let mut sink = Filter::new(
            |s: &String| s.len() > 3,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::send(&mut sink, "hi".to_string()).await.unwrap();
        SinkExt::send(&mut sink, "hello".to_string()).await.unwrap();
        SinkExt::send(&mut sink, "bye".to_string()).await.unwrap();
        SinkExt::send(&mut sink, "world".to_string()).await.unwrap();
        SinkExt::flush(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), vec!["hello".to_string(), "world".to_string()]);
    }

    #[tokio::test]
    async fn test_filter_with_builder() {
        use crate::{SinkBuild, SinkBuilder};
        let result = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<i32>::new()
            .filter(|x| *x > 5)
            .for_each(|x| result.borrow_mut().push(x));
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 3).await.unwrap();
        SinkExt::send(sink.as_mut(), 7).await.unwrap();
        SinkExt::send(sink.as_mut(), 10).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![7, 10]);
    }

    #[tokio::test]
    async fn test_filter_and_map_chain() {
        use crate::{SinkBuild, SinkBuilder};
        let result = RefCell::new(Vec::new());
        
        let sink = SinkBuilder::<i32>::new()
            .filter(|x| *x % 2 == 0)
            .map(|x| x * 2)
            .for_each(|x| result.borrow_mut().push(x));
        
        let mut sink = Box::pin(sink);
        SinkExt::send(sink.as_mut(), 1).await.unwrap();
        SinkExt::send(sink.as_mut(), 2).await.unwrap();
        SinkExt::send(sink.as_mut(), 3).await.unwrap();
        SinkExt::send(sink.as_mut(), 4).await.unwrap();
        SinkExt::flush(sink.as_mut()).await.unwrap();
        
        assert_eq!(*result.borrow(), vec![4, 8]);
    }

    #[tokio::test]
    async fn test_filter_empty_stream() {
        let result = RefCell::new(Vec::new());
        let mut sink = Filter::new(
            |x: &i32| x % 2 == 0,
            ForEach::new(|x| result.borrow_mut().push(x))
        );
        
        SinkExt::flush(&mut sink).await.unwrap();
        SinkExt::close(&mut sink).await.unwrap();
        
        assert_eq!(*result.borrow(), Vec::<i32>::new());
    }
}
