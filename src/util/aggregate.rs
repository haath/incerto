use std::{cmp::Ordering, collections::BinaryHeap};

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

macro_rules! blanket_impl_sample_aggr_median_int {
    ($t: tt) => {
        impl<T> SampleAggregate<Option<Median<$t>>> for T
        where
            T: Sample<$t>,
        {
            fn sample_aggregate(components: &[&Self]) -> Option<Median<$t>>
            {
                if components.is_empty()
                {
                    return None;
                }

                let sorted: BinaryHeap<_> = components.iter().map(|&c| Sample::sample(c)).collect();
                let sorted = sorted.as_slice();
                let median = sorted[sorted.len() / 2];
                Some(Median(median))
            }
        }
    };
}
blanket_impl_sample_aggr_median_int!(usize);
blanket_impl_sample_aggr_median_int!(u8);
blanket_impl_sample_aggr_median_int!(u16);
blanket_impl_sample_aggr_median_int!(u32);
blanket_impl_sample_aggr_median_int!(u64);
blanket_impl_sample_aggr_median_int!(u128);
blanket_impl_sample_aggr_median_int!(i8);
blanket_impl_sample_aggr_median_int!(i16);
blanket_impl_sample_aggr_median_int!(i32);
blanket_impl_sample_aggr_median_int!(i64);
blanket_impl_sample_aggr_median_int!(i128);

macro_rules! blanket_impl_sample_aggr_median_float {
    ($t: tt) => {
        impl<T> SampleAggregate<Option<Median<$t>>> for T
        where
            T: Sample<$t>,
        {
            fn sample_aggregate(components: &[&Self]) -> Option<Median<$t>>
            {
                if components.is_empty()
                {
                    return None;
                }

                let sorted: BinaryHeap<_> = components
                    .iter()
                    .map(|&c| Sample::sample(c))
                    .map(|f| ordered_float::OrderedFloat(f))
                    .collect();
                let sorted = sorted.as_slice();
                let median = sorted[sorted.len() / 2];
                Some(Median(*median))
            }
        }
    };
}
blanket_impl_sample_aggr_median_float!(f32);
blanket_impl_sample_aggr_median_float!(f64);
