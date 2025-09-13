#![allow(clippy::expect_used)]
use incerto::prelude::*;

#[derive(Component, Default)]
struct MyValue(usize);

#[derive(Component)]
struct MyMarker;

impl SampleAggregate<usize> for MyValue
{
    fn sample_aggregate(components: &[&Self]) -> usize
    {
        components.iter().map(|c| c.0).sum()
    }
}

impl SampleAggregate<bool> for MyValue
{
    fn sample_aggregate(components: &[&Self]) -> bool
    {
        components.is_empty()
    }
}

#[test]
fn test_record_time_series_conflict()
{
    let mut builder = SimulationBuilder::new();

    // the first call to set up the recording is expected to succeed
    builder = builder
        .record_aggregate_time_series::<MyValue, usize>(8)
        .expect("first recording is expected to succeed");

    // the second is expected to error
    let res = builder.record_aggregate_time_series::<MyValue, usize>(12);
    let err = res.err().expect("the call above should have errored");
    assert_eq!(err, BuilderError::TimeSeriesRecordingConflict);
}

#[test]
fn test_record_time_series_same_component_different_out()
{
    let mut builder = SimulationBuilder::new();

    // the first call to set up the recording is expected to succeed
    builder = builder
        .record_aggregate_time_series::<MyValue, usize>(8)
        .expect("first recording is expected to succeed");

    // the second is expected to also succeed due to the different output type
    builder
        .record_aggregate_time_series::<MyValue, bool>(8)
        .expect("second recording is expected to succeed");
}

#[test]
fn test_record_time_series_same_component_same_out_different_filter()
{
    let mut builder = SimulationBuilder::new();

    // the first call to set up the recording is expected to succeed
    builder = builder
        .record_aggregate_time_series_filtered::<MyValue, With<MyMarker>, usize>(8)
        .expect("first recording is expected to succeed");

    // the second is expected to also succeed due to the different output type
    builder
        .record_aggregate_time_series_filtered::<MyValue, (), usize>(8)
        .expect("second recording is expected to succeed");
}
