//! Comprehensive unit tests for membership tracking functionality.
//!
//! This module tests the track_membership function which tracks cluster member
//! join and leave events to maintain the current membership state.

use hydro_lang::location::MembershipEvent;
use hydro_lang::prelude::*;
use hydro_std::membership::track_membership;

#[test]
fn test_track_membership_single_join() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Member 1 joins
        in_send.send((1, MembershipEvent::Joined)).unwrap();

        // Should see member 1 as present
        out_recv.assert_yields_only_unordered([(1, ())]).await;
    });
}

#[test]
fn test_track_membership_multiple_joins() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Multiple members join
        in_send.send((1, MembershipEvent::Joined)).unwrap();
        in_send.send((2, MembershipEvent::Joined)).unwrap();
        in_send.send((3, MembershipEvent::Joined)).unwrap();

        // Should see all three members
        out_recv
            .assert_yields_only_unordered([(1, ()), (2, ()), (3, ())])
            .await;
    });
}

#[test]
fn test_track_membership_join_then_leave() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Member 1 joins then leaves
        in_send.send((1, MembershipEvent::Joined)).unwrap();
        in_send.send((1, MembershipEvent::Left)).unwrap();

        // Member should not appear in output after leaving
        out_recv.assert_no_more().await;
    });
}

#[test]
fn test_track_membership_multiple_members_mixed_events() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Multiple members with different states
        in_send.send((1, MembershipEvent::Joined)).unwrap();
        in_send.send((2, MembershipEvent::Joined)).unwrap();
        in_send.send((1, MembershipEvent::Left)).unwrap(); // Member 1 leaves
        in_send.send((3, MembershipEvent::Joined)).unwrap();

        // Should only see members 2 and 3 (member 1 left)
        out_recv.assert_yields_only_unordered([(2, ()), (3, ())]).await;
    });
}

#[test]
fn test_track_membership_rejoin_after_leave() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Member joins, leaves, then rejoins
        in_send.send((1, MembershipEvent::Joined)).unwrap();
        in_send.send((1, MembershipEvent::Left)).unwrap();
        in_send.send((1, MembershipEvent::Joined)).unwrap();

        // Member should be present after rejoining
        out_recv.assert_yields_only_unordered([(1, ())]).await;
    });
}

#[test]
fn test_track_membership_empty_input() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let _in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // No events sent
        out_recv.assert_no_more().await;
    });
}

#[test]
fn test_track_membership_only_leaves() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Only leave events (members never joined)
        in_send.send((1, MembershipEvent::Left)).unwrap();
        in_send.send((2, MembershipEvent::Left)).unwrap();

        // Should have no members (they were never present)
        out_recv.assert_no_more().await;
    });
}

#[test]
fn test_track_membership_duplicate_joins() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Member joins multiple times (idempotent)
        in_send.send((1, MembershipEvent::Joined)).unwrap();
        in_send.send((1, MembershipEvent::Joined)).unwrap();
        in_send.send((1, MembershipEvent::Joined)).unwrap();

        // Should still only see member once
        out_recv.assert_yields_only_unordered([(1, ())]).await;
    });
}

#[test]
fn test_track_membership_duplicate_leaves() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Member joins then leaves multiple times
        in_send.send((1, MembershipEvent::Joined)).unwrap();
        in_send.send((1, MembershipEvent::Left)).unwrap();
        in_send.send((1, MembershipEvent::Left)).unwrap();

        // Member should not be present
        out_recv.assert_no_more().await;
    });
}

#[test]
fn test_track_membership_string_keys() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Using String keys instead of integers
        in_send
            .send(("alice".to_string(), MembershipEvent::Joined))
            .unwrap();
        in_send
            .send(("bob".to_string(), MembershipEvent::Joined))
            .unwrap();
        in_send
            .send(("alice".to_string(), MembershipEvent::Left))
            .unwrap();

        // Only bob should remain
        out_recv
            .assert_yields_only_unordered([("bob".to_string(), ())])
            .await;
    });
}

#[test]
fn test_track_membership_complex_sequence() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let (port, input) = node.source_external_bincode(&external);
    let out_port = track_membership(input).send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let in_send = compiled.connect(&port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Complex sequence of events
        in_send.send((1, MembershipEvent::Joined)).unwrap(); // 1 joins
        in_send.send((2, MembershipEvent::Joined)).unwrap(); // 2 joins
        in_send.send((3, MembershipEvent::Joined)).unwrap(); // 3 joins
        in_send.send((2, MembershipEvent::Left)).unwrap(); // 2 leaves
        in_send.send((4, MembershipEvent::Joined)).unwrap(); // 4 joins
        in_send.send((1, MembershipEvent::Left)).unwrap(); // 1 leaves
        in_send.send((2, MembershipEvent::Joined)).unwrap(); // 2 rejoins

        // Members 2, 3, and 4 should be present (1 left)
        out_recv
            .assert_yields_only_unordered([(2, ()), (3, ()), (4, ())])
            .await;
    });
}
