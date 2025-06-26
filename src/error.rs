use bevy::ecs::query::QuerySingleError;

#[derive(Debug, Clone, Copy)]
pub enum CollectError
{
    /// The component type given to `collect()` was not found in the simulation.
    /// This indicates that no entity with this component was ever spawned.
    ComponentMissing,

    /// No entities with the component type given to `collect()` were found in the simulation.
    NoEntities,

    /// The component type given to `collect_single()` was found on multiple entities.
    MultipleEntities,
}

impl From<QuerySingleError> for CollectError
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
