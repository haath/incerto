use bevy::prelude::*;

/// Implements the collection of a single component's value from the simulation.
///
/// Needed for [`super::prelude::MonteCarlo::observe_single`].
pub trait ObserveSingle: Component
{
    type Out;

    fn observe(component: &Self) -> Self::Out;
}

/// Implements the collection of a value from multiple components in the simulation
///
/// Needed for [`super::prelude::MonteCarlo::observe_many`].
pub trait ObserveMany: Component + Sized
{
    type Out;

    fn observe(components: &[&Self]) -> Self::Out;
}
