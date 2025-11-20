use std::hash::Hash;

use hydro_lang::live_collections::stream::NoOrder;
use hydro_lang::location::{Location, NoTick};
use hydro_lang::prelude::*;

type JoinResponses<K, M, V, L> = Stream<(K, (M, V)), L, Unbounded, NoOrder>;

/// Given an incoming stream of request-response responses, joins with metadata generated
/// at request time that is stored in-memory.
///
/// The metadata must be generated in the same or a previous tick than the response,
/// typically at request time. Only one response element should be produced with a given
/// key, same for the metadata stream.
pub fn join_responses<'a, K: Clone + Eq + Hash, M: Clone, V: Clone, L: Location<'a> + NoTick>(
    responses: Stream<(K, V), L, Unbounded, NoOrder>,
    metadata: Stream<(K, M), Tick<L>, Bounded, NoOrder>,
) -> JoinResponses<K, M, V, L> {
    let tick = metadata.location().clone();
    let (remaining_to_join_complete_cycle, remaining_to_join) =
        tick.cycle::<Stream<_, _, _, NoOrder>>();

    let remaining_and_new: Stream<(K, M), Tick<L>, Bounded, _> = remaining_to_join.chain(metadata);

    let responses = responses.batch(
        &tick,
        nondet!(
            /// Because we persist the metadata, delays resulting from
            /// batching boundaries do not affect the output contents.
        ),
    );

    // TODO(shadaj): we should have a "split-join" operator
    // that returns both join and anti-join without cloning
    let joined_this_tick =
        remaining_and_new
            .clone()
            .join(responses.clone())
            .map(q!(|(key, (meta, resp))| (key, (meta, resp))));

    remaining_to_join_complete_cycle
        .complete_next_tick(remaining_and_new.anti_join(responses.map(q!(|(key, _)| key))));

    joined_this_tick.all_ticks()
}

#[cfg(test)]
mod tests {
    use hydro_lang::prelude::*;

    use super::join_responses;

    #[test]
    fn test_join_responses_basic() {
        let flow = FlowBuilder::new();
        let external = flow.external::<()>();
        let node = flow.process::<()>();

        let (response_port, responses) = node.source_external_bincode(&external);
        let tick = node.tick();
        let (metadata_port, metadata) = tick.source_external_bincode(&external);

        let out_port = join_responses(responses, metadata).send_bincode_external(&external);

        flow.sim().exhaustive(async move |mut compiled| {
            let response_send = compiled.connect(&response_port);
            let metadata_send = compiled.connect(&metadata_port);
            let out_recv = compiled.connect(&out_port);
            compiled.launch();

            // Send metadata first
            metadata_send.send((1, "request_1")).unwrap();
            metadata_send.send((2, "request_2")).unwrap();

            // Send corresponding responses
            response_send.send((1, "response_1")).unwrap();
            response_send.send((2, "response_2")).unwrap();

            // Should receive joined pairs
            out_recv
                .assert_yields_only_unordered([
                    (1, ("request_1", "response_1")),
                    (2, ("request_2", "response_2")),
                ])
                .await;
        });
    }

    #[test]
    fn test_join_responses_delayed_response() {
        let flow = FlowBuilder::new();
        let external = flow.external::<()>();
        let node = flow.process::<()>();

        let (response_port, responses) = node.source_external_bincode(&external);
        let tick = node.tick();
        let (metadata_port, metadata) = tick.source_external_bincode(&external);

        let out_port = join_responses(responses, metadata).send_bincode_external(&external);

        flow.sim().exhaustive(async move |mut compiled| {
            let response_send = compiled.connect(&response_port);
            let metadata_send = compiled.connect(&metadata_port);
            let out_recv = compiled.connect(&out_port);
            compiled.launch();

            // Send metadata but delay response
            metadata_send.send((1, 100)).unwrap();
            metadata_send.send((2, 200)).unwrap();

            // Only send one response
            response_send.send((1, "ok")).unwrap();

            // Should only receive the joined pair for key 1
            out_recv.assert_yields_only_unordered([(1, (100, "ok"))]).await;
        });
    }
}
