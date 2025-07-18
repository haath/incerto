use bevy::prelude::*;

/// Implements the sampling of a value from components in the simulation.
///
/// Needed for [`super::prelude::Simulation::sample`].
pub trait Sample<Out>: Component + Sized
{
    /// Samples a single value of type [`Out`] from the values of all
    /// components in the simulation.
    /// Note that the order of the `components` array passed here is random
    /// and should not be relied on.
    fn sample(components: &[&Self]) -> Out;
}
