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
        assert!(avg.confidence_interval_99().is_none());
        
        // Add samples: [2.0, 4.0, 6.0, 8.0, 10.0]
        avg.add_sample(2.0);
        avg.add_sample(4.0);
        avg.add_sample(6.0);
        avg.add_sample(8.0);
        avg.add_sample(10.0);
        
        assert_eq!(avg.sample_count(), 5);
        assert_eq!(avg.sample_mean(), 6.0); // (2+4+6+8+10)/5 = 30/5 = 6
        
        // Variance calculation: E[X^2] - E[X]^2
        // (4+16+36+64+100)/5 = 220/5 = 44
        // 44 - 36 = 8 (population variance)
        // Sample variance: n/(n-1) * population_variance = 5/4 * 8 = 10
        assert!((avg.sample_variance() - 10.0).abs() < 0.0001);
        
        // Standard deviation = sqrt(10) ≈ 3.162
        assert!((avg.sample_std_dev() - 3.162).abs() < 0.01);
    }

    #[test]
    fn test_rolling_average_single_sample() {
        let mut avg = RollingAverage::new();
        avg.add_sample(5.0);
        
        assert_eq!(avg.sample_count(), 1);
        assert_eq!(avg.sample_mean(), 5.0);
        assert_eq!(avg.sample_variance(), 0.0); // Single sample has no variance
        assert_eq!(avg.sample_std_dev(), 0.0);
        assert!(avg.confidence_interval_99().is_none()); // Need at least 2 samples
    }

    #[test]
    fn test_rolling_average_confidence_interval() {
        let mut avg = RollingAverage::new();
        
        // Add multiple samples to get meaningful confidence interval
        for i in 1..=10 {
            avg.add_sample(i as f64);
        }
        
        let mean = avg.sample_mean();
        assert_eq!(mean, 5.5); // (1+2+...+10)/10 = 55/10 = 5.5
        
        let ci = avg.confidence_interval_99();
        assert!(ci.is_some());
        
        let (lower, upper) = ci.unwrap();
        assert!(lower < mean);
        assert!(upper > mean);
        assert!(lower > 0.0);
        assert!(upper < 11.0);
        
        // The interval should be symmetric around the mean
        let margin = (upper - lower) / 2.0;
        assert!((mean - lower - margin).abs() < 0.0001);
        assert!((upper - mean - margin).abs() < 0.0001);
    }

    #[test]
    fn test_rolling_average_combine() {
        let mut avg1 = RollingAverage::new();
        avg1.add_sample(1.0);
        avg1.add_sample(2.0);
        avg1.add_sample(3.0);
        
        let mut avg2 = RollingAverage::new();
        avg2.add_sample(4.0);
        avg2.add_sample(5.0);
        
        // Combine avg2 into avg1
        avg1.add(avg2);
        
        assert_eq!(avg1.sample_count(), 5);
        assert_eq!(avg1.sample_mean(), 3.0); // (1+2+3+4+5)/5 = 15/5 = 3
        
        // Verify all samples are present
        let variance = avg1.sample_variance();
        assert!(variance > 0.0);
    }
}
