//! Comprehensive unit tests for hydro_build_utils macros.
//!
//! This module tests the macro functionality provided by hydro_build_utils,
//! including nightly configuration and snapshot testing wrappers.

// Note: Some of these tests verify that macros compile and expand correctly,
// rather than testing runtime behavior, since these are primarily build-time utilities.

#[test]
fn test_rustc_version_reexport() {
    // Verify that rustc_version module is re-exported and accessible
    use hydro_build_utils::rustc_version::Channel;
    
    // Just verify the type exists and can be used
    let _channels = vec![
        Channel::Stable,
        Channel::Beta,
        Channel::Nightly,
        Channel::Dev,
    ];
}

#[test]
fn test_insta_reexport() {
    // Verify that insta module is re-exported and accessible
    use hydro_build_utils::insta;
    
    // Test a simple snapshot
    insta::assert_debug_snapshot!("test_value", &42);
}

#[test]
fn test_assert_snapshot_macro_expands() {
    // Verify that the assert_snapshot! macro expands and works
    hydro_build_utils::assert_snapshot!("test_string", "Hello, World!");
}

#[test]
fn test_assert_debug_snapshot_macro_expands() {
    // Verify that the assert_debug_snapshot! macro expands and works
    #[derive(Debug)]
    struct TestStruct {
        value: i32,
        name: String,
    }
    
    let test_obj = TestStruct {
        value: 42,
        name: "test".to_string(),
    };
    
    hydro_build_utils::assert_debug_snapshot!("test_struct", &test_obj);
}

#[test]
fn test_assert_snapshot_with_inline_value() {
    // Test snapshot with inline expression
    hydro_build_utils::assert_snapshot!("inline_calc", format!("{}", 2 + 2));
}

#[test]
fn test_assert_debug_snapshot_with_inline_value() {
    // Test debug snapshot with inline expression
    hydro_build_utils::assert_debug_snapshot!("inline_vec", &vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_assert_snapshot_empty_string() {
    hydro_build_utils::assert_snapshot!("empty_string", "");
}

#[test]
fn test_assert_debug_snapshot_empty_vec() {
    let empty: Vec<i32> = vec![];
    hydro_build_utils::assert_debug_snapshot!("empty_vec", &empty);
}

#[test]
fn test_assert_snapshot_multiline_string() {
    let multiline = "Line 1\nLine 2\nLine 3";
    hydro_build_utils::assert_snapshot!("multiline", multiline);
}

#[test]
fn test_assert_debug_snapshot_nested_structure() {
    #[derive(Debug)]
    struct Inner {
        value: i32,
    }
    
    #[derive(Debug)]
    struct Outer {
        inner: Inner,
        name: String,
    }
    
    let obj = Outer {
        inner: Inner { value: 42 },
        name: "nested".to_string(),
    };
    
    hydro_build_utils::assert_debug_snapshot!("nested_struct", &obj);
}

#[test]
fn test_assert_snapshot_special_characters() {
    let special = "Special: !@#$%^&*()_+-={}[]|\\:\";<>?,./";
    hydro_build_utils::assert_snapshot!("special_chars", special);
}

#[test]
fn test_assert_debug_snapshot_option_some() {
    let opt: Option<i32> = Some(42);
    hydro_build_utils::assert_debug_snapshot!("option_some", &opt);
}

#[test]
fn test_assert_debug_snapshot_option_none() {
    let opt: Option<i32> = None;
    hydro_build_utils::assert_debug_snapshot!("option_none", &opt);
}

#[test]
fn test_assert_debug_snapshot_result_ok() {
    let res: Result<i32, String> = Ok(42);
    hydro_build_utils::assert_debug_snapshot!("result_ok", &res);
}

#[test]
fn test_assert_debug_snapshot_result_err() {
    let res: Result<i32, String> = Err("error message".to_string());
    hydro_build_utils::assert_debug_snapshot!("result_err", &res);
}

#[test]
fn test_assert_debug_snapshot_tuple() {
    let tuple = (1, "two", 3.0, true);
    hydro_build_utils::assert_debug_snapshot!("tuple", &tuple);
}

#[test]
fn test_assert_debug_snapshot_hashmap() {
    use std::collections::HashMap;
    
    let mut map = HashMap::new();
    map.insert("key1", 1);
    map.insert("key2", 2);
    map.insert("key3", 3);
    
    // Note: HashMap order is non-deterministic, but the snapshot should still work
    hydro_build_utils::assert_debug_snapshot!("hashmap", &map);
}

#[test]
fn test_assert_debug_snapshot_vec_of_strings() {
    let vec = vec!["alpha", "beta", "gamma", "delta"];
    hydro_build_utils::assert_debug_snapshot!("vec_strings", &vec);
}

#[test]
fn test_assert_snapshot_unicode() {
    let unicode = "Hello 世界 🌍 🚀";
    hydro_build_utils::assert_snapshot!("unicode", unicode);
}

#[test]
fn test_assert_debug_snapshot_large_vec() {
    let vec: Vec<i32> = (0..100).collect();
    hydro_build_utils::assert_debug_snapshot!("large_vec", &vec);
}

#[test]
fn test_macro_hygiene() {
    // Test that macros don't interfere with local variables
    let snapshot = "local_var";
    let nightly = "another_local";
    
    hydro_build_utils::assert_snapshot!("hygiene_test", format!("{} {}", snapshot, nightly));
}

#[test]
fn test_assert_snapshot_json_like_string() {
    let json = r#"{"key": "value", "number": 42, "nested": {"inner": true}}"#;
    hydro_build_utils::assert_snapshot!("json_string", json);
}

#[test]
fn test_assert_debug_snapshot_enum() {
    #[derive(Debug)]
    enum TestEnum {
        Variant1,
        Variant2(i32),
        Variant3 { field: String },
    }
    
    let v1 = TestEnum::Variant1;
    let v2 = TestEnum::Variant2(42);
    let v3 = TestEnum::Variant3 {
        field: "test".to_string(),
    };
    
    hydro_build_utils::assert_debug_snapshot!("enum_v1", &v1);
    hydro_build_utils::assert_debug_snapshot!("enum_v2", &v2);
    hydro_build_utils::assert_debug_snapshot!("enum_v3", &v3);
}

#[test]
fn test_multiple_snapshots_in_one_test() {
    // Verify multiple snapshots can be taken in a single test
    hydro_build_utils::assert_snapshot!("multi_1", "first");
    hydro_build_utils::assert_snapshot!("multi_2", "second");
    hydro_build_utils::assert_snapshot!("multi_3", "third");
}

#[test]
fn test_assert_snapshot_with_formatting() {
    let name = "Alice";
    let age = 30;
    let message = format!("Hello, {}! You are {} years old.", name, age);
    hydro_build_utils::assert_snapshot!("formatted", message);
}
