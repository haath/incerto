use bevy::prelude::*;

/// Implements the collection of a single component's value from the simulation.
///
/// Needed for [`super::prelude::MonteCarlo::collect_single`].
pub trait CollectSingle: Component
{
    type Out;

    fn collect(component: &Self) -> Self::Out;
}

/// Implements the collection of a value from multiple components in the simulation
///
/// Needed for [`super::prelude::MonteCarlo::collect_many`].
pub trait CollectMany: Component + Sized
{
    type Out;

    fn collect(components: &[&Self]) -> Self::Out;
}
