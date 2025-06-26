use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct StepCounter(usize);

pub struct StepCounterPlugin;

impl Plugin for StepCounterPlugin
{
    fn build(&self, app: &mut App)
    {
        Self::init(app);

        app.add_systems(PreUpdate, step_counter_increment);
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
