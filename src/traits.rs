use std::hash::Hash;

use crate::prelude::*;

/// Implements the sampling of a value from multiple components in the simulation.
///
/// Needed for:
/// * [`Simulation::sample`]
/// * [`Simulation::sample_single`]
/// * [`SimulationBuilder::record_time_series`]
pub trait SampleAggregate<Out>: Component + Sized
{
    /// Samples a single value of type `Out` from the values of all
    /// components in the simulation.
    ///
    /// The slice passed to this method is:
    /// * Guaranteed to not be empty.
    /// * In random arbitrary order.
    fn sample_aggregate(components: &[&Self]) -> Out;
}

/// Implements the sampling of a value from a component in the simulation.
///
/// Needed for:
/// * [`Simulation::sample_aggregate`]
/// * [`Simulation::sample_aggregate_filtered`]
/// * [`SimulationBuilder::record_aggregate_time_series`]
/// * [`SimulationBuilder::record_aggregate_time_series_filtered`]
///
/// `Sample<O>` is automatically implemented for any component that implements `Into<O>`
/// for numeric types: `u8`, `i32`, `f64` etc.
pub trait Sample<Out>: Component + Sized
{
    /// Samples a single value of type [`Out`] from the values of a
    /// components in the simulation.
    fn sample(component: &Self) -> Out;
}

/// A component whose value shall be used to uniquely identify an entity.
///
/// Typically, this component would hold some enum value or ID number.
/// Note that the user will need to ensure no two entities share the same [`Identifier`] value.
///
/// Automatically implemented for any type that is [`Component`] + [`Eq`] + [`Hash`]
pub trait Identifier: Component + Eq + Hash {}

// ===========================================================
//              Blanket implementations
// ===========================================================
impl<I> Identifier for I where I: Component + Eq + Hash {}

macro_rules! blanket_impl_sample {
    ($t: tt) => {
        impl<T> Sample<$t> for T
        where
            T: Component,
            for<'a> &'a T: Into<$t>,
        {
            fn sample(component: &Self) -> $t
            {
                component.into()
            }
        }
        // Deref will also become available when negative
        // generic constraints come to stable rust.
        // impl<T> Sample<$t> for T
        // where
        //     T: Component + Deref<Target = $t>,
        // {
        //     fn sample(component: &Self) -> $t
        //     {
        //         **component
        //     }
        // }
    };
}
blanket_impl_sample!(usize);
blanket_impl_sample!(u8);
blanket_impl_sample!(u16);
blanket_impl_sample!(u32);
blanket_impl_sample!(u64);
blanket_impl_sample!(u128);
blanket_impl_sample!(i8);
blanket_impl_sample!(i16);
blanket_impl_sample!(i32);
blanket_impl_sample!(i64);
blanket_impl_sample!(i128);
blanket_impl_sample!(f32);
blanket_impl_sample!(f64);
