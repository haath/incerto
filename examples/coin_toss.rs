//! # Monte Carlo simulation of coin tossing.
//!
//! This example showcases a simple simulation where the goal is to measure the odds of
//! a coin toss turning up heads.
//!
//! The entities are coin-tossers, who on each step of the simulation toss a coin and record their result.
//! We spawn a number of these coin-tossers, and at the end of the simulation collect the average of
//! the odds that each one of them observed.
//!
//! Note that the entities in this example do not interact with each other, and are therefore independent.
//! In such a case, running a single simulation with multiple entities is equivalent to running multiple
//! simulations with a single entity.

#![allow(clippy::unwrap_used)]
use incerto::prelude::*;
use rand::prelude::*;

const SIMULATION_STEPS: usize = 10_000;
const NUM_ENTITIES: usize = 100;
const ODDS_HEADS: f64 = 0.5;

/// 1. Create a component for our simulated entities.
#[derive(Component, Default)]
struct CoinTosser
{
    num_heads: usize,
    num_tosses: usize,
}

/// 2. Implement a collector for the component values.
///
/// In this case we want to collect the average odds of getting heads in a coin toss.
impl ObserveMany for CoinTosser
{
    type Out = f64;

    #[allow(clippy::cast_precision_loss)]
    fn observe(components: &[&Self]) -> Self::Out
    {
        assert!(!components.is_empty());

        let odds_sum = components
            .iter()
            .map(|coin_tosser|
            // compute the odds of heads
            (coin_tosser.num_heads as f64) / (coin_tosser.num_tosses as f64))
            .sum::<f64>();

        odds_sum / (components.len() as f64)
    }
}

fn main()
{
    // 3. Initialize a SimulationBuilder.
    let mut builder = SimulationBuilder::new();

    // 4. Add a spawner to the simulation.
    //    These are executed once at the beginning of each simulation.
    builder = builder.add_entity_spawner(|spawner| {
        for _ in 0..NUM_ENTITIES
        {
            spawner.spawn(CoinTosser::default());
        }
    });

    // 5. Add a system which will update the simulation on each step.
    //    The following system simulates a single coin toss for each entity.
    builder = builder.add_systems(|mut query: Query<&mut CoinTosser>| {
        let mut rng = rand::rng();

        for mut coin_tosser in &mut query
        {
            // toss a coin with the appropriate odds
            let coin_toss_heads = rng.random_bool(ODDS_HEADS);

            if coin_toss_heads
            {
                coin_tosser.num_heads += 1;
            }
            coin_tosser.num_tosses += 1;
        }
    });

    // 6. Create and run the simulation.
    //    Note that the Simulation object can be reused to run the simulation
    //    multiple times.
    let mut simulation = builder.build();
    simulation.run(SIMULATION_STEPS);

    // 7. Collect results from the simulation.
    let odds_heads = simulation.observe_many::<CoinTosser>().unwrap();
    println!("heads odds: {:.2} %", odds_heads * 100.0);
}
