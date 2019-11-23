use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InvalidConfidenceError(String);

impl fmt::Display for InvalidConfidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid confidence level: {}", self.0)
    }
}

impl Error for InvalidConfidenceError {}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum Confidence {
    P80,
    P90,
    P95,
    P98,
    P99,
    P995,
}

impl FromStr for Confidence {
    type Err = InvalidConfidenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "P80" => Ok(Confidence::P80),
            "P90" => Ok(Confidence::P90),
            "P95" => Ok(Confidence::P95),
            "P98" => Ok(Confidence::P98),
            "P99" => Ok(Confidence::P99),
            "P999" => Ok(Confidence::P99),
            _ => Err(InvalidConfidenceError(value.to_string())),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Difference {
    pub delta: f64,
    pub error: f64,
    pub rel_delta: f64,
    pub rel_error: f64,
    pub std_dev: f64,
    pub confidence: Confidence,
}

impl Difference {
    pub fn is_significant(&self) -> bool {
        self.delta > self.error
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Summary {
    pub n: f64,
    pub mean: f64,
    pub variance: f64,
}

impl Summary {
    pub fn of(values: &[f64]) -> Summary {
        // Welford algorithm for corrected variance
        let mut mean = 0.0;
        let mut m2 = 0.0;
        let mut n = 0.0;
        for x in values {
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

    pub fn compare(&self, other: &Summary, confidence: Confidence) -> Difference {
        let a = self.n - 1.0;
        let b = other.n - 1.0;
        let d_of_f = a + b;
        let d_of_f_idx = if d_of_f as usize > STUDENT.len() {
            0
        } else {
            d_of_f as usize
        };
        let t = STUDENT[d_of_f_idx][confidence as usize];
        let std_dev = ((a * self.variance + b * other.variance) / d_of_f).sqrt();
        let delta = (self.mean - other.mean).abs();
        let error = t * std_dev * (self.n.recip() + other.n.recip()).sqrt();
        Difference {
            delta,
            error,
            rel_delta: delta / self.mean,
            rel_error: error / self.mean,
            std_dev,
            confidence,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_summarize_odd() {
        let s = Summary::of(&vec![1.0, 2.0, 3.0]);

        assert_eq!(s.n, 3.0, "n");
        assert_eq!(s.mean, 2.0, "mean");
        assert_eq!(s.variance, 1.0, "variance");
    }

    #[test]
    fn test_summarize_even() {
        let s = Summary::of(&vec![1.0, 2.0, 3.0, 4.0]);

        assert_eq!(s.n, 4.0, "n");
        assert_eq!(s.mean, 2.5, "mean");
        assert_eq!(s.variance, 1.6666666666666667, "variance");
    }

    #[test]
    fn test_compare_similar_data() {
        let a = Summary::of(&vec![1.0, 2.0, 3.0, 4.0]);
        let b = Summary::of(&vec![1.0, 2.0, 3.0, 4.0]);
        let diff = a.compare(&b, Confidence::P80);

        assert_eq!(diff.delta, 0.0, "delta");
        assert_eq!(diff.error, 1.3145341380123987, "error");
        assert_eq!(diff.rel_delta, 0.0, "rel_delta");
        assert_eq!(diff.rel_error, 0.5258136552049595, "rel_error");
        assert_eq!(diff.std_dev, 1.2909944487358056, "std_dev");
        assert!(!diff.is_significant(), "significant");
    }

    #[test]
    fn test_compare_different_data() {
        let a = Summary::of(&vec![1.0, 2.0, 3.0, 4.0]);
        let b = Summary::of(&vec![10.0, 20.0, 30.0, 40.0]);
        let diff = a.compare(&b, Confidence::P80);

        assert_eq!(diff.delta, 22.5, "delta");
        assert_eq!(diff.error, 9.341520218893711, "error");
        assert_eq!(diff.rel_delta, 9.0, "rel_delta");
        assert_eq!(diff.rel_error, 3.7366080875574843, "rel_error");
        assert_eq!(diff.std_dev, 9.17423929634859, "std_dev");
        assert!(diff.is_significant(), "significant");
    }

    #[test]
    fn test_confidence() {
        assert_eq!(Confidence::P80 as usize, 0);
        assert_eq!(Confidence::P90 as usize, 1);
        assert_eq!(Confidence::P95 as usize, 2);
        assert_eq!(Confidence::P98 as usize, 3);
        assert_eq!(Confidence::P99 as usize, 4);
        assert_eq!(Confidence::P995 as usize, 5);
    }
}

/// Student's T critical values for two-tailed tests at α=0.2, 0.1, 0.05, 0.01, and 0.005, for
/// degrees of freedom from 1 to 100. 0th row is the normal approximation for dof>100.
#[allow(clippy::approx_constant)]
static STUDENT: [[f64; (Confidence::P995 as usize) + 1]; 101] = [
    /*   ∞. */ [1.282, 1.645, 1.960, 2.326, 2.576, 3.090],
    /*   1. */ [3.078, 6.314, 12.706, 31.821, 63.657, 318.313],
    /*   2. */ [1.886, 2.920, 4.303, 6.965, 9.925, 22.327],
    /*   3. */ [1.638, 2.353, 3.182, 4.541, 5.841, 10.215],
    /*   4. */ [1.533, 2.132, 2.776, 3.747, 4.604, 7.173],
    /*   5. */ [1.476, 2.015, 2.571, 3.365, 4.032, 5.893],
    /*   6. */ [1.440, 1.943, 2.447, 3.143, 3.707, 5.208],
    /*   7. */ [1.415, 1.895, 2.365, 2.998, 3.499, 4.782],
    /*   8. */ [1.397, 1.860, 2.306, 2.896, 3.355, 4.499],
    /*   9. */ [1.383, 1.833, 2.262, 2.821, 3.250, 4.296],
    /*  10. */ [1.372, 1.812, 2.228, 2.764, 3.169, 4.143],
    /*  11. */ [1.363, 1.796, 2.201, 2.718, 3.106, 4.024],
    /*  12. */ [1.356, 1.782, 2.179, 2.681, 3.055, 3.929],
    /*  13. */ [1.350, 1.771, 2.160, 2.650, 3.012, 3.852],
    /*  14. */ [1.345, 1.761, 2.145, 2.624, 2.977, 3.787],
    /*  15. */ [1.341, 1.753, 2.131, 2.602, 2.947, 3.733],
    /*  16. */ [1.337, 1.746, 2.120, 2.583, 2.921, 3.686],
    /*  17. */ [1.333, 1.740, 2.110, 2.567, 2.898, 3.646],
    /*  18. */ [1.330, 1.734, 2.101, 2.552, 2.878, 3.610],
    /*  19. */ [1.328, 1.729, 2.093, 2.539, 2.861, 3.579],
    /*  20. */ [1.325, 1.725, 2.086, 2.528, 2.845, 3.552],
    /*  21. */ [1.323, 1.721, 2.080, 2.518, 2.831, 3.527],
    /*  22. */ [1.321, 1.717, 2.074, 2.508, 2.819, 3.505],
    /*  23. */ [1.319, 1.714, 2.069, 2.500, 2.807, 3.485],
    /*  24. */ [1.318, 1.711, 2.064, 2.492, 2.797, 3.467],
    /*  25. */ [1.316, 1.708, 2.060, 2.485, 2.787, 3.450],
    /*  26. */ [1.315, 1.706, 2.056, 2.479, 2.779, 3.435],
    /*  27. */ [1.314, 1.703, 2.052, 2.473, 2.771, 3.421],
    /*  28. */ [1.313, 1.701, 2.048, 2.467, 2.763, 3.408],
    /*  29. */ [1.311, 1.699, 2.045, 2.462, 2.756, 3.396],
    /*  30. */ [1.310, 1.697, 2.042, 2.457, 2.750, 3.385],
    /*  31. */ [1.309, 1.696, 2.040, 2.453, 2.744, 3.375],
    /*  32. */ [1.309, 1.694, 2.037, 2.449, 2.738, 3.365],
    /*  33. */ [1.308, 1.692, 2.035, 2.445, 2.733, 3.356],
    /*  34. */ [1.307, 1.691, 2.032, 2.441, 2.728, 3.348],
    /*  35. */ [1.306, 1.690, 2.030, 2.438, 2.724, 3.340],
    /*  36. */ [1.306, 1.688, 2.028, 2.434, 2.719, 3.333],
    /*  37. */ [1.305, 1.687, 2.026, 2.431, 2.715, 3.326],
    /*  38. */ [1.304, 1.686, 2.024, 2.429, 2.712, 3.319],
    /*  39. */ [1.304, 1.685, 2.023, 2.426, 2.708, 3.313],
    /*  40. */ [1.303, 1.684, 2.021, 2.423, 2.704, 3.307],
    /*  41. */ [1.303, 1.683, 2.020, 2.421, 2.701, 3.301],
    /*  42. */ [1.302, 1.682, 2.018, 2.418, 2.698, 3.296],
    /*  43. */ [1.302, 1.681, 2.017, 2.416, 2.695, 3.291],
    /*  44. */ [1.301, 1.680, 2.015, 2.414, 2.692, 3.286],
    /*  45. */ [1.301, 1.679, 2.014, 2.412, 2.690, 3.281],
    /*  46. */ [1.300, 1.679, 2.013, 2.410, 2.687, 3.277],
    /*  47. */ [1.300, 1.678, 2.012, 2.408, 2.685, 3.273],
    /*  48. */ [1.299, 1.677, 2.011, 2.407, 2.682, 3.269],
    /*  49. */ [1.299, 1.677, 2.010, 2.405, 2.680, 3.265],
    /*  50. */ [1.299, 1.676, 2.009, 2.403, 2.678, 3.261],
    /*  51. */ [1.298, 1.675, 2.008, 2.402, 2.676, 3.258],
    /*  52. */ [1.298, 1.675, 2.007, 2.400, 2.674, 3.255],
    /*  53. */ [1.298, 1.674, 2.006, 2.399, 2.672, 3.251],
    /*  54. */ [1.297, 1.674, 2.005, 2.397, 2.670, 3.248],
    /*  55. */ [1.297, 1.673, 2.004, 2.396, 2.668, 3.245],
    /*  56. */ [1.297, 1.673, 2.003, 2.395, 2.667, 3.242],
    /*  57. */ [1.297, 1.672, 2.002, 2.394, 2.665, 3.239],
    /*  58. */ [1.296, 1.672, 2.002, 2.392, 2.663, 3.237],
    /*  59. */ [1.296, 1.671, 2.001, 2.391, 2.662, 3.234],
    /*  60. */ [1.296, 1.671, 2.000, 2.390, 2.660, 3.232],
    /*  61. */ [1.296, 1.670, 2.000, 2.389, 2.659, 3.229],
    /*  62. */ [1.295, 1.670, 1.999, 2.388, 2.657, 3.227],
    /*  63. */ [1.295, 1.669, 1.998, 2.387, 2.656, 3.225],
    /*  64. */ [1.295, 1.669, 1.998, 2.386, 2.655, 3.223],
    /*  65. */ [1.295, 1.669, 1.997, 2.385, 2.654, 3.220],
    /*  66. */ [1.295, 1.668, 1.997, 2.384, 2.652, 3.218],
    /*  67. */ [1.294, 1.668, 1.996, 2.383, 2.651, 3.216],
    /*  68. */ [1.294, 1.668, 1.995, 2.382, 2.650, 3.214],
    /*  69. */ [1.294, 1.667, 1.995, 2.382, 2.649, 3.213],
    /*  70. */ [1.294, 1.667, 1.994, 2.381, 2.648, 3.211],
    /*  71. */ [1.294, 1.667, 1.994, 2.380, 2.647, 3.209],
    /*  72. */ [1.293, 1.666, 1.993, 2.379, 2.646, 3.207],
    /*  73. */ [1.293, 1.666, 1.993, 2.379, 2.645, 3.206],
    /*  74. */ [1.293, 1.666, 1.993, 2.378, 2.644, 3.204],
    /*  75. */ [1.293, 1.665, 1.992, 2.377, 2.643, 3.202],
    /*  76. */ [1.293, 1.665, 1.992, 2.376, 2.642, 3.201],
    /*  77. */ [1.293, 1.665, 1.991, 2.376, 2.641, 3.199],
    /*  78. */ [1.292, 1.665, 1.991, 2.375, 2.640, 3.198],
    /*  79. */ [1.292, 1.664, 1.990, 2.374, 2.640, 3.197],
    /*  80. */ [1.292, 1.664, 1.990, 2.374, 2.639, 3.195],
    /*  81. */ [1.292, 1.664, 1.990, 2.373, 2.638, 3.194],
    /*  82. */ [1.292, 1.664, 1.989, 2.373, 2.637, 3.193],
    /*  83. */ [1.292, 1.663, 1.989, 2.372, 2.636, 3.191],
    /*  84. */ [1.292, 1.663, 1.989, 2.372, 2.636, 3.190],
    /*  85. */ [1.292, 1.663, 1.988, 2.371, 2.635, 3.189],
    /*  86. */ [1.291, 1.663, 1.988, 2.370, 2.634, 3.188],
    /*  87. */ [1.291, 1.663, 1.988, 2.370, 2.634, 3.187],
    /*  88. */ [1.291, 1.662, 1.987, 2.369, 2.633, 3.185],
    /*  89. */ [1.291, 1.662, 1.987, 2.369, 2.632, 3.184],
    /*  90. */ [1.291, 1.662, 1.987, 2.368, 2.632, 3.183],
    /*  91. */ [1.291, 1.662, 1.986, 2.368, 2.631, 3.182],
    /*  92. */ [1.291, 1.662, 1.986, 2.368, 2.630, 3.181],
    /*  93. */ [1.291, 1.661, 1.986, 2.367, 2.630, 3.180],
    /*  94. */ [1.291, 1.661, 1.986, 2.367, 2.629, 3.179],
    /*  95. */ [1.291, 1.661, 1.985, 2.366, 2.629, 3.178],
    /*  96. */ [1.290, 1.661, 1.985, 2.366, 2.628, 3.177],
    /*  97. */ [1.290, 1.661, 1.985, 2.365, 2.627, 3.176],
    /*  98. */ [1.290, 1.661, 1.984, 2.365, 2.627, 3.175],
    /*  99. */ [1.290, 1.660, 1.984, 2.365, 2.626, 3.175],
    /* 100. */ [1.290, 1.660, 1.984, 2.364, 2.626, 3.174],
];
