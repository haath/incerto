use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::{
    Spawner,
    error::SampleError,
    plugins::{StepCounterPlugin, TimeSeries},
    traits::Sample,
};

type SpawnFn = Box<dyn Fn(&mut Spawner)>;

/// Executor of monte carlo experiments.
///
/// Constructed using [`super::SimulationBuilder`].
///
/// This type holds a simulation's state and provides methods for interacting with it,
/// such as running it, resetting it, and extracting values from the entities existing
/// within.
pub struct Simulation
{
    pub(super) app: App,
    pub(super) spawners: Vec<SpawnFn>,
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

    /// Fetch the value from a all entities' components in the simulation.
    ///
    /// This method uses the [`Sample<O>`] implementation to extract a single value
    /// of type `O` from all of the existing components and return it.
    ///
    /// # Errors
    ///
    /// - [`SampleError::ComponentMissing`]
    pub fn sample<C: Sample<Out>, Out>(&self) -> Result<Out, SampleError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<&C>()
            .ok_or(SampleError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(C::sample(&results))
    }

    /// Fetch the value from a multiple entities' components in the simulation.
    ///
    /// This method uses the [`Sample<O>`] implementation to extract a single value
    /// of type `O` from all of the components on entities selected with
    /// the filter `F`, and return it.
    ///
    /// # Errors
    ///
    /// - [`SampleError::ComponentMissing`]
    pub fn sample_filtered<C: Sample<Out>, F: QueryFilter, Out>(&self) -> Result<Out, SampleError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query_filtered::<&C, F>()
            .ok_or(SampleError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(C::sample(&results))
    }

    /// Counts the number of entities in the simulation that can be selected
    /// with a given filter `F`.
    ///
    /// The filter is a query filter, meaning it shall use selectors like
    /// [`With`] and [`Without`].
    ///
    /// # Errors
    ///
    /// - [`SampleError::ComponentMissing`]
    pub fn count<F: QueryFilter>(&self) -> Result<usize, SampleError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query_filtered::<(), F>()
            .ok_or(SampleError::ComponentMissing)?;

        let count = query.iter(world).count();

        Ok(count)
    }

    /// Retrieve the values of a time series that was recorded during the simulation.
    ///
    /// This is possible only after having called [`crate::SimulationBuilder::record_time_series`]
    /// during the construction of the simulation.
    ///
    /// # Errors
    ///
    /// - [`SampleError::TimeSeriesNotRecorded`]
    pub fn get_time_series<C, O>(&self) -> Result<Vec<&O>, SampleError>
    where
        C: Sample<O>,
        O: Send + Sync + 'static,
    {
        self.get_time_series_filtered::<C, (), O>()
    }

    /// Retrieve the values of a time series that was recorded during the simulation with filtering.
    ///
    /// This is possible only after having called [`crate::SimulationBuilder::record_time_series_filtered`]
    /// during the construction of the simulation.
    ///
    /// # Errors
    ///
    /// - [`SampleError::TimeSeriesNotRecorded`]
    pub fn get_time_series_filtered<C, F, O>(&self) -> Result<Vec<&O>, SampleError>
    where
        C: Sample<O>,
        F: QueryFilter + Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        let world = self.app.world();
        let time_series = world
            .get_resource::<TimeSeries<C, F, O>>()
            .ok_or(SampleError::TimeSeriesNotRecorded)?;

        let values = time_series.values.iter().by_ref().collect();

        Ok(values)
    }
}
