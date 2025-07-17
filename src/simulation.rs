use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::{error::SampleError, plugins::TimeSeries, traits::SampleAggregate};

/// Executor of monte carlo experiments.
///
/// Constructed using [`super::SimulationBuilder`].
///
/// This type holds a simulation's state and provides methods for interacting with it,
/// such as running it, and extracting values from the entities existing within.
pub struct Simulation
{
    pub(super) app: App,
}

impl Simulation
{
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
    pub fn sample<C: SampleAggregate<Out>, Out>(&self) -> Result<Out, SampleError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<&C>()
            .ok_or(SampleError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(C::sample_aggregate(&results))
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
    pub fn sample_filtered<C: SampleAggregate<Out>, F: QueryFilter, Out>(
        &self,
    ) -> Result<Out, SampleError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query_filtered::<&C, F>()
            .ok_or(SampleError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(C::sample_aggregate(&results))
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
    pub fn get_aggregate_time_series<C, O>(&self) -> Result<Vec<&O>, SampleError>
    where
        C: SampleAggregate<O>,
        O: Send + Sync + 'static,
    {
        self.get_aggregate_time_series_filtered::<C, (), O>()
    }

    /// Retrieve the values of a time series that was recorded during the simulation with filtering.
    ///
    /// This is possible only after having called [`crate::SimulationBuilder::record_time_series_filtered`]
    /// during the construction of the simulation.
    ///
    /// # Errors
    ///
    /// - [`SampleError::TimeSeriesNotRecorded`]
    pub fn get_aggregate_time_series_filtered<C, F, O>(&self) -> Result<Vec<&O>, SampleError>
    where
        C: SampleAggregate<O>,
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
