use bevy::{ecs::system::ScheduleSystem, prelude::*};

use crate::{
    error::CollectError,
    simulation::Simulation,
    spawner::Spawner,
    traits::{CollectMany, CollectSingle},
};

pub struct MonteCarlo
{
    sim: Simulation,
}

pub struct MonteCarloBuilder
{
    sim: Simulation,
}

impl Default for MonteCarloBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl MonteCarloBuilder
{
    #[must_use]
    pub fn new() -> Self
    {
        let sim = Simulation::new();

        Self { sim }
    }

    #[must_use]
    pub fn add_systems<M>(mut self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self
    {
        self.sim.app.add_systems(Update, systems);
        self
    }

    #[must_use]
    pub fn add_entity_spawner(mut self, entity_spawner: fn(&mut Spawner)) -> Self
    {
        self.sim.spawners.push(entity_spawner);
        self
    }

    pub fn build(self) -> MonteCarlo
    {
        MonteCarlo { sim: self.sim }
    }
}

impl MonteCarlo
{
    pub fn run(&mut self, num_steps: usize)
    {
        self.sim.run(num_steps);
    }

    /// Collects the value from a single entity's component in the simulation.
    ///
    /// # Errors
    ///
    /// - [`CollectError::ComponentMissing`]
    /// - [`CollectError::NoEntities`]
    /// - [`CollectError::MultipleEntities`]
    pub fn collect_single<CS: CollectSingle>(&self) -> Result<CS::Out, CollectError>
    {
        let world = self.sim.app.world();
        let mut query = world
            .try_query::<&CS>()
            .ok_or(CollectError::ComponentMissing)?;

        let result = query.single(world)?;

        Ok(CS::collect(result))
    }

    /// Collects the value from a multiple entities' components in the simulation.
    ///
    /// # Errors
    ///
    /// - [`CollectError::ComponentMissing`]
    pub fn collect_many<CM: CollectMany>(&self) -> Result<CM::Out, CollectError>
    {
        let world = self.sim.app.world();
        let mut query = world
            .try_query::<&CM>()
            .ok_or(CollectError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(CM::collect(&results))
    }

    /// Counts the number of components in the simulation.
    ///
    /// # Errors
    ///
    /// - [`CollectError::ComponentMissing`]
    pub fn count<C: Component>(&self) -> Result<usize, CollectError>
    {
        let world = self.sim.app.world();
        let mut query = world
            .try_query::<&C>()
            .ok_or(CollectError::ComponentMissing)?;

        let count = query.iter(world).count();

        Ok(count)
    }
}
