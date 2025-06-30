use std::marker::PhantomData;

use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::{Sample, plugins::step_counter::StepCounter};

#[derive(Resource, Default)]
pub struct TimeSeries<C, F, O>
{
    pub(crate) values: Vec<O>,
    sample_interval: usize,
    _pd_c: PhantomData<C>,
    _pd_f: PhantomData<F>,
}

#[derive(Default)]
pub struct TimeSeriesPlugin<C, F, O>
where
    C: Sample<O>,
    O: Send + Sync + 'static,
    F: QueryFilter + Send + Sync + 'static,
{
    sample_interval: usize,
    _pd_c: PhantomData<C>,
    _pd_f: PhantomData<F>,
    _pd_o: PhantomData<O>,
}

impl<C, F, O> TimeSeriesPlugin<C, F, O>
where
    C: Sample<O>,
    O: Send + Sync + 'static,
    F: QueryFilter + Send + Sync + 'static,
{
    pub const fn new(sample_interval: usize) -> Self
    {
        Self {
            sample_interval,
            _pd_c: PhantomData,
            _pd_f: PhantomData,
            _pd_o: PhantomData,
        }
    }

    fn time_series_reset(
        mut time_series: ResMut<TimeSeries<C, F, O>>,
        step_counter: Res<StepCounter>,
    )
    {
        // reset the time series data whenever the step counter is 0
        // this should occur on the first step of every simulation
        if **step_counter == 0
        {
            time_series.values.clear();
        }
    }

    fn time_series_sample(
        mut time_series: ResMut<TimeSeries<C, F, O>>,
        step_counter: Res<StepCounter>,
        query: Query<&C, F>,
    )
    {
        // only get new samples once every 'sample_interval' steps
        if **step_counter % time_series.sample_interval != 0
        {
            return;
        }

        let component_values = query.iter().collect::<Vec<_>>();

        let sample = C::sample(&component_values);

        time_series.values.push(sample);
    }
}

impl<C, F, O> Plugin for TimeSeriesPlugin<C, F, O>
where
    C: Sample<O>,
    O: Send + Sync + 'static,
    F: QueryFilter + Send + Sync + 'static,
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(TimeSeries {
            values: Vec::<O>::new(),
            sample_interval: self.sample_interval,
            _pd_c: PhantomData::<C>,
            _pd_f: PhantomData::<F>,
        });

        app.add_systems(PreUpdate, Self::time_series_reset)
            .add_systems(PostUpdate, Self::time_series_sample.chain());
    }
}
