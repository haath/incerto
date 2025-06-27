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

unsafe impl Send for SampleError {}
unsafe impl Sync for SampleError {}
