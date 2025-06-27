use bevy::prelude::*;

/// Implements the sampling of a value from components in the simulation.
///
/// Needed for [`super::prelude::Simulation::sample`].
pub trait Sample<Out>: Component + Sized
{
    fn sample(components: &[&Self]) -> Out;
}
