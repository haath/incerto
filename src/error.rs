/// Grouping of all other error types in the crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationError
{
    Sampling(SamplingError),
    Builder(BuilderError),
}

/// An error that occured when attempting to sample the value of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingError
{
    /// The component type being sampled was not possible to query.
    /// This indicates that no entity with this component was ever spawned.
    ComponentDoesNotExist,

    /// Expected only a single entity with the given component type in the simulation
    /// from the call to [`crate::Simulation::sample_single`].
    /// This error indicates that no entities were found.
    SingleNoEntities,

    /// Expected only a single entity with the given component type in the simulation
    /// from the call to [`crate::Simulation::sample_single`].
    /// This error indicates that more than one entity was found.
    SingleMultipleEntities,

    /// Expected one or more entities with the given component type in the simulation
    /// for aggregate sampling.
    /// From the call to [`crate::Simulation::sample_aggregate`] or [`crate::Simulation::sample_aggregate_filtered`].
    AggregateNoEntities,

    /// The requested time series has not been recorded in the simulation.
    /// This indicates that [`crate::Simulation::get_time_series`] was called without
    /// first having called [`crate::SimulationBuilder::record_time_series`] or
    /// [`crate::SimulationBuilder::record_time_series_filtered`].
    TimeSeriesNotRecorded,

    /// No entity was found in the simulation bearing the given value of the [`crate::Identifier`] component.
    EntityIdentifierNotFound,

    /// More than one entity was found in the simulation with the same value of the [`crate::Identifier`] component.
    EntityIdentifierNotUnique,
}

/// An error that occured when building a simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderError
{
    /// The time series for the given pair of component and out types
    /// has already been set up for recording.
    TimeSeriesRecordingConflict,
}

unsafe impl Send for SamplingError {}
unsafe impl Sync for SamplingError {}
unsafe impl Send for BuilderError {}
unsafe impl Sync for BuilderError {}

impl From<SamplingError> for SimulationError
{
    fn from(value: SamplingError) -> Self
    {
        Self::Sampling(value)
    }
}

impl From<BuilderError> for SimulationError
{
    fn from(value: BuilderError) -> Self
    {
        Self::Builder(value)
    }
}
