//! Comprehensive unit tests for request-response pattern functionality.
//!
//! This module tests the join_responses function which joins incoming responses
//! with metadata generated at request time, handling the case where responses
//! and metadata may arrive in different ticks.

use hydro_lang::prelude::*;
use hydro_std::request_response::join_responses;

#[test]
fn test_join_responses_single_request() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Send metadata for request
        meta_send.send((1, "request_metadata")).unwrap();

        // Send response
        resp_send.send((1, "response_value")).unwrap();

        // Should receive joined result
        out_recv
            .assert_yields_only_unordered([(1, ("request_metadata", "response_value"))])
            .await;
    });
}

#[test]
fn test_join_responses_multiple_requests() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Send metadata for multiple requests
        meta_send.send((1, "meta1")).unwrap();
        meta_send.send((2, "meta2")).unwrap();
        meta_send.send((3, "meta3")).unwrap();

        // Send responses
        resp_send.send((1, "resp1")).unwrap();
        resp_send.send((2, "resp2")).unwrap();
        resp_send.send((3, "resp3")).unwrap();

        // Should receive all joined results
        out_recv
            .assert_yields_only_unordered([
                (1, ("meta1", "resp1")),
                (2, ("meta2", "resp2")),
                (3, ("meta3", "resp3")),
            ])
            .await;
    });
}

#[test]
fn test_join_responses_metadata_without_response() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let _resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Send metadata but no response
        meta_send.send((1, "orphan_metadata")).unwrap();

        // Should not produce output without matching response
        out_recv.assert_no_more().await;
    });
}

#[test]
fn test_join_responses_response_without_metadata() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let _meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Send response but no metadata
        resp_send.send((1, "orphan_response")).unwrap();

        // Should not produce output without matching metadata
        out_recv.assert_no_more().await;
    });
}

#[test]
fn test_join_responses_empty_inputs() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let _resp_send = compiled.connect(&resp_port);
        let _meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // No data sent
        out_recv.assert_no_more().await;
    });
}

#[test]
fn test_join_responses_partial_matches() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Send metadata for keys 1, 2, 3
        meta_send.send((1, "m1")).unwrap();
        meta_send.send((2, "m2")).unwrap();
        meta_send.send((3, "m3")).unwrap();

        // Send responses only for keys 1 and 3
        resp_send.send((1, "r1")).unwrap();
        resp_send.send((3, "r3")).unwrap();

        // Should only get joined results for keys 1 and 3
        out_recv
            .assert_yields_only_unordered([(1, ("m1", "r1")), (3, ("m3", "r3"))])
            .await;
    });
}

#[test]
fn test_join_responses_string_keys() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Using string keys
        meta_send
            .send(("alice".to_string(), "alice_meta"))
            .unwrap();
        meta_send.send(("bob".to_string(), "bob_meta")).unwrap();

        resp_send
            .send(("alice".to_string(), "alice_resp"))
            .unwrap();
        resp_send.send(("bob".to_string(), "bob_resp")).unwrap();

        out_recv
            .assert_yields_only_unordered([
                ("alice".to_string(), ("alice_meta", "alice_resp")),
                ("bob".to_string(), ("bob_meta", "bob_resp")),
            ])
            .await;
    });
}

#[test]
fn test_join_responses_numeric_values() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Numeric metadata and responses
        meta_send.send((1, 100)).unwrap();
        meta_send.send((2, 200)).unwrap();

        resp_send.send((1, 1000)).unwrap();
        resp_send.send((2, 2000)).unwrap();

        out_recv
            .assert_yields_only_unordered([(1, (100, 1000)), (2, (200, 2000))])
            .await;
    });
}

#[test]
fn test_join_responses_complex_metadata() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Complex metadata (tuple)
        meta_send.send((1, ("user", 42, true))).unwrap();

        resp_send.send((1, "success")).unwrap();

        out_recv
            .assert_yields_only_unordered([(1, (("user", 42, true), "success"))])
            .await;
    });
}

#[test]
fn test_join_responses_ordered_sequence() {
    let flow = FlowBuilder::new();
    let external = flow.external::<()>();
    let node = flow.process::<()>();

    let tick = node.tick();
    let (resp_port, responses) = node.source_external_bincode(&external);
    let (meta_port, metadata) = tick.source_external_bincode(&external);

    let output = join_responses(responses, metadata);
    let out_port = output.send_bincode_external(&external);

    flow.sim().exhaustive(async move |mut compiled| {
        let resp_send = compiled.connect(&resp_port);
        let meta_send = compiled.connect(&meta_port);
        let out_recv = compiled.connect(&out_port);
        compiled.launch();

        // Send in sequence
        for i in 1..=5 {
            meta_send.send((i, format!("meta_{}", i))).unwrap();
            resp_send.send((i, format!("resp_{}", i))).unwrap();
        }

        out_recv
            .assert_yields_only_unordered([
                (1, ("meta_1".to_string(), "resp_1".to_string())),
                (2, ("meta_2".to_string(), "resp_2".to_string())),
                (3, ("meta_3".to_string(), "resp_3".to_string())),
                (4, ("meta_4".to_string(), "resp_4".to_string())),
                (5, ("meta_5".to_string(), "resp_5".to_string())),
            ])
            .await;
    });
}
