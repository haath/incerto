#![allow(clippy::expect_used)]
use incerto::prelude::*;

#[derive(Component)]
struct MyCounter(usize);

/// Collect the counter value.
impl CollectSingle for MyCounter
{
    type Out = usize;

    fn collect(component: &Self) -> Self::Out
    {
        component.0
    }
}

/// Collect the sum of all counter values.
impl CollectMany for MyCounter
{
    type Out = usize;

    fn collect(components: &[&Self]) -> Self::Out
    {
        components.iter().map(|c| c.0).sum()
    }
}

#[test]
fn test_counter()
{
    const NUM_STEPS: usize = 100;

    let builder = MonteCarloBuilder::new(NUM_STEPS)
        .add_systems(|mut query: Query<&mut MyCounter>| {
            let mut counter = query.single_mut().expect("expect a single counter entity");

            counter.0 += 1;
        })
        .add_entity_spawner(|spawner| {
            spawner.spawn(MyCounter(0));
        });

    let mut monte_carlo = builder.build();

    monte_carlo.run();
    let counter = monte_carlo
        .collect_single::<MyCounter>()
        .expect("expect a single counter result");
    assert_eq!(counter, NUM_STEPS);

    monte_carlo.run();
    let counter = monte_carlo
        .collect_single::<MyCounter>()
        .expect("expect a single counter result");
    assert_eq!(counter, NUM_STEPS);
}

#[test]
fn test_many_counters()
{
    const NUM_STEPS: usize = 100;
    const NUM_COUNTERS: usize = 56;

    let builder = MonteCarloBuilder::new(NUM_STEPS)
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

    let mut monte_carlo = builder.build();

    monte_carlo.run();
    let counter_sum = monte_carlo
        .collect_many::<MyCounter>()
        .expect("expect a single counter result");

    assert_eq!(counter_sum, NUM_STEPS * NUM_COUNTERS);
}
