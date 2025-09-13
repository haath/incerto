use std::marker::PhantomData;

use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::{Identifier, Sample, SampleAggregate, plugins::step_counter::StepCounter};

#[derive(Component, Resource, Default)]
pub struct TimeSeries<C, F, O>
{
    pub(crate) values: Vec<O>,
    sample_interval: usize,
    _phantom: PhantomData<(C, F)>,
}

impl<C, F, O> TimeSeries<C, F, O>
{
    const fn new(sample_interval: usize) -> Self
    {
        Self {
            values: Vec::new(),
            sample_interval,
            _phantom: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct AggregateTimeSeriesPlugin<C, F, O>
where
    C: SampleAggregate<O>,
    O: Send + Sync + 'static,
    F: QueryFilter + Send + Sync + 'static,
{
    sample_interval: usize,
    _phantom: PhantomData<(C, F, O)>,
}

impl<C, F, O> AggregateTimeSeriesPlugin<C, F, O>
where
    C: SampleAggregate<O>,
    O: Send + Sync + 'static,
    F: QueryFilter + Send + Sync + 'static,
{
    #[must_use]
    pub const fn new(sample_interval: usize) -> Self
    {
        Self {
            sample_interval,
            _phantom: PhantomData,
        }
    }

    fn time_series_sample(
        mut time_series: ResMut<TimeSeries<C, F, O>>,
        step_counter: Res<StepCounter>,
        query: Query<&C, F>,
    )
    {
        // only get new samples once every 'sample_interval' steps
        if step_counter.is_multiple_of(time_series.sample_interval)
        {
            let component_values = query.iter().collect::<Vec<_>>();

            let sample = C::sample_aggregate(&component_values);

            time_series.values.push(sample);
        }
    }
}

impl<C, F, O> Plugin for AggregateTimeSeriesPlugin<C, F, O>
where
    C: SampleAggregate<O>,
    O: Send + Sync + 'static,
    F: QueryFilter + Send + Sync + 'static,
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(TimeSeries::<C, F, O>::new(self.sample_interval));

        app.add_systems(PostUpdate, Self::time_series_sample);
    }
}

#[derive(Resource, Deref)]
pub struct SampleInterval<C, F, O>(#[deref] usize, PhantomData<(C, F, O)>);

#[derive(Default)]
pub struct TimeSeriesPlugin<C, I, O>
where
    C: Sample<O>,
    O: Send + Sync + 'static,
    I: Identifier,
{
    sample_interval: usize,
    _phantom: PhantomData<(C, I, O)>,
}

impl<C, I, O> TimeSeriesPlugin<C, I, O>
where
    C: Sample<O>,
    O: Send + Sync + 'static,
    I: Identifier,
{
    #[must_use]
    pub const fn new(sample_interval: usize) -> Self
    {
        Self {
            sample_interval,
            _phantom: PhantomData,
        }
    }

    fn time_series_init(
        mut commands: Commands,
        query: Query<Entity, Added<C>>,
        sample_interval: Res<SampleInterval<C, I, O>>,
    )
    {
        for entity in &query
        {
            commands
                .entity(entity)
                .insert(TimeSeries::<C, I, O>::new(**sample_interval));
        }
    }

    fn time_series_sample(mut query: Query<(&C, &mut TimeSeries<C, I, O>)>)
    {
        for (component, mut time_series) in &mut query
        {
            let sample = C::sample(component);
            time_series.values.push(sample);
        }
    }
}

impl<C, I, O> Plugin for TimeSeriesPlugin<C, I, O>
where
    C: Sample<O>,
    O: Send + Sync + 'static,
    I: Identifier,
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(SampleInterval::<C, I, O>(self.sample_interval, PhantomData));

        app.add_systems(
            PostUpdate,
            (Self::time_series_init, Self::time_series_sample).chain(),
        );
    }
}
