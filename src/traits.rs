use bevy::prelude::*;

/// Implements the collection of a single component's value from the simulation.
///
/// Needed for [`super::prelude::Simulation::observe_single`].
pub trait ObserveSingle<Out>: Component
{
    fn observe(component: &Self) -> Out;
}

/// Implements the collection of a value from multiple components in the simulation
///
/// Needed for [`super::prelude::Simulation::observe_many`].
pub trait ObserveMany<Out>: Component + Sized
{
    fn observe(components: &[&Self]) -> Out;
}
