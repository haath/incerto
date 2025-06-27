use bevy::ecs::query::QuerySingleError;

/// An error that occured when attempting to observe the value of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObserveError
{
    /// The component type given was not found in the simulation.
    /// This indicates that no entity with this component was ever spawned.
    ComponentMissing,

    /// No entities with the given component type  were found in the simulation.
    NoEntities,

    /// The component type given was found on multiple entities.
    /// This is an error when calling [`super::Simulation::observe_single`], as only
    /// one entity is expected to exist.
    MultipleEntities,
}

unsafe impl Send for ObserveError {}
unsafe impl Sync for ObserveError {}

impl From<QuerySingleError> for ObserveError
{
    fn from(value: QuerySingleError) -> Self
    {
        match value
        {
            QuerySingleError::NoEntities(_) => Self::NoEntities,
            QuerySingleError::MultipleEntities(_) => Self::MultipleEntities,
        }
    }
}
