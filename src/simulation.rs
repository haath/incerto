use bevy::prelude::*;

use crate::{
    Spawner,
    error::ObserveError,
    plugins::StepCounterPlugin,
    traits::{ObserveMany, ObserveSingle},
};

/// Executor of monte carlo experiments.
///
/// Constructed using [`super::SimulationBuilder`].
///
/// This type holds a simulation's states and provides methods for interacting with it,
/// such as running it, resetting it, and extracting values from the entities existing
/// within.
pub struct Simulation
{
    pub(super) app: App,
    pub(super) spawners: Vec<fn(&mut Spawner)>,
}

impl Simulation
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
        self.app.world_mut().clear_entities();

        // init all plugins necessary
        StepCounterPlugin::init(&mut self.app);

        // spawn all entities
        let mut spawner = Spawner(self.app.world_mut());
        for spawn_fn in &self.spawners
        {
            spawn_fn(&mut spawner);
        }
    }

    /// Run a number of steps of the simulation.
    pub fn run(&mut self, num_steps: usize)
    {
        for _ in 0..num_steps
        {
            self.app.update();
        }
    }

    /// Fetch the value from a single entity's component in the simulation.
    ///
    /// Calling this method implies that only a single entity exists with the
    /// given component type.
    /// This method then uses the [`ObserveSingle`] implementation to extract
    /// a value of [`ObserveSingle::Out`] from that component and return it.
    ///
    /// # Errors
    ///
    /// - [`ObserveError::ComponentMissing`]
    /// - [`ObserveError::NoEntities`]
    /// - [`ObserveError::MultipleEntities`]
    pub fn observe_single<CS: ObserveSingle>(&self) -> Result<CS::Out, ObserveError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<&CS>()
            .ok_or(ObserveError::ComponentMissing)?;

        let result = query.single(world)?;

        Ok(CS::observe(result))
    }

    /// Fetch the value from a multiple entities' components in the simulation.
    ///
    /// This method uses the [`ObserveMany`] implementation to extract a single value
    /// of [`ObserveMany::Out`] from all of the existing components and return it.
    ///
    /// # Errors
    ///
    /// - [`ObserveError::ComponentMissing`]
    pub fn observe_many<CM: ObserveMany>(&self) -> Result<CM::Out, ObserveError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<&CM>()
            .ok_or(ObserveError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(CM::observe(&results))
    }

    /// Counts the number of components in the simulation.
    ///
    /// # Errors
    ///
    /// - [`ObserveError::ComponentMissing`]
    pub fn count<C: Component>(&self) -> Result<usize, ObserveError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<&C>()
            .ok_or(ObserveError::ComponentMissing)?;

        let count = query.iter(world).count();

        Ok(count)
    }
}
