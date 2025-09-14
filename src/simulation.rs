use bevy::{
    ecs::query::{QueryFilter, QuerySingleError},
    prelude::*,
};

use crate::{
    Identifier, Sample, TimeSeries, error::SamplingError, plugins::TimeSeriesData,
    traits::SampleAggregate,
};

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

    /// Fetch the value from a specific entity's component in the simulation.
    ///
    /// This method uses the [`Sample<O>`] implementation to extract a single value
    /// of type `O` from a specific entity identified by `id` and return it.
    ///
    /// # Errors
    ///
    /// - [`SamplingError::ComponentMissing`]
    /// - [`SamplingError::EntityIdentifierNotFound`]
    /// - [`SamplingError::EntityIdentifierNotUnique`]
    pub fn sample<C: Sample<Out>, Id: Identifier, Out>(&self, id: &Id)
    -> Result<Out, SamplingError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<(&C, &Id)>()
            .ok_or(SamplingError::ComponentMissing)?;

        let mut result_iter = query.iter(world).filter(|&(_, entity_id)| entity_id == id);

        // sample the component of the first entity
        let (component, _) = result_iter
            .next()
            .ok_or(SamplingError::EntityIdentifierNotFound)?;

        // there should not be any more entities with the same ID
        if result_iter.next().is_some()
        {
            return Err(SamplingError::EntityIdentifierNotUnique);
        }

        Ok(C::sample(component))
    }

    /// Sample a single entity's component in the simulation.
    ///
    /// This method expects that exactly one entity exists in the simulation with
    /// the given component `C`.
    /// If no (or many) entities are found with this component, the corresponding
    /// error is returned.
    ///
    /// # Errors
    ///
    /// - [`SamplingError::ComponentMissing`]
    /// - [`SamplingError::SingleNoEntities`]
    /// - [`SamplingError::SingleMultipleEntities`]
    pub fn sample_single<C: Sample<Out>, Out>(&self) -> Result<Out, SamplingError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<&C>()
            .ok_or(SamplingError::ComponentMissing)?;

        let component = query.single(world).map_err(|e| match e
        {
            QuerySingleError::NoEntities(_) => SamplingError::SingleNoEntities,
            QuerySingleError::MultipleEntities(_) => SamplingError::SingleMultipleEntities,
        })?;

        Ok(C::sample(component))
    }

    /// Fetch the value from a all entities' components in the simulation.
    ///
    /// This method uses the [`SampleAggregate<O>`] implementation to extract a single value
    /// of type `O` from all of the existing components and return it.
    ///
    /// # Errors
    ///
    /// - [`SamplingError::ComponentMissing`]
    pub fn sample_aggregate<C: SampleAggregate<Out>, Out>(&self) -> Result<Out, SamplingError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<&C>()
            .ok_or(SamplingError::ComponentMissing)?;

        let results = query.iter(world).collect::<Vec<_>>();

        Ok(C::sample_aggregate(&results))
    }

    /// Fetch the value from a multiple entities' components in the simulation.
    ///
    /// This method uses the [`SampleAggregate<O>`] implementation to extract a single value
    /// of type `O` from all of the components on entities selected with
    /// the filter `F`, and return it.
    ///
    /// # Errors
    ///
    /// - [`SamplingError::ComponentMissing`]
    pub fn sample_aggregate_filtered<C: SampleAggregate<Out>, F: QueryFilter, Out>(
        &self,
    ) -> Result<Out, SamplingError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query_filtered::<&C, F>()
            .ok_or(SamplingError::ComponentMissing)?;

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
    /// - [`SamplingError::ComponentMissing`]
    pub fn count<F: QueryFilter>(&self) -> Result<usize, SamplingError>
    {
        let world = self.app.world();
        let mut query = world
            .try_query_filtered::<(), F>()
            .ok_or(SamplingError::ComponentMissing)?;

        let count = query.iter(world).count();

        Ok(count)
    }

    /// Retrieve the values of a time series that was recorded during the simulation on
    /// a specific entity identified by `id`.
    ///
    /// This is possible only after having called [`crate::SimulationBuilder::record_time_series`]
    /// during the construction of the simulation.
    ///
    /// # Errors
    ///
    /// - [`SamplingError::ComponentMissing`]
    /// - [`SamplingError::EntityIdentifierNotFound`]
    /// - [`SamplingError::EntityIdentifierNotUnique`]
    pub fn get_time_series<C, Id, Out>(
        &'_ self,
        id: &Id,
    ) -> Result<TimeSeries<'_, Out>, SamplingError>
    where
        Out: Send + Sync + 'static,
        C: Sample<Out>,
        Id: Identifier,
    {
        let world = self.app.world();
        let mut query = world
            .try_query::<(&TimeSeriesData<C, Id, Out>, &Id)>()
            .ok_or(SamplingError::ComponentMissing)?;

        let mut result_iter = query.iter(world).filter(|&(_, entity_id)| entity_id == id);

        // fetch the time series of the first entity
        let (time_series, _) = result_iter
            .next()
            .ok_or(SamplingError::EntityIdentifierNotFound)?;

        // there should not be any more entities with the same ID
        if result_iter.next().is_some()
        {
            return Err(SamplingError::EntityIdentifierNotUnique);
        }

        let time_series = time_series.collect();

        Ok(time_series)
    }

    /// Retrieve the values of a time series that was recorded during the simulation.
    ///
    /// This is possible only after having called [`crate::SimulationBuilder::record_aggregate_time_series`]
    /// during the construction of the simulation.
    ///
    /// # Errors
    ///
    /// - [`SamplingError::TimeSeriesNotRecorded`]
    pub fn get_aggregate_time_series<C, Out>(&'_ self) -> Result<TimeSeries<'_, Out>, SamplingError>
    where
        C: SampleAggregate<Out>,
        Out: Send + Sync + 'static,
    {
        self.get_aggregate_time_series_filtered::<C, (), Out>()
    }

    /// Retrieve the values of a time series that was recorded during the simulation with filtering.
    ///
    /// This is possible only after having called [`crate::SimulationBuilder::record_aggregate_time_series_filtered`]
    /// during the construction of the simulation.
    ///
    /// # Errors
    ///
    /// - [`SamplingError::TimeSeriesNotRecorded`]
    pub fn get_aggregate_time_series_filtered<C, Filter, Out>(
        &'_ self,
    ) -> Result<TimeSeries<'_, Out>, SamplingError>
    where
        C: SampleAggregate<Out>,
        Filter: QueryFilter + Send + Sync + 'static,
        Out: Send + Sync + 'static,
    {
        let world = self.app.world();
        let time_series = world
            .get_resource::<TimeSeriesData<C, Filter, Out>>()
            .ok_or(SamplingError::TimeSeriesNotRecorded)?;

        let time_series = time_series.collect();

        Ok(time_series)
    }
}
