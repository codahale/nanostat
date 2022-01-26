//! nanostat compares data sets using Welch's t-test at various levels of confidence.

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications,
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::cognitive_complexity,
    clippy::missing_const_for_fn,
    clippy::needless_borrow
)]

use std::iter::FromIterator;

use statrs::distribution::{ContinuousCDF, Normal, StudentsT};

/// The statistical difference between two [Summary] instances.
#[derive(Copy, Clone, Debug)]
pub struct Difference {
    /// The absolute difference between the samples' means.
    pub effect: f64,

    /// The difference in means between the two samples, normalized for variance. Technically, this
    /// is Cohen's d.
    pub effect_size: f64,

    /// The minimum allowed effect at the given confidence level.
    pub critical_value: f64,

    /// The p-value for the test: the probability that accepting the results of this test will be a
    /// Type 1 error, in which the null hypothesis (i.e. there is no difference between the means of
    /// the two samples) will be rejected when it is in fact true.
    pub p_value: f64,

    /// The significance level of the test. It is the maximum allowed value of the p-value.
    pub alpha: f64,

    /// The probability of a Type 2 error: the probability that the null hypothesis will be retained
    /// despite it not being true.
    pub beta: f64,
}

impl Difference {
    /// Whether or not the difference is statistically significant.
    #[must_use]
    pub fn is_significant(&self) -> bool {
        self.effect > self.critical_value
    }
}

/// A statistical summary of a normally distributed data set.
///
/// Created from an iterable of `f64`s:
///
/// ```
/// let summary: nanostat::Summary = vec![0.1, 0.45, 0.42].iter().collect();
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Summary {
    /// The number of measurements in the set.
    pub n: f64,
    /// The arithmetic mean of the measurements.
    pub mean: f64,
    /// The sample variance of the data set.
    pub variance: f64,
}

impl<'a> FromIterator<&'a f64> for Summary {
    fn from_iter<T: IntoIterator<Item = &'a f64>>(iter: T) -> Self {
        // Welford's one-pass algorithm for corrected variance
        let (mut mean, mut s, mut n) = (0.0, 0.0, 0.0);
        for x in iter {
            n += 1.0;
            let delta = x - mean;
            mean += delta / n;
            s += delta * (x - mean);
        }
        let variance = s / (n - 1.0); // Bessel's correction
        Summary { n, mean, variance }
    }
}

impl Summary {
    /// The standard deviation of the sample.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }

    /// The standard error of the sample.
    #[must_use]
    pub fn std_err(&self) -> f64 {
        self.std_dev() / self.n.sqrt()
    }

    /// Calculate the statistical difference between the two summaries using a two-tailed Welch's
    /// t-test. The confidence level must be in the range `(0, 100)`.
    #[must_use]
    pub fn compare(&self, other: &Summary, confidence: f64) -> Difference {
        assert!(0.0 < confidence && confidence < 100.0, "confidence must be (0,100)");

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

        // Calculate the statistical power.
        let z = effect / (std_dev * (1.0 / a.n + 1.0 / b.n).sqrt());
        let dist_norm = Normal::new(0.0, 1.0).unwrap();
        let za = dist_norm.inverse_cdf(1.0 - alpha / TAILS);
        let beta = dist_norm.cdf(z - za) - dist_norm.cdf(-z - za);

        Difference { effect, effect_size, critical_value, p_value, alpha, beta }
    }
}

/// The number of distribution tails used to determine significance. In this case, we always use a
/// two-tailed test because our null hypothesis is that the samples are not different.
const TAILS: f64 = 2.0;

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn summarize_odd() {
        let s: Summary = vec![1.0, 2.0, 3.0].iter().collect();

        assert_relative_eq!(s.n, 3.0);
        assert_relative_eq!(s.mean, 2.0);
        assert_relative_eq!(s.variance, 1.0);
    }

    #[test]
    fn summarize_even() {
        let s: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();

        assert_relative_eq!(s.n, 4.0);
        assert_relative_eq!(s.mean, 2.5);
        assert_relative_eq!(s.variance, 1.6666666666666667);
    }

    #[test]
    fn compare_similar_data() {
        let a: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();
        let b: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();
        let diff = a.compare(&b, 80.0);

        assert_relative_eq!(diff.effect, 0.0);
        assert_relative_eq!(diff.effect_size, 0.0);
        assert_relative_eq!(diff.critical_value, 1.3143111667913936);
        assert_relative_eq!(diff.p_value, 1.0);
        assert_relative_eq!(diff.alpha, 0.19999999999999996);
        assert_relative_eq!(diff.beta, 0.0);
        assert!(!diff.is_significant());
    }

    #[test]
    fn compare_different_data() {
        let a: Summary = vec![1.0, 2.0, 3.0, 4.0].iter().collect();
        let b: Summary = vec![10.0, 20.0, 30.0, 40.0].iter().collect();
        let diff = a.compare(&b, 80.0);

        assert_relative_eq!(diff.effect, 22.5);
        assert_relative_eq!(diff.effect_size, 2.452519415855564);
        assert_relative_eq!(diff.critical_value, 10.568344341563591);
        assert_relative_eq!(diff.p_value, 0.03916791618893325);
        assert_relative_eq!(diff.alpha, 0.19999999999999996);
        assert_relative_eq!(diff.beta, 0.985621684277956);
        assert!(diff.is_significant());
    }
}
