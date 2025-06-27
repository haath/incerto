#![allow(clippy::expect_used)]
use incerto::prelude::*;

#[derive(Component)]
struct MyCounter(usize);

/// Collect the counter value.
impl ObserveSingle<usize> for MyCounter
{
    fn observe(component: &Self) -> usize
    {
        component.0
    }
}

/// Collect the sum of all counter values.
impl ObserveMany<usize> for MyCounter
{
    fn observe(components: &[&Self]) -> usize
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
        .observe_single::<MyCounter, _>()
        .expect("expect a single counter result");
    assert_eq!(counter, NUM_STEPS);

    simulation.run_new(NUM_STEPS);
    let counter = simulation
        .observe_single::<MyCounter, _>()
        .expect("expect a single counter result");
    assert_eq!(counter, NUM_STEPS);

    simulation.reset();
    simulation.run(NUM_STEPS);
    let counter = simulation
        .observe_single::<MyCounter, _>()
        .expect("expect a single counter result");
    assert_eq!(counter, NUM_STEPS);
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
        .observe_many::<MyCounter, _>()
        .expect("expect a single counter result");

    assert_eq!(counter_sum, NUM_STEPS * NUM_COUNTERS);
}
