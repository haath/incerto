use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    fmt::{Debug, Display},
};

use bevy::prelude::Deref;

use crate::{SampleAggregate, prelude::*};

/// Utility aggregator that fetches the minimum value.
///
/// Implemented automatically for any numeric type `T` where a [`Sample<T>`]
/// implementation also exist.
///
/// ```ignore
/// let min = simulation.sample_aggregate::<MyComponent, Option<Minimum<f32>>>().unwrap();
/// ```
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Minimum<T>(T);

/// Utility aggregator that fetches the maximum value.
///
/// Implemented automatically for any numeric type `T` where a [`Sample<T>`]
/// implementation also exist.
///
/// ```ignore
/// let max = simulation.sample_aggregate::<MyComponent, Option<Maximum<f32>>>().unwrap();
/// ```
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Maximum<T>(T);

/// Utility aggregator that computes the median value.
///
/// Implemented automatically for any numeric type `T` such as [`i16`], [`f32`], etc.
///
/// ```ignore
/// let median = simulation.sample_aggregate::<MyComponent, Option<Median<f32>>>().unwrap();
/// ```
///
/// Note that computing float medians will panic if any of the samples being aggregated are not
/// comparable (e.g `NaN`).
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Median<T>(T);

/// Utility aggregator that computes the mean value.
///
/// Implemented automatically for any numeric type `T` such as [`i16`], [`f32`], etc.
///
/// ```ignore
/// let mean = simulation.sample_aggregate::<MyComponent, Option<Mean<f32>>>().unwrap();
/// ```
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Mean<T>(T);

/// Utility aggregator that computes the **P-th percentile** value.
///
/// The percentile is selected via the parameter `P`.
/// The aggregated value is the one, where `P%` of all samples are at or below it.
///
/// ```ignore
/// let tength_percentile = simulation.sample_aggregate::<MyComponent, Option<Percentile<f32, 10>>>().unwrap();
/// ```
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Percentile<T, const P: u8>(T);

// ===========================================================
//              Blanket implementations
// ===========================================================
impl<T, O> SampleAggregate<Option<Minimum<O>>> for T
where
    T: Sample<O>,
    O: PartialOrd + Copy,
{
    fn sample_aggregate(components: &[&Self]) -> Option<Minimum<O>>
    {
        components
            .iter()
            .map(|&c| Sample::sample(c))
            .fold(None, |acc: Option<O>, v| {
                acc.map_or(Some(v), |cur_min| match v.partial_cmp(&cur_min)
                {
                    Some(Ordering::Less) => Some(v),
                    _ => Some(cur_min),
                })
            })
            .map(|min| Minimum(min))
    }
}

impl<T, O> SampleAggregate<Option<Maximum<O>>> for T
where
    T: Sample<O>,
    O: PartialOrd + Copy,
{
    fn sample_aggregate(components: &[&Self]) -> Option<Maximum<O>>
    {
        components
            .iter()
            .map(|&c| Sample::sample(c))
            .fold(None, |acc: Option<O>, v| {
                acc.map_or(Some(v), |cur_max| match v.partial_cmp(&cur_max)
                {
                    Some(Ordering::Greater) => Some(v),
                    _ => Some(cur_max),
                })
            })
            .map(|max| Maximum(max))
    }
}

impl<O, const P: u8> Percentile<O, P>
{
    const PERCENTAGE: f64 = (P as f64) / 100.0;
    const _ASSERT: () = assert!(P <= 100, "percentile must be between 0 and 100");
}
impl<T, O, const P: u8> SampleAggregate<Option<Percentile<O, P>>> for T
where
    T: Sample<O>,
    O: PartialEq + PartialOrd + Copy + Display + Debug,
{
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn sample_aggregate(components: &[&Self]) -> Option<Percentile<O, P>>
    {
        if components.is_empty()
        {
            return None;
        }

        let sorted: BinaryHeap<_> = components
            .iter()
            .map(|&c| Sample::sample(c))
            .map(|v| sealed::Ordered(v))
            .collect();
        let sorted = sorted.into_sorted_vec();

        let idx = (Percentile::<O, P>::PERCENTAGE * sorted.len() as f64).floor();
        let value = sorted[idx as usize];
        Some(Percentile(*value))
    }
}

impl<T, O> SampleAggregate<Option<Median<O>>> for T
where
    T: Sample<O>,
    O: PartialEq + PartialOrd + Copy + Display,
{
    fn sample_aggregate(components: &[&Self]) -> Option<Median<O>>
    {
        if components.is_empty()
        {
            return None;
        }

        let sorted: BinaryHeap<_> = components
            .iter()
            .map(|&c| Sample::sample(c))
            .map(|v| sealed::Ordered(v))
            .collect();
        let sorted = sorted.into_sorted_vec();
        let median = sorted[sorted.len() / 2];
        Some(Median(*median))
    }
}

macro_rules! blanket_impl_sample_aggr_mean {
    ($t: tt) => {
        impl<T> SampleAggregate<Option<Mean<$t>>> for T
        where
            T: Sample<$t>,
        {
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::cast_possible_wrap)]
            #[allow(clippy::cast_possible_truncation)]
            fn sample_aggregate(components: &[&Self]) -> Option<Mean<$t>>
            {
                if components.is_empty()
                {
                    return None;
                }

                let sum: $t = components.iter().map(|&c| Sample::sample(c)).sum();
                let cnt = components.len() as $t;

                let mean = sum / cnt;

                Some(Mean(mean))
            }
        }
    };
}
blanket_impl_sample_aggr_mean!(usize);
blanket_impl_sample_aggr_mean!(u8);
blanket_impl_sample_aggr_mean!(u16);
blanket_impl_sample_aggr_mean!(u32);
blanket_impl_sample_aggr_mean!(u64);
blanket_impl_sample_aggr_mean!(u128);
blanket_impl_sample_aggr_mean!(i8);
blanket_impl_sample_aggr_mean!(i16);
blanket_impl_sample_aggr_mean!(i32);
blanket_impl_sample_aggr_mean!(i64);
blanket_impl_sample_aggr_mean!(i128);
blanket_impl_sample_aggr_mean!(f32);
blanket_impl_sample_aggr_mean!(f64);

mod sealed
{
    use std::fmt::Display;

    use bevy::prelude::Deref;

    /// Hidden type used for the implementation of some aggregate functions that require [`Ord`].
    /// Avoid bringing in `unordered_float` as a dependency prematurely.
    #[derive(PartialEq, Deref, Clone, Copy, Debug)]
    pub struct Ordered<T>(pub T)
    where
        T: PartialEq + PartialOrd;

    impl<T> PartialOrd for Ordered<T>
    where
        T: PartialEq + PartialOrd,
    {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
        {
            self.0.partial_cmp(&other.0)
        }
    }

    impl<T> Eq for Ordered<T> where T: PartialEq + PartialOrd {}

    impl<T> Ord for Ordered<T>
    where
        T: PartialEq + PartialOrd + Copy + Display,
    {
        #[allow(clippy::expect_used)]
        fn cmp(&self, other: &Self) -> std::cmp::Ordering
        {
            self.partial_cmp(other).unwrap_or_else(|| {
                panic!(
                    "error during aggregate sampling, cannot compare values: {}, {}",
                    self.0, other.0
                );
            })
        }
    }
}
