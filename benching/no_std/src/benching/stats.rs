///! Taken from [rust test crate](https://github.dev/rust-lang/rust/blob/master/library/test/src/stats.rs)
use core::{fmt::Debug, mem};
use num_traits::Float;

fn local_sort<T: Float>(v: &mut [T]) {
    v.sort_unstable_by(|x: &T, y: &T| x.partial_cmp(y).unwrap());
}

/// Trait that provides simple descriptive statistics on a univariate set of numeric samples.
pub trait Stats<T> {
    /// Sum of the samples.
    ///
    /// Note: this method sacrifices performance at the altar of accuracy
    /// Depends on IEEE-754 arithmetic guarantees. See proof of correctness at:
    /// ["Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric
    /// Predicates"][paper]
    ///
    /// [paper]: https://www.cs.cmu.edu/~quake-papers/robust-arithmetic.ps
    fn sum(&self) -> T;

    /// Minimum value of the samples.
    fn min(&self) -> T;

    /// Maximum value of the samples.
    fn max(&self) -> T;

    /// Arithmetic mean (average) of the samples: sum divided by sample-count.
    ///
    /// See: <https://en.wikipedia.org/wiki/Arithmetic_mean>
    fn mean(&self) -> T;

    /// Median of the samples: value separating the lower half of the samples from the higher half.
    /// Equal to `self.percentile(50.0)`.
    ///
    /// See: <https://en.wikipedia.org/wiki/Median>
    fn median(&self) -> T;

    /// Variance of the samples: bias-corrected mean of the squares of the differences of each
    /// sample from the sample mean. Note that this calculates the _sample variance_ rather than the
    /// population variance, which is assumed to be unknown. It therefore corrects the `(n-1)/n`
    /// bias that would appear if we calculated a population variance, by dividing by `(n-1)` rather
    /// than `n`.
    ///
    /// See: <https://en.wikipedia.org/wiki/Variance>
    fn var(&self) -> T;

    /// Standard deviation: the square root of the sample variance.
    ///
    /// Note: this is not a robust statistic for non-normal distributions. Prefer the
    /// `median_abs_dev` for unknown distributions.
    ///
    /// See: <https://en.wikipedia.org/wiki/Standard_deviation>
    fn std_dev(&self) -> T;

    /// Standard deviation as a percent of the mean value. See `std_dev` and `mean`.
    ///
    /// Note: this is not a robust statistic for non-normal distributions. Prefer the
    /// `median_abs_dev_pct` for unknown distributions.
    fn std_dev_pct(&self) -> T;

    /// Scaled median of the absolute deviations of each sample from the sample median. This is a
    /// robust (distribution-agnostic) estimator of sample variability. Use this in preference to
    /// `std_dev` if you cannot assume your sample is normally distributed. Note that this is scaled
    /// by the constant `1.4826` to allow its use as a consistent estimator for the standard
    /// deviation.
    ///
    /// See: <https://en.wikipedia.org/wiki/Median_absolute_deviation>
    fn median_abs_dev(&self) -> T;

    /// Median absolute deviation as a percent of the median. See `median_abs_dev` and `median`.
    fn median_abs_dev_pct(&self) -> T;

    /// Percentile: the value below which `pct` percent of the values in `self` fall. For example,
    /// percentile(95.0) will return the value `v` such that 95% of the samples `s` in `self`
    /// satisfy `s <= v`.
    ///
    /// Calculated by linear interpolation between closest ranks.
    ///
    /// See: <https://en.wikipedia.org/wiki/Percentile>
    fn percentile(&self, pct: T) -> T;

    /// Quartiles of the sample: three values that divide the sample into four equal groups, each
    /// with 1/4 of the data. The middle value is the median. See `median` and `percentile`. This
    /// function may calculate the 3 quartiles more efficiently than 3 calls to `percentile`, but
    /// is otherwise equivalent.
    ///
    /// See also: <https://en.wikipedia.org/wiki/Quartile>
    fn quartiles(&self) -> (T, T, T);

    /// Inter-quartile range: the difference between the 25th percentile (1st quartile) and the 75th
    /// percentile (3rd quartile). See `quartiles`.
    ///
    /// See also: <https://en.wikipedia.org/wiki/Interquartile_range>
    fn iqr(&self) -> T;
}

/// Extracted collection of all the summary statistics of a sample set.
#[derive(Debug, Clone, PartialEq, Copy)]
#[allow(missing_docs)]
pub struct Summary<T> {
    pub sum: T,
    pub min: T,
    pub max: T,
    pub mean: T,
    pub median: T,
    pub var: T,
    pub std_dev: T,
    pub std_dev_pct: T,
    pub median_abs_dev: T,
    pub median_abs_dev_pct: T,
    pub quartiles: (T, T, T),
    pub iqr: T,
}

impl<T> Summary<T>
where
    T: Float + Debug,
{
    /// Construct a new summary of a sample set.
    pub fn new<const N: usize>(samples: &[T; N]) -> Self {
        Self {
            sum: samples.sum(),
            min: samples.min(),
            max: samples.max(),
            mean: samples.mean(),
            median: samples.median(),
            var: samples.var(),
            std_dev: samples.std_dev(),
            std_dev_pct: samples.std_dev_pct(),
            median_abs_dev: samples.median_abs_dev(),
            median_abs_dev_pct: samples.median_abs_dev_pct(),
            quartiles: samples.quartiles(),
            iqr: samples.iqr(),
        }
    }
}

