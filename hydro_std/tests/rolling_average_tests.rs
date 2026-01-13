//! Comprehensive unit tests for the RollingAverage statistics tracker.
//!
//! This module tests all functionality of the RollingAverage struct, including:
//! - Basic sample addition and counting
//! - Mean calculation
//! - Variance and standard deviation
//! - Confidence interval computation
//! - Combining multiple RollingAverage instances
//! - Edge cases (empty, single sample, etc.)

use hydro_std::bench_client::rolling_average::RollingAverage;

#[test]
fn test_new_rolling_average() {
    let ra = RollingAverage::new();
    assert_eq!(ra.sample_count(), 0);
    assert_eq!(ra.sample_mean(), 0.0);
    assert_eq!(ra.sample_variance(), 0.0);
    assert_eq!(ra.sample_std_dev(), 0.0);
}

#[test]
fn test_default_rolling_average() {
    let ra = RollingAverage::default();
    assert_eq!(ra.sample_count(), 0);
    assert_eq!(ra.sample_mean(), 0.0);
}

#[test]
fn test_add_single_sample() {
    let mut ra = RollingAverage::new();
    ra.add_sample(5.0);
    
    assert_eq!(ra.sample_count(), 1);
    assert_eq!(ra.sample_mean(), 5.0);
    assert_eq!(ra.sample_variance(), 0.0); // Single sample has no variance
    assert_eq!(ra.sample_std_dev(), 0.0);
}

#[test]
fn test_add_multiple_samples() {
    let mut ra = RollingAverage::new();
    ra.add_sample(1.0);
    ra.add_sample(2.0);
    ra.add_sample(3.0);
    ra.add_sample(4.0);
    ra.add_sample(5.0);
    
    assert_eq!(ra.sample_count(), 5);
    assert_eq!(ra.sample_mean(), 3.0);
}

#[test]
fn test_sample_mean_calculation() {
    let mut ra = RollingAverage::new();
    
    // Add samples that should average to 10.0
    ra.add_sample(5.0);
    ra.add_sample(10.0);
    ra.add_sample(15.0);
    
    assert_eq!(ra.sample_count(), 3);
    assert_eq!(ra.sample_mean(), 10.0);
}

#[test]
fn test_sample_variance_two_samples() {
    let mut ra = RollingAverage::new();
    ra.add_sample(2.0);
    ra.add_sample(4.0);
    
    // Variance = sum((x - mean)^2) / (n - 1)
    // mean = 3, variance = ((2-3)^2 + (4-3)^2) / 1 = 2
    assert_eq!(ra.sample_variance(), 2.0);
}

#[test]
fn test_sample_variance_multiple_samples() {
    let mut ra = RollingAverage::new();
    ra.add_sample(1.0);
    ra.add_sample(2.0);
    ra.add_sample(3.0);
    ra.add_sample(4.0);
    ra.add_sample(5.0);
    
    // mean = 3.0
    // variance = ((1-3)^2 + (2-3)^2 + (3-3)^2 + (4-3)^2 + (5-3)^2) / 4
    // variance = (4 + 1 + 0 + 1 + 4) / 4 = 10 / 4 = 2.5
    assert_eq!(ra.sample_variance(), 2.5);
}

#[test]
fn test_sample_std_dev() {
    let mut ra = RollingAverage::new();
    ra.add_sample(1.0);
    ra.add_sample(2.0);
    ra.add_sample(3.0);
    ra.add_sample(4.0);
    ra.add_sample(5.0);
    
    // Standard deviation is sqrt of variance
    let expected_std_dev = 2.5_f64.sqrt();
    assert!((ra.sample_std_dev() - expected_std_dev).abs() < 1e-10);
}

#[test]
fn test_confidence_interval_insufficient_samples() {
    let mut ra = RollingAverage::new();
    
    // No samples
    assert_eq!(ra.confidence_interval_99(), None);
    
    // One sample
    ra.add_sample(5.0);
    assert_eq!(ra.confidence_interval_99(), None);
}

#[test]
fn test_confidence_interval_two_samples() {
    let mut ra = RollingAverage::new();
    ra.add_sample(2.0);
    ra.add_sample(4.0);
    
    let interval = ra.confidence_interval_99();
    assert!(interval.is_some());
    
    if let Some((lower, upper)) = interval {
        // Mean is 3.0, should have reasonable bounds
        assert!(lower < 3.0);
        assert!(upper > 3.0);
        assert!(lower < upper);
    }
}

#[test]
fn test_confidence_interval_multiple_samples() {
    let mut ra = RollingAverage::new();
    for i in 1..=100 {
        ra.add_sample(i as f64);
    }
    
    let mean = ra.sample_mean();
    let interval = ra.confidence_interval_99();
    assert!(interval.is_some());
    
    if let Some((lower, upper)) = interval {
        // Mean should be within the interval
        assert!(lower <= mean);
        assert!(upper >= mean);
        assert!(lower < upper);
    }
}

