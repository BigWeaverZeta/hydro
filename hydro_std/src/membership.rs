use std::hash::Hash;

use hydro_lang::location::{Location, MembershipEvent};
use hydro_lang::prelude::*;
use stageleft::q;

pub fn track_membership<'a, K: Hash + Eq, L: Location<'a>>(
    membership: KeyedStream<K, MembershipEvent, L, Unbounded>,
) -> KeyedSingleton<K, (), L, Unbounded> {
    membership
        .fold(
            q!(|| false),
            q!(|present, event| {
                match event {
                    MembershipEvent::Joined => *present = true,
                    MembershipEvent::Left => *present = false,
                }
            }),
        )
        .filter_map(q!(|v| if v { Some(()) } else { None }))
}

#[cfg(test)]
mod tests {
    use hydro_lang::location::MembershipEvent;
    use hydro_lang::prelude::*;

    use super::track_membership;

    #[test]
    fn test_basic_join_leave() {
        let flow = FlowBuilder::new();
        let external = flow.external::<()>();
        let node = flow.process::<()>();

        let (port, input) = node.source_external_keyed_bincode(&external);
        let out_port = track_membership(input)
            .entries()
            .send_bincode_external(&external);

        flow.sim().exhaustive(async move |mut compiled| {
            let in_send = compiled.connect(&port);
            let out_recv = compiled.connect(&out_port);
            compiled.launch();

            // Member 1 joins
            in_send.send((1, MembershipEvent::Joined)).unwrap();

            // Member 1 should now be in the membership set
            out_recv.assert_yields_only_unordered([(1, ())]).await;
        });
    }

    #[test]
    fn test_multiple_members_join_leave() {
        let flow = FlowBuilder::new();
        let external = flow.external::<()>();
        let node = flow.process::<()>();

        let (port, input) = node.source_external_keyed_bincode(&external);
        let out_port = track_membership(input)
            .entries()
            .send_bincode_external(&external);

        let compiled_sim = flow.sim().compiled();

        // Test case 1: Multiple members join
        compiled_sim.exhaustive(async |mut compiled| {
            let in_send = compiled.connect(&port);
            let out_recv = compiled.connect(&out_port);
            compiled.launch();

            in_send.send((1, MembershipEvent::Joined)).unwrap();
            in_send.send((2, MembershipEvent::Joined)).unwrap();
            in_send.send((3, MembershipEvent::Joined)).unwrap();

            // All three members should be present
            out_recv
                .assert_yields_only_unordered([(1, ()), (2, ()), (3, ())])
                .await;
        });

        // Test case 2: Member leaves
        compiled_sim.exhaustive(async |mut compiled| {
            let in_send = compiled.connect(&port);
            let out_recv = compiled.connect(&out_port);
            compiled.launch();

            in_send.send((1, MembershipEvent::Joined)).unwrap();
            in_send.send((2, MembershipEvent::Joined)).unwrap();
            in_send.send((1, MembershipEvent::Left)).unwrap();

            // Only member 2 should remain
            out_recv.assert_yields_only_unordered([(2, ())]).await;
        });

        // Test case 3: Member rejoins after leaving
        compiled_sim.exhaustive(async |mut compiled| {
            let in_send = compiled.connect(&port);
            let out_recv = compiled.connect(&out_port);
            compiled.launch();

            in_send.send((1, MembershipEvent::Joined)).unwrap();
            in_send.send((1, MembershipEvent::Left)).unwrap();
            in_send.send((1, MembershipEvent::Joined)).unwrap();

            // Member 1 should be present again
            out_recv.assert_yields_only_unordered([(1, ())]).await;
        });
    }
}
