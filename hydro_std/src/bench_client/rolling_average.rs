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
    fn test_basic_statistics() {
        let mut avg = RollingAverage::new();

        // Add samples
        avg.add_sample(10.0);
        avg.add_sample(20.0);
        avg.add_sample(30.0);
        avg.add_sample(40.0);
        avg.add_sample(50.0);

        // Verify count
        assert_eq!(avg.sample_count(), 5);

        // Verify mean: (10 + 20 + 30 + 40 + 50) / 5 = 30
        assert!((avg.sample_mean() - 30.0).abs() < 1e-10);

        // Verify variance: sum((x - mean)^2) / (n - 1)
        // ((-20)^2 + (-10)^2 + 0^2 + 10^2 + 20^2) / 4 = 1000 / 4 = 250
        assert!((avg.sample_variance() - 250.0).abs() < 1e-10);

        // Verify standard deviation: sqrt(250) ≈ 15.811
        assert!((avg.sample_std_dev() - 15.811388300841896).abs() < 1e-10);
    }

    #[test]
    fn test_edge_cases() {
        // Test empty rolling average
        let empty_avg = RollingAverage::new();
        assert_eq!(empty_avg.sample_count(), 0);
        assert_eq!(empty_avg.sample_mean(), 0.0);
        assert_eq!(empty_avg.sample_variance(), 0.0);
        assert_eq!(empty_avg.sample_std_dev(), 0.0);
        assert!(empty_avg.confidence_interval_99().is_none());

        // Test single sample
        let mut single_avg = RollingAverage::new();
        single_avg.add_sample(42.0);
        assert_eq!(single_avg.sample_count(), 1);
        assert_eq!(single_avg.sample_mean(), 42.0);
        assert_eq!(single_avg.sample_variance(), 0.0);
        assert_eq!(single_avg.sample_std_dev(), 0.0);
        assert!(single_avg.confidence_interval_99().is_none());

        // Test with negative values
        let mut neg_avg = RollingAverage::new();
        neg_avg.add_sample(-10.0);
        neg_avg.add_sample(-20.0);
        neg_avg.add_sample(-30.0);
        assert_eq!(neg_avg.sample_count(), 3);
        assert_eq!(neg_avg.sample_mean(), -20.0);
        assert!(neg_avg.sample_variance() > 0.0);
    }

    #[test]
    fn test_combine_averages() {
        let mut avg1 = RollingAverage::new();
        avg1.add_sample(10.0);
        avg1.add_sample(20.0);
        avg1.add_sample(30.0);

        let mut avg2 = RollingAverage::new();
        avg2.add_sample(40.0);
        avg2.add_sample(50.0);

        // Combine averages
        avg1.add(avg2);

        // Should have all 5 samples
        assert_eq!(avg1.sample_count(), 5);
        assert_eq!(avg1.sample_mean(), 30.0);
    }

    #[test]
    fn test_confidence_interval() {
        let mut avg = RollingAverage::new();

        // Add samples with known distribution
        for i in 1..=10 {
            avg.add_sample(i as f64);
        }

        // Verify confidence interval exists
        let ci = avg.confidence_interval_99();
        assert!(ci.is_some());

        let (lower, upper) = ci.unwrap();

        // Mean should be 5.5
        let mean = avg.sample_mean();
        assert!((mean - 5.5).abs() < 1e-10);

        // Confidence interval should contain the mean
        assert!(lower < mean);
        assert!(upper > mean);

        // Confidence interval should be symmetric around the mean
        assert!((mean - lower - (upper - mean)).abs() < 1e-10);
    }
}
