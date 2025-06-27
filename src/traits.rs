use bevy::prelude::*;

/// Implements the sampling of a value from components in the simulation.
///
/// Needed for [`super::prelude::Simulation::observe`].
pub trait Observe<Out>: Component + Sized
{
    fn observe(components: &[&Self]) -> Out;
}
