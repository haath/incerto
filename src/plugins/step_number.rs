use bevy::prelude::*;

#[derive(Resource, Default, Debug, Deref)]
pub struct StepNumber(usize);

pub struct StepNumberPlugin;

impl Plugin for StepNumberPlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(StepNumber(0));

        // increment the step counter after any other systems in the simulation
        // this enables a reliable step number reading in PreUpdate
        app.add_systems(Last, step_counter_increment);
    }
}

fn step_counter_increment(mut step_counter: ResMut<StepNumber>)
{
    step_counter.0 += 1;
}
