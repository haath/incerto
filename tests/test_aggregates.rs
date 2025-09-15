#![allow(clippy::expect_used)]
use incerto::prelude::*;

#[derive(Component)]
struct Item(usize);

#[derive(Component)]
struct ItemFloat(f32);

impl Sample<usize> for Item
{
    fn sample(component: &Self) -> usize
    {
        component.0
    }
}

impl Sample<f32> for ItemFloat
{
    fn sample(component: &Self) -> f32
    {
        component.0
    }
}

#[test]
fn test_aggregates_int() -> Result<(), SimulationError>
{
    let builder = SimulationBuilder::new().add_entity_spawner(|spawner| {
        for i in 0..10
        {
            spawner.spawn(Item(2 * i + 1));
        }
    });

    let simulation = builder.build();

    let min = simulation.sample_aggregate::<Item, Minimum<_>>()?;
    assert_eq!(*min, 1);

    let max = simulation.sample_aggregate::<Item, Maximum<_>>()?;
    assert_eq!(*max, 19);

    let mean = simulation.sample_aggregate::<Item, Mean<_>>()?;
    assert_eq!(*mean, 10);

    let median = simulation.sample_aggregate::<Item, Median<_>>()?;
    assert_eq!(*median, 11);

    let percentile_10 = simulation.sample_aggregate::<Item, Percentile<_, 10>>()?;
    assert_eq!(*percentile_10, 3);

    let percentile_70 = simulation.sample_aggregate::<Item, Percentile<_, 70>>()?;
    assert_eq!(*percentile_70, 15);

    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::float_cmp)]
#[allow(clippy::suboptimal_flops)]
fn test_aggregates_float() -> Result<(), SimulationError>
{
    let builder = SimulationBuilder::new().add_entity_spawner(|spawner| {
        for i in 0..10
        {
            spawner.spawn(ItemFloat(2.0 * i as f32 + 1.0));
        }
    });

    let simulation = builder.build();

    let min = simulation.sample_aggregate::<ItemFloat, Minimum<_>>()?;
    assert_eq!(*min, 1.0);

    let max = simulation.sample_aggregate::<ItemFloat, Maximum<_>>()?;
    assert_eq!(*max, 19.0);

    let mean = simulation.sample_aggregate::<ItemFloat, Mean<_>>()?;
    assert_eq!(*mean, 10.0);

    let median = simulation.sample_aggregate::<ItemFloat, Median<_>>()?;
    assert_eq!(*median, 11.0);

    let percentile_30 = simulation.sample_aggregate::<ItemFloat, Percentile<_, 30>>()?;
    assert_eq!(*percentile_30, 7.0);

    Ok(())
}