impl<T: Float + Debug, const N: usize> Stats<T> for [T; N] {
    // FIXME #11059 handle NaN, inf and overflow
    fn sum(&self) -> T {
        let mut partials = heapless::Vec::<T, N>::new();

        for &x in self {
            let mut x = x;
            let mut j = 0;
            // This inner loop applies `hi`/`lo` summation to each
            // partial so that the list of partial sums remains exact.
            for i in 0..partials.len() {
                let mut y = partials[i];
                if x.abs() < y.abs() {
                    mem::swap(&mut x, &mut y);
                }
                // Rounded `x+y` is stored in `hi` with round-off stored in
                // `lo`. Together `hi+lo` are exactly equal to `x+y`.
                let hi = x + y;
                let lo = y - (hi - x);
                if lo != T::from(0.0).unwrap() {
                    partials[j] = lo;
                    j += 1;
                }
                x = hi;
            }
            if j >= partials.len() {
                partials.extend_one(x);
            } else {
                partials[j] = x;
                partials.extend_one(T::from(j + 1).unwrap());
            }
        }
        let zero = T::zero();
        partials.iter().fold(zero, |p, q| p + *q)
    }

    fn min(&self) -> T {
        assert!(!self.is_empty());
        self.iter().fold(self[0], |p, q| p.min(*q))
    }

    fn max(&self) -> T {
        assert!(!self.is_empty());
        self.iter().fold(self[0], |p, q| p.max(*q))
    }

    fn mean(&self) -> T {
        assert!(!self.is_empty());
        self.sum() / T::from(self.len()).unwrap()
    }

    fn median(&self) -> T {
        self.percentile(T::from(50.0).unwrap())
    }

    fn var(&self) -> T {
        if self.len() < 2 {
            T::zero()
        } else {
            let mean = self.mean();
            let mut v = T::zero();
            for s in self {
                let x = *s - mean;
                v = v + x * x;
            }
            // N.B., this is _supposed to be_ len-1, not len. If you
            // change it back to len, you will be calculating a
            // population variance, not a sample variance.
            let denom = T::from(self.len() - 1).unwrap();
            v / denom
        }
    }

    fn std_dev(&self) -> T {
        self.var().sqrt()
    }

    fn std_dev_pct(&self) -> T {
        let hundred = T::from(100.0).unwrap();
        (self.std_dev() / self.mean()) * hundred
    }

    fn median_abs_dev(&self) -> T {
        let med = self.median();
        let abs_devs = self
            .iter()
            .map(|&v| med - v)
            .collect::<heapless::Vec<T, N>>();
        // This constant is derived by smarter statistics brains than me, but it is
        // consistent with how R and other packages treat the MAD.
        let number = T::from(1.4826).unwrap();
        abs_devs.into_array::<N>().unwrap().median() * number
    }

    fn median_abs_dev_pct(&self) -> T {
        let hundred = T::from(100.0).unwrap();
        (self.median_abs_dev() / self.median()) * hundred
    }

    fn percentile(&self, pct: T) -> T {
        let mut tmp = heapless::Vec::<T, N>::from_slice(self).unwrap();
        local_sort(&mut tmp);
        percentile_of_sorted(&tmp, pct)
    }

    fn quartiles(&self) -> (T, T, T) {
        let mut tmp = heapless::Vec::<T, N>::from_slice(self).unwrap();
        local_sort(&mut tmp);
        let first = T::from(25.0).unwrap();
        let a = percentile_of_sorted(tmp.as_slice(), first);
        let second = T::from(50.0).unwrap();
        let b = percentile_of_sorted(&tmp, second);
        let third = T::from(75.0).unwrap();
        let c = percentile_of_sorted(&tmp, third);
        (a, b, c)
    }

    fn iqr(&self) -> T {
        let (a, _, c) = self.quartiles();
        c - a
    }
}

// Helper function: extract a value representing the `pct` percentile of a sorted sample-set, using
// linear interpolation. If samples are not sorted, return nonsensical value.
fn percentile_of_sorted<T: Float>(sorted_samples: &[T], pct: T) -> T {
    assert!(!sorted_samples.is_empty());
    if sorted_samples.len() == 1 {
        return sorted_samples[0];
    }
    let zero = T::zero();
    assert!(zero <= pct);
    let hundred = T::from(100.0).unwrap();
    assert!(pct <= hundred);
    if pct == hundred {
        return sorted_samples[sorted_samples.len() - 1];
    }
    let length = T::from(sorted_samples.len() - 1).unwrap();
    let rank = (pct / hundred) * length;
    let lrank = rank.floor();
    let d = rank - lrank;
    let n = lrank.to_usize().unwrap();
    let lo = sorted_samples[n];
    let hi = sorted_samples[n + 1];
    lo + (hi - lo) * d
}

/// Winsorize a set of samples, replacing values above the `100-pct` percentile
/// and below the `pct` percentile with those percentiles themselves. This is a
/// way of minimizing the effect of outliers, at the cost of biasing the sample.
/// It differs from trimming in that it does not change the number of samples,
/// just changes the values of those that are outliers.
///
/// See: <https://en.wikipedia.org/wiki/Winsorising>
pub fn winsorize<T: Float>(samples: &mut [T], pct: T) {
    let mut tmp = samples.to_vec();
    local_sort(&mut tmp);
    let lo = percentile_of_sorted(&tmp, pct);
    let hundred = T::from(100.0).unwrap();
    let hi = percentile_of_sorted(&tmp, hundred - pct);
    for samp in samples {
        if *samp > hi {
            *samp = hi
        } else if *samp < lo {
            *samp = lo
        }
    }
}
