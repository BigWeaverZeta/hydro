//! [`DemuxMap`] and related items.
use core::fmt::Debug;
use core::hash::Hash;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::collections::HashMap;

use crate::{Sink, ready_both};

/// Sink which receives keys paired with items `(Key, Item)`, and pushes to the corresponding output sink in a [`HashMap`] of sinks.
pub struct DemuxMap<Key, Si> {
    sinks: HashMap<Key, Si>,
}

impl<Key, Si> DemuxMap<Key, Si> {
    /// Create with the given next `sinks` map.
    pub fn new<Item>(sinks: impl Into<HashMap<Key, Si>>) -> Self
    where
        Self: Sink<(Key, Item)>,
    {
        Self {
            sinks: sinks.into(),
        }
    }
}

impl<Key, Si, Item> Sink<(Key, Item)> for DemuxMap<Key, Si>
where
    Key: Eq + Hash + Debug + Unpin,
    Si: Sink<Item> + Unpin,
{
    type Error = Si::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut()
            .sinks
            .values_mut()
            .try_fold(Poll::Ready(()), |poll, sink| {
                ready_both!(poll, Pin::new(sink).poll_ready(cx)?);
                Poll::Ready(Ok(()))
            })
    }

    fn start_send(self: Pin<&mut Self>, item: (Key, Item)) -> Result<(), Self::Error> {
        let sink = self
            .get_mut()
            .sinks
            .get_mut(&item.0)
            .unwrap_or_else(|| panic!("`DemuxMap` missing key {:?}", item.0));
        Pin::new(sink).start_send(item.1)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut()
            .sinks
            .values_mut()
            .try_fold(Poll::Ready(()), |poll, sink| {
                ready_both!(poll, Pin::new(sink).poll_flush(cx)?);
                Poll::Ready(Ok(()))
            })
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut()
            .sinks
            .values_mut()
            .try_fold(Poll::Ready(()), |poll, sink| {
                ready_both!(poll, Pin::new(sink).poll_close(cx)?);
                Poll::Ready(Ok(()))
            })
    }
}

/// Creates a `DemuxMap` sink that sends each item to one of many outputs, depending on the key.
///
/// This requires sinks `Si` to be `Unpin`. If your sinks are not `Unpin`, first wrap them in `Box::pin` to make them `Unpin`.
pub fn demux_map<Key, Si, Item>(sinks: impl Into<HashMap<Key, Si>>) -> DemuxMap<Key, Si>
where
    Key: Eq + Hash + Debug + Unpin,
    Si: Sink<Item> + Unpin,
{
    DemuxMap::new(sinks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_each::ForEach;
    use futures_util::SinkExt;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_demux_map_routes_to_correct_sink() {
        let result_a = RefCell::new(Vec::new());
        let result_b = RefCell::new(Vec::new());

        let sink_a = ForEach::new(|x: i32| result_a.borrow_mut().push(x));
        let sink_b = ForEach::new(|x: i32| result_b.borrow_mut().push(x));

        let mut sinks_map = HashMap::new();
        sinks_map.insert("a", sink_a);
        sinks_map.insert("b", sink_b);

        let mut demux: DemuxMap<&str, _> = DemuxMap::new::<i32>(sinks_map);

        SinkExt::send(&mut demux, ("a", 1)).await.unwrap();
        SinkExt::send(&mut demux, ("b", 2)).await.unwrap();
        SinkExt::send(&mut demux, ("a", 3)).await.unwrap();
        SinkExt::send(&mut demux, ("b", 4)).await.unwrap();
        SinkExt::flush(&mut demux).await.unwrap();

        assert_eq!(*result_a.borrow(), vec![1, 3]);
        assert_eq!(*result_b.borrow(), vec![2, 4]);
    }

    #[tokio::test]
    async fn test_demux_map_single_key() {
        let result = RefCell::new(Vec::new());

        let sink = ForEach::new(|x: i32| result.borrow_mut().push(x));

        let mut sinks_map = HashMap::new();
        sinks_map.insert("only", sink);

        let mut demux: DemuxMap<&str, _> = DemuxMap::new::<i32>(sinks_map);

        SinkExt::send(&mut demux, ("only", 10)).await.unwrap();
        SinkExt::send(&mut demux, ("only", 20)).await.unwrap();
        SinkExt::send(&mut demux, ("only", 30)).await.unwrap();
        SinkExt::flush(&mut demux).await.unwrap();

        assert_eq!(*result.borrow(), vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn test_demux_map_empty_after_close() {
        let result_a = RefCell::new(Vec::new());
        let result_b = RefCell::new(Vec::new());

        let sink_a = ForEach::new(|x: i32| result_a.borrow_mut().push(x));
        let sink_b = ForEach::new(|x: i32| result_b.borrow_mut().push(x));

        let mut sinks_map = HashMap::new();
        sinks_map.insert("a", sink_a);
        sinks_map.insert("b", sink_b);

        let mut demux: DemuxMap<&str, _> = DemuxMap::new::<i32>(sinks_map);

        SinkExt::send(&mut demux, ("a", 100)).await.unwrap();
        SinkExt::send(&mut demux, ("b", 200)).await.unwrap();
        SinkExt::close(&mut demux).await.unwrap();

        assert_eq!(*result_a.borrow(), vec![100]);
        assert_eq!(*result_b.borrow(), vec![200]);
    }

    #[tokio::test]
    async fn test_demux_map_free_fn() {
        let result_x = RefCell::new(Vec::new());
        let result_y = RefCell::new(Vec::new());

        let sink_x = ForEach::new(|x: i32| result_x.borrow_mut().push(x));
        let sink_y = ForEach::new(|x: i32| result_y.borrow_mut().push(x));

        let mut sinks_map = HashMap::new();
        sinks_map.insert("x", sink_x);
        sinks_map.insert("y", sink_y);

        let mut demux = demux_map(sinks_map);

        SinkExt::send(&mut demux, ("x", 5)).await.unwrap();
        SinkExt::send(&mut demux, ("y", 10)).await.unwrap();
        SinkExt::send(&mut demux, ("x", 15)).await.unwrap();
        SinkExt::flush(&mut demux).await.unwrap();

        assert_eq!(*result_x.borrow(), vec![5, 15]);
        assert_eq!(*result_y.borrow(), vec![10]);
    }
}
