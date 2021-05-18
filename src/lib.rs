use std::iter::FromIterator;

use statrs::distribution::{ContinuousCDF, Normal, StudentsT};

#[derive(Copy, Clone, Debug)]
pub struct Difference {
    pub effect: f64,
    pub effect_size: f64,
    pub critical_value: f64,
    pub p_value: f64,
    pub alpha: f64,
    pub beta: f64,
}

impl Difference {
    pub fn is_significant(&self) -> bool {
        self.effect > self.critical_value
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Summary {
    pub n: f64,
    pub mean: f64,
    pub variance: f64,
}

impl<'a> FromIterator<&'a f64> for Summary {
    fn from_iter<T: IntoIterator<Item = &'a f64>>(iter: T) -> Self {
        // Welford algorithm for corrected variance
        let mut mean = 0.0;
        let mut m2 = 0.0;
        let mut n = 0.0;
        for x in iter {
            n += 1.0;
            let delta = x - mean;
            mean += delta / n;
            m2 += delta * (x - mean);
        }
        Summary {
            n,
            mean,
            variance: m2 / (n - 1.0), // Bessel's correction
        }
    }
}

impl Summary {
    pub fn compare(&self, other: &Summary, confidence: f64) -> Difference {
        assert!(
            (0.0..100.0).contains(&confidence),
            "confidence must be [0,100)"
        );

        let (a, b) = (self, other);

        // Calculate the significance level.
        let alpha = 1.0 - (confidence / 100.0);

        // Calculate the degrees of freedom.
        let nu = (a.variance / a.n + b.variance / b.n).powf(2.0)
            / ((a.variance).powf(2.0) / ((a.n).powf(2.0) * (a.n - 1.0))
                + (b.variance).powf(2.0) / ((b.n).powf(2.0) * (b.n - 1.0)));

        // Create a Student's T distribution with location of 0, a scale of 1, and the same number
        // of degrees of freedom as in the test.
        let dist_st = StudentsT::new(0.0, 1.0, nu).unwrap();

        // Calculate the hypothetical two-tailed t-value for the given significance level.
        let t_hyp = dist_st.inverse_cdf(1.0 - (alpha / TAILS));

        // Calculate the absolute difference between the means of the two samples.
        let effect = (a.mean - b.mean).abs();

        // Calculate the standard error.
        let std_err = (a.variance / a.n + b.variance / b.n).sqrt();

        // Calculate the experimental t-value.
        let t_exp = effect / std_err;

        // Calculate the p-value given the experimental t-value.
        let p_value = dist_st.cdf(-t_exp) * TAILS;

        // Calculate the critical value.
        let critical_value = t_hyp * std_err;

        // Calculate the standard deviation using mean variance.
        let std_dev = ((a.variance + b.variance) / 2.0).sqrt();

        // Calculate Cohen's d for the effect size.
        let effect_size = effect / std_dev;

        // Create a unit normal distribution.
        let dist_norm = Normal::new(0.0, 1.0).unwrap();

        // Calculate the statistical power.
        let z = effect / (std_dev * (1.0 / a.n + 1.0 / b.n).sqrt());
        let za = dist_norm.inverse_cdf(1.0 - alpha / TAILS);
        let beta = dist_norm.cdf(z - za) - dist_norm.cdf(-z - za);

        Difference {
            effect,
            effect_size,
            critical_value,
            p_value,
            alpha,
            beta,
        }
    }
}

const TAILS: f64 = 2.0;

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::*;

    #[test]
    fn test_summarize_odd() {
        let s: Summary = vec![1.0, 2.0, 3.0].iter().collect();

        assert_relative_eq!(s.n, 3.0);
        assert_relative_eq!(s.mean, 2.0);
        assert_relative_eq!(s.variance, 1.0);
    }

    #[test]
    fn test_summarize_even() {
        let s: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();

        assert_relative_eq!(s.n, 4.0);
        assert_relative_eq!(s.mean, 2.5);
        assert_relative_eq!(s.variance, 1.6666666666666667);
    }

    #[test]
    fn test_compare_similar_data() {
        let a: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();
        let b: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();
        let diff = a.compare(&b, 80.0);

        assert_relative_eq!(diff.effect, 0.0);
        assert_relative_eq!(diff.effect_size, 0.0);
        assert_relative_eq!(diff.critical_value, 1.3143111667913936);
        assert_relative_eq!(diff.p_value, 1.0);
        assert_relative_eq!(diff.alpha, 0.19999999999999996);
        assert_relative_eq!(diff.beta, 0.0);
        assert_eq!(diff.is_significant(), false);
    }

    #[test]
    fn test_compare_different_data() {
        let a: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();
        let b: Summary = vec![10.0, 20.0, 30.0, 40.0].iter().collect();
        let diff = a.compare(&b, 80.0);

        assert_relative_eq!(diff.effect, 22.5);
        assert_relative_eq!(diff.effect_size, 2.452519415855564);
        assert_relative_eq!(diff.critical_value, 10.568344341563591);
        assert_relative_eq!(diff.p_value, 0.03916791618893325);
        assert_relative_eq!(diff.alpha, 0.19999999999999996);
        assert_relative_eq!(diff.beta, 0.985621684277956);
        assert_eq!(diff.is_significant(), true);
    }
}
