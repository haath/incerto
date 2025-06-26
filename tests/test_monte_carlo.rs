#![allow(clippy::expect_used)]
use incerto::prelude::*;

#[derive(Component)]
struct MyCounter(usize);

impl CollectSingle for MyCounter
{
    type Out = usize;

    fn collect(&self) -> Self::Out
    {
        self.0
    }
}

#[test]
fn test_simple_counter()
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
