use bevy::{
    app::Update,
    ecs::{schedule::IntoScheduleConfigs, system::ScheduleSystem},
};

use crate::{error::CollectError, simulation::Simulation, spawner::Spawner, traits::CollectSingle};

pub struct MonteCarlo
{
    num_steps: usize,
    sim: Simulation,
}

pub struct MonteCarloBuilder
{
    num_steps: usize,
    sim: Simulation,
}

impl MonteCarloBuilder
{
    #[must_use]
    pub fn new(num_steps: usize) -> Self
    {
        let sim = Simulation::new();

        Self { num_steps, sim }
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
        MonteCarlo {
            num_steps: self.num_steps,
            sim: self.sim,
        }
    }
}

impl MonteCarlo
{
    pub fn run(&mut self)
    {
        self.sim.run(self.num_steps);
    }

    /// Collects the value from a single component in the simulation.
    ///
    /// # Errors
    ///
    /// - [`CollectError::ComponentMissing`]
    /// - [`CollectError::NoEntities`]
    /// - [`CollectError::MultipleEntities`]
    pub fn collect_single<C: CollectSingle>(&self) -> Result<C::Out, CollectError>
    {
        let world = self.sim.app.world();
        let mut query = world
            .try_query::<&C>()
            .ok_or(CollectError::ComponentMissing)?;

        let result = query.single(world)?;

        Ok(result.collect())
    }
}
