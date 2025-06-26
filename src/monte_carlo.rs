use bevy::{ecs::system::ScheduleSystem, prelude::*};

use crate::{
    error::CollectError,
    simulation::Simulation,
    spawner::Spawner,
    traits::{ObserveMany, ObserveSingle},
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

    pub fn build(mut self) -> MonteCarlo
    {
        self.sim.reset();

        MonteCarlo { sim: self.sim }
    }
}

impl MonteCarlo
{
    /// Run a number of steps of a new simulation.
    ///
    /// Equivalent to calling [`Self::reset`] followed by [`Self::run`].
    pub fn run_new(&mut self, num_steps: usize)
    {
        self.reset();
        self.run(num_steps);
    }

    /// Reset the simulation to its initial state.
    ///
    /// Calling this method will clear all existing entities, reset all system resources,
    /// and execute all configured spawners.
    pub fn reset(&mut self)
    {
        self.sim.reset();
    }

    /// Run a number of steps of the simulation.
    pub fn run(&mut self, num_steps: usize)
    {
        for _ in 0..num_steps
        {
            self.sim.app.update();
        }
    }

    /// Fetch the value from a single entity's component in the simulation.
    ///
    /// # Errors
    ///
    /// - [`CollectError::ComponentMissing`]
    /// - [`CollectError::NoEntities`]
    /// - [`CollectError::MultipleEntities`]
    pub fn observe_single<CS: ObserveSingle>(&self) -> Result<CS::Out, CollectError>
    {
        let world = self.sim.app.world();
        let mut query = world
            .try_query::<&CS>()
            .ok_or(CollectError::ComponentMissing)?;

        let result = query.single(world)?;

        Ok(CS::observe(result))
    }

    /// Fetch the value from a multiple entities' components in the simulation.
    ///
    /// # Errors
    ///
    /// - [`CollectError::ComponentMissing`]
    pub fn observe_many<CM: ObserveMany>(&self) -> Result<CM::Out, CollectError>
    {
        let world = self.sim.app.world();
        let mut query = world
            .try_query::<&CM>()
            .ok_or(CollectError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(CM::observe(&results))
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
