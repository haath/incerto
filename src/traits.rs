use bevy::prelude::*;

/// Implements the collection of a single component's value from the simulation.
///
/// Needed for [`super::prelude::MonteCarlo::collect_single`].
pub trait CollectSingle: Component
{
    type Out;

    fn collect(component: &Self) -> Self::Out;
}

pub trait CollectMany: Component + Sized
{
    type Out;

    fn collect(components: &[&Self]) -> Self::Out;
}
