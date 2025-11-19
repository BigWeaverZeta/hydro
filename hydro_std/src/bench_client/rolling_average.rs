use serde::{Deserialize, Serialize};

/// Rolling statistics tracker for computing mean, standard deviation, and confidence intervals
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollingAverage {
    samples: Vec<f64>,
    sum: f64,
    sum_squares: f64,
    count: usize,
}

impl Default for RollingAverage {
    fn default() -> Self {
        Self::new()
    }
}

impl RollingAverage {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            sum: 0.0,
            sum_squares: 0.0,
            count: 0,
        }
    }

    pub fn add_sample(&mut self, value: f64) {
        self.samples.push(value);
        self.sum += value;
        self.sum_squares += value * value;
        self.count += 1;
    }

    pub fn sample_count(&self) -> usize {
        self.count
    }

    pub fn sample_mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    pub fn sample_variance(&self) -> f64 {
        if self.count <= 1 {
            0.0
        } else {
            let mean = self.sample_mean();
            (self.sum_squares - self.count as f64 * mean * mean) / (self.count - 1) as f64
        }
    }

    pub fn sample_std_dev(&self) -> f64 {
        self.sample_variance().sqrt()
    }

    /// Compute 99% confidence interval for the mean using t-distribution approximation
    pub fn confidence_interval_99(&self) -> Option<(f64, f64)> {
        if self.count < 2 {
            return None;
        }

        let mean = self.sample_mean();
        let std_dev = self.sample_std_dev();
        let std_error = std_dev / (self.count as f64).sqrt();

        // t-value for 99% confidence interval (approximation for large n)
        let t_value = 2.576; // z-score for 99% confidence

        let margin = t_value * std_error;
        Some((mean - margin, mean + margin))
    }

    /// Combine two RollingAverage instances
    pub fn add(&mut self, other: Self) {
        for sample in other.samples {
            self.add_sample(sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_average_basic_statistics() {
        let mut avg = RollingAverage::new();
        
        // Test empty state
        assert_eq!(avg.sample_count(), 0);
        assert_eq!(avg.sample_mean(), 0.0);
        assert_eq!(avg.sample_variance(), 0.0);
        assert_eq!(avg.sample_std_dev(), 0.0);
        assert_eq!(avg.confidence_interval_99(), None);
        
        // Add samples
        avg.add_sample(10.0);
        avg.add_sample(20.0);
        avg.add_sample(30.0);
        
        assert_eq!(avg.sample_count(), 3);
        assert_eq!(avg.sample_mean(), 20.0);
        
        // Variance should be 100.0 for [10, 20, 30]
        assert!((avg.sample_variance() - 100.0).abs() < 1e-10);
        assert!((avg.sample_std_dev() - 10.0).abs() < 1e-10);
        
        // Confidence interval should exist
        let ci = avg.confidence_interval_99();
        assert!(ci.is_some());
        let (lower, upper) = ci.unwrap();
        assert!(lower < 20.0);
        assert!(upper > 20.0);
        assert!(lower < upper);
    }

    #[test]
    fn test_rolling_average_single_sample() {
        let mut avg = RollingAverage::new();
        avg.add_sample(42.0);
        
        assert_eq!(avg.sample_count(), 1);
        assert_eq!(avg.sample_mean(), 42.0);
        assert_eq!(avg.sample_variance(), 0.0);
        assert_eq!(avg.sample_std_dev(), 0.0);
        assert_eq!(avg.confidence_interval_99(), None); // Need at least 2 samples
    }

    #[test]
    fn test_rolling_average_merge() {
        let mut avg1 = RollingAverage::new();
        avg1.add_sample(10.0);
        avg1.add_sample(20.0);
        
        let mut avg2 = RollingAverage::new();
        avg2.add_sample(30.0);
        avg2.add_sample(40.0);
        
        avg1.add(avg2);
        
        assert_eq!(avg1.sample_count(), 4);
        assert_eq!(avg1.sample_mean(), 25.0);
    }

    #[test]
    fn test_rolling_average_negative_values() {
        let mut avg = RollingAverage::new();
        avg.add_sample(-10.0);
        avg.add_sample(-5.0);
        avg.add_sample(5.0);
        avg.add_sample(10.0);
        
        assert_eq!(avg.sample_count(), 4);
        assert_eq!(avg.sample_mean(), 0.0);
        assert!(avg.sample_variance() > 0.0);
    }

    #[test]
    fn test_rolling_average_default() {
        let avg = RollingAverage::default();
        assert_eq!(avg.sample_count(), 0);
        assert_eq!(avg.sample_mean(), 0.0);
    }
}
