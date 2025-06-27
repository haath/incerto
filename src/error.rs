/// An error that occured when attempting to observe the value of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObserveError
{
    /// The component type given was not found in the simulation.
    /// This indicates that no entity with this component was ever spawned.
    ComponentMissing,
}

unsafe impl Send for ObserveError {}
unsafe impl Sync for ObserveError {}
