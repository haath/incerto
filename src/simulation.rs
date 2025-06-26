use bevy::{app::ScheduleRunnerPlugin, prelude::*};

use crate::{plugins::StepCounterPlugin, spawner::Spawner};

pub struct Simulation
{
    pub app: App,
    pub spawners: Vec<fn(&mut Spawner)>,
}

impl Simulation
{
    pub fn new() -> Self
    {
        let mut sim = App::new();

        sim.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_once()))
            .add_plugins(StepCounterPlugin);

        sim.update();

        Self {
            app: sim,
            spawners: Vec::new(),
        }
    }

    pub fn reset(&mut self)
    {
        self.app.world_mut().clear_entities();

        // init all plugins necessary
        StepCounterPlugin::init(&mut self.app);

        // spawn all entities
        let mut spawner = Spawner(self.app.world_mut());
        for spawn_fn in &self.spawners
        {
            spawn_fn(&mut spawner);
        }
    }
}
