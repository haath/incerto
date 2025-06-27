use bevy::prelude::*;

#[derive(Resource, Default, Debug, Deref)]
pub struct StepCounter(usize);

pub struct StepCounterPlugin;

impl Plugin for StepCounterPlugin
{
    fn build(&self, app: &mut App)
    {
        Self::init(app);

        // increment the step counter before any other systems in the simulation
        // this enables a reliable step number reading in PreUpdate
        app.add_systems(First, step_counter_increment);
    }
}

impl StepCounterPlugin
{
    pub fn init(app: &mut App)
    {
        app.insert_resource(StepCounter(0));
    }
}

fn step_counter_increment(mut step_counter: ResMut<StepCounter>)
{
    step_counter.0 += 1;
}
