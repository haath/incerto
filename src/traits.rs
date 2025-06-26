use bevy::prelude::*;

/// Implements the collection of a single component's value from the simulation.
///
/// Needed for [`super::prelude::MonteCarlo::collect_single`].
pub trait CollectSingle: Component
{
    type Out;

    fn collect(&self) -> Self::Out;
}