#[test]
fn test_confidence_interval_narrow_distribution() {
    let mut ra = RollingAverage::new();
    // All samples very close to each other
    for _ in 0..10 {
        ra.add_sample(10.0);
        ra.add_sample(10.1);
    }
    
    let mean = ra.sample_mean();
    let interval = ra.confidence_interval_99();
    assert!(interval.is_some());
    
    if let Some((lower, upper)) = interval {
        // Tight distribution should have a narrow confidence interval
        let width = upper - lower;
        assert!(width < 1.0); // Width should be less than 1.0
        assert!(lower <= mean && mean <= upper);
    }
}

#[test]
fn test_add_combines_rolling_averages() {
    let mut ra1 = RollingAverage::new();
    ra1.add_sample(1.0);
    ra1.add_sample(2.0);
    ra1.add_sample(3.0);
    
    let mut ra2 = RollingAverage::new();
    ra2.add_sample(4.0);
    ra2.add_sample(5.0);
    
    ra1.add(ra2);
    
    assert_eq!(ra1.sample_count(), 5);
    assert_eq!(ra1.sample_mean(), 3.0);
}

#[test]
fn test_add_empty_rolling_average() {
    let mut ra1 = RollingAverage::new();
    ra1.add_sample(10.0);
    ra1.add_sample(20.0);
    
    let ra2 = RollingAverage::new();
    
    let original_count = ra1.sample_count();
    let original_mean = ra1.sample_mean();
    
    ra1.add(ra2);
    
    // Should remain unchanged
    assert_eq!(ra1.sample_count(), original_count);
    assert_eq!(ra1.sample_mean(), original_mean);
}

#[test]
fn test_add_to_empty_rolling_average() {
    let mut ra1 = RollingAverage::new();
    
    let mut ra2 = RollingAverage::new();
    ra2.add_sample(10.0);
    ra2.add_sample(20.0);
    
    ra1.add(ra2);
    
    assert_eq!(ra1.sample_count(), 2);
    assert_eq!(ra1.sample_mean(), 15.0);
}

#[test]
fn test_negative_samples() {
    let mut ra = RollingAverage::new();
    ra.add_sample(-5.0);
    ra.add_sample(-10.0);
    ra.add_sample(-15.0);
    
    assert_eq!(ra.sample_count(), 3);
    assert_eq!(ra.sample_mean(), -10.0);
}

#[test]
fn test_mixed_positive_negative_samples() {
    let mut ra = RollingAverage::new();
    ra.add_sample(-10.0);
    ra.add_sample(0.0);
    ra.add_sample(10.0);
    
    assert_eq!(ra.sample_count(), 3);
    assert_eq!(ra.sample_mean(), 0.0);
}

#[test]
fn test_zero_samples() {
    let mut ra = RollingAverage::new();
    ra.add_sample(0.0);
    ra.add_sample(0.0);
    ra.add_sample(0.0);
    
    assert_eq!(ra.sample_count(), 3);
    assert_eq!(ra.sample_mean(), 0.0);
    assert_eq!(ra.sample_variance(), 0.0);
    assert_eq!(ra.sample_std_dev(), 0.0);
}

#[test]
fn test_large_values() {
    let mut ra = RollingAverage::new();
    ra.add_sample(1_000_000.0);
    ra.add_sample(2_000_000.0);
    ra.add_sample(3_000_000.0);
    
    assert_eq!(ra.sample_count(), 3);
    assert_eq!(ra.sample_mean(), 2_000_000.0);
}

#[test]
fn test_very_small_values() {
    let mut ra = RollingAverage::new();
    ra.add_sample(0.0001);
    ra.add_sample(0.0002);
    ra.add_sample(0.0003);
    
    assert_eq!(ra.sample_count(), 3);
    assert!((ra.sample_mean() - 0.0002).abs() < 1e-10);
}

#[test]
fn test_clone() {
    let mut ra1 = RollingAverage::new();
    ra1.add_sample(1.0);
    ra1.add_sample(2.0);
    ra1.add_sample(3.0);
    
    let ra2 = ra1.clone();
    
    assert_eq!(ra2.sample_count(), 3);
    assert_eq!(ra2.sample_mean(), 2.0);
}

#[test]
fn test_sequential_operations() {
    let mut ra = RollingAverage::new();
    
    // Add samples one by one and verify state
    ra.add_sample(10.0);
    assert_eq!(ra.sample_count(), 1);
    assert_eq!(ra.sample_mean(), 10.0);
    
    ra.add_sample(20.0);
    assert_eq!(ra.sample_count(), 2);
    assert_eq!(ra.sample_mean(), 15.0);
    
    ra.add_sample(30.0);
    assert_eq!(ra.sample_count(), 3);
    assert_eq!(ra.sample_mean(), 20.0);
}

#[test]
fn test_variance_formula_correctness() {
    let mut ra = RollingAverage::new();
    let samples = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    
    for &sample in &samples {
        ra.add_sample(sample);
    }
    
    // Expected: mean = 5.0, variance = 4.0
    assert_eq!(ra.sample_mean(), 5.0);
    assert_eq!(ra.sample_variance(), 4.0);
    assert_eq!(ra.sample_std_dev(), 2.0);
}
