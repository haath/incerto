/// Grouping of all other error types in the crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationError
{
    SampleError(SampleError),
    BuildError(SimulationBuildError),
}

/// An error that occured when attempting to sample the value of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleError
{
    /// The component type given was not found in the simulation.
    /// This indicates that no entity with this component was ever spawned.
    ComponentMissing,

    /// The requested time series has not been recorded in the simulation.
    /// This indicates that [`crate::Simulation::get_time_series`] was called without
    /// first having called [`crate::SimulationBuilder::record_time_series`] or
    /// [`crate::SimulationBuilder::record_time_series_filtered`].
    TimeSeriesNotRecorded,
}

/// An error that occured when building a simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationBuildError
{
    /// The time series for the given pair of component and out types
    /// has already been set up for recording.
    TimeSeriesRecordingConflict,
}

unsafe impl Send for SampleError {}
unsafe impl Sync for SampleError {}
unsafe impl Send for SimulationBuildError {}
unsafe impl Sync for SimulationBuildError {}

impl From<SampleError> for SimulationError
{
    fn from(value: SampleError) -> Self
    {
        Self::SampleError(value)
    }
}

impl From<SimulationBuildError> for SimulationError
{
    fn from(value: SimulationBuildError) -> Self
    {
        Self::BuildError(value)
    }
}
