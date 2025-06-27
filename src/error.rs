/// An error that occured when attempting to sample the value of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleError
{
    /// The component type given was not found in the simulation.
    /// This indicates that no entity with this component was ever spawned.
    ComponentMissing,
}

unsafe impl Send for SampleError {}
unsafe impl Sync for SampleError {}
