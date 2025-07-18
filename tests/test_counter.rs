#![allow(clippy::expect_used)]
#![allow(clippy::similar_names)]
use incerto::prelude::*;

#[derive(Component)]
struct MyCounter(usize);

#[derive(Component)]
struct GroupA;

#[derive(Component)]
struct GroupB;

/// Collect the sum of all counter values.
impl Sample<usize> for MyCounter
{
    fn sample(components: &[&Self]) -> usize
    {
        components.iter().map(|c| c.0).sum()
    }
}

#[test]
fn test_counter()
{
    const NUM_STEPS: usize = 100;

    let builder = SimulationBuilder::new()
        .add_systems(|mut query: Query<&mut MyCounter>| {
            let mut counter = query.single_mut().expect("expect a single counter entity");

            counter.0 += 1;
        })
        .add_entity_spawner(|spawner| {
            spawner.spawn(MyCounter(0));
        });

    let mut simulation = builder.build();

    simulation.run(NUM_STEPS);
    let counter = simulation
        .sample::<MyCounter, _>()
        .expect("expected to sample the counter sum");
    assert_eq!(counter, NUM_STEPS);

    simulation.run(NUM_STEPS);
    let counter = simulation
        .sample::<MyCounter, _>()
        .expect("expected to sample the counter sum");
    assert_eq!(counter, 2 * NUM_STEPS);
}

#[test]
fn test_many_counters()
{
    const NUM_STEPS: usize = 100;
    const NUM_COUNTERS: usize = 56;

    let builder = SimulationBuilder::new()
        .add_systems(|mut query: Query<&mut MyCounter>| {
            for mut counter in &mut query
            {
                counter.0 += 1;
            }
        })
        .add_entity_spawner(|spawner| {
            for _ in 0..NUM_COUNTERS
            {
                spawner.spawn(MyCounter(0));
            }
        });

    let mut simulation = builder.build();

    simulation.run(NUM_STEPS);

    let counter_sum = simulation
        .sample::<MyCounter, _>()
        .expect("expected to sample the counter sum");

    assert_eq!(counter_sum, NUM_STEPS * NUM_COUNTERS);
}

#[test]
fn test_counters_two_groups()
{
    const NUM_STEPS: usize = 100;
    const NUM_COUNTERS_PER_GROUP: usize = 50;

    let builder = SimulationBuilder::new()
        // Increment all counters on each step.
        .add_systems(|mut query: Query<&mut MyCounter>| {
            for mut counter in &mut query
            {
                counter.0 += 1;
            }
        })
        // Increment all counters in GroupA a second time
        .add_systems(|mut query: Query<&mut MyCounter, With<GroupA>>| {
            for mut counter in &mut query
            {
                counter.0 += 1;
            }
        })
        .add_entity_spawner(|spawner| {
            for _ in 0..NUM_COUNTERS_PER_GROUP
            {
                spawner.spawn((GroupA, MyCounter(0)));
                spawner.spawn((GroupB, MyCounter(0)));
            }
        });

    let mut simulation = builder.build();

    simulation.run(NUM_STEPS);
    let all_counters_sum = simulation
        .sample::<MyCounter, _>()
        .expect("expected to sample the counter sum");
    assert_eq!(all_counters_sum, 3 * NUM_STEPS * NUM_COUNTERS_PER_GROUP);

    let group_a_sum = simulation
        .sample_filtered::<MyCounter, With<GroupA>, _>()
        .expect("expected to sample the counter sum");
    assert_eq!(group_a_sum, 2 * NUM_STEPS * NUM_COUNTERS_PER_GROUP);

    let group_b_sum = simulation
        .sample_filtered::<MyCounter, With<GroupB>, _>()
        .expect("expected to sample the counter sum");
    assert_eq!(group_b_sum, NUM_STEPS * NUM_COUNTERS_PER_GROUP);
}

#[test]
fn test_counter_time_series()
{
    const NUM_STEPS: usize = 100;

    let builder = SimulationBuilder::new()
        .add_systems(|mut query: Query<&mut MyCounter>| {
            let mut counter = query.single_mut().expect("expect a single counter entity");

            counter.0 += 1;
        })
        .add_entity_spawner(|spawner| {
            spawner.spawn(MyCounter(0));
        })
        .record_time_series::<MyCounter, _>(10)
        .expect("error building simulation");

    let mut simulation = builder.build();

    simulation.run(NUM_STEPS);

    let values = simulation
        .get_time_series::<MyCounter, _>()
        .expect("time series not recorded")
        .into_iter()
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(values, vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);

    // run for 10 more steps, check that the next value is sampled into the continuing series
    simulation.run(10);

    let values = simulation
        .get_time_series::<MyCounter, _>()
        .expect("time series not recorded")
        .into_iter()
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(values, vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110]);
}
