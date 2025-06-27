use bevy::{app::ScheduleRunnerPlugin, ecs::system::ScheduleSystem, prelude::*};

use crate::{plugins::StepCounterPlugin, simulation::Simulation, spawner::Spawner};

/// Builder type used to construct a [`Simulation`] object.
///
/// The builder is used to logically separate the construction of a simulation with its execution.
/// Once built, a [`Simulation`] object may be reused in order to intermitently run simulation steps,
/// restart the simulation from the beginning, collect results and so on.
pub struct SimulationBuilder
{
    sim: Simulation,
}

impl Default for SimulationBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl SimulationBuilder
{
    #[must_use]
    pub fn new() -> Self
    {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_once()))
            .add_plugins(StepCounterPlugin);

        app.update();

        let sim = Simulation {
            app,
            spawners: Vec::new(),
        };

        Self { sim }
    }

    /// Add systems to the simulation.
    ///
    /// These are [`bevy systems`](https://bevy-cheatbook.github.io/programming/systems.html).
    ///
    /// This method can be called multiple times.
    /// The order in which systems are added via this method has no impact on the order in which
    /// systems may be executed.
    #[must_use]
    pub fn add_systems<M>(mut self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self
    {
        self.sim.app.add_systems(Update, systems);
        self
    }

    /// Add an entity spawner function to the simulation.
    ///
    /// In the beginning of ever simulation, each of the spawner functions added here
    /// will get called once. The [`Spawner`] that is passed as an argument shall be used
    /// to spawn entities into the simulation.
    ///
    /// This method may be called multiple times.
    /// The order in which the given functions are called is the same as the order in
    /// which they are added to the builder.
    ///
    /// Spawners shall be used to set up the initial state of a simulation.
    /// Additional entities can be spawned in an ongoing simulation using [Commands](https://bevy-cheatbook.github.io/programming/commands.html).
    #[must_use]
    pub fn add_entity_spawner(mut self, entity_spawner: fn(&mut Spawner)) -> Self
    {
        self.sim.spawners.push(entity_spawner);
        self
    }

    pub fn build(mut self) -> Simulation
    {
        self.sim.reset();

        self.sim
    }
}
