use bevy::{
    app::ScheduleRunnerPlugin,
    ecs::{query::QueryFilter, system::ScheduleSystem},
    prelude::*,
};

use crate::{
    Sample, SimulationBuildError,
    plugins::{
        GridBounds2D, GridBounds3D, SpatialGridPlugin2D, SpatialGridPlugin3D, StepCounterPlugin,
        TimeSeries, TimeSeriesPlugin,
    },
    simulation::Simulation,
    spawner::Spawner,
};

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

    /// Register an event type in the simulation.
    ///
    /// These are [`bevy events`](https://bevy-cheatbook.github.io/programming/events.html).
    ///
    /// After registering, events can be used in simulation systems by arguments such as
    /// [`EventReader<E>`] and [`EventWriter<E>`].
    #[must_use]
    pub fn register_event<E: Event>(mut self) -> Self
    {
        self.sim.app.add_event::<E>();
        self
    }

    /// Add 2D spatial grid support to the simulation.
    ///
    /// This enables efficient spatial queries for entities with `GridPosition2D` components.
    /// The spatial grid provides O(1) neighbor lookups and distance-based entity searches.
    ///
    /// # Example
    /// ```rust
    /// use incerto::prelude::*;
    ///
    /// let bounds = GridBounds2D::new_2d(0, 99, 0, 99);
    /// let simulation = SimulationBuilder::new()
    ///     .add_spatial_grid(bounds)
    ///     .build();
    /// ```
    #[must_use]
    pub fn add_spatial_grid(mut self, bounds: GridBounds2D) -> Self
    {
        self.sim
            .app
            .add_plugins(SpatialGridPlugin2D::with_bounds(bounds));
        self
    }

    /// Add 3D spatial grid support to the simulation.
    ///
    /// This enables efficient spatial queries for entities with `GridPosition3D` components.
    /// The spatial grid provides O(1) neighbor lookups and distance-based entity searches.
    ///
    /// # Example
    /// ```rust
    /// use incerto::{SimulationBuilder, plugins::GridBounds3D};
    ///
    /// let bounds = GridBounds3D::new_3d(0, 99, 0, 99, 0, 99);
    /// let simulation = SimulationBuilder::new()
    ///     .add_spatial_grid_3d(bounds)
    ///     .build();
    /// ```
    #[must_use]
    pub fn add_spatial_grid_3d(mut self, bounds: GridBounds3D) -> Self
    {
        self.sim
            .app
            .add_plugins(SpatialGridPlugin3D::with_bounds(bounds));
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
    pub fn add_entity_spawner(mut self, entity_spawner: impl Fn(&mut Spawner) + 'static) -> Self
    {
        self.sim.spawners.push(Box::new(entity_spawner));
        self
    }

    /// Sets up the recording of a time series.
    ///
    /// The values in the time series will be values of type `O`
    /// sampled from all components `C` in the simulation according to the
    /// implementation of [`Sample<O>`] for `C`.
    ///
    /// The sampling will occur once every `sample_interval` steps.
    /// Specifically at the end of the step, after all user-defined systems have run.
    ///
    /// Note that it is currently not possible to record more than one time series
    /// with the same pair of component (`C`) and value (`O`).
    ///
    /// # Errors
    ///
    /// - [`SimulationBuildError::TimeSeriesRecordingConflict`]
    ///
    /// # Panics
    ///
    /// This method will panic if:
    ///
    /// - The given `sample_interval` is `0`.
    #[inline]
    pub fn record_time_series<C, O>(
        self,
        sample_interval: usize,
    ) -> Result<Self, SimulationBuildError>
    where
        C: Sample<O>,
        O: Send + Sync + 'static,
    {
        self.record_time_series_filtered::<C, (), O>(sample_interval)
    }

    /// Sets up the recording of a time series.
    ///
    /// The values in the time series will be values of type `O`
    /// sampled from components `C`, from all entities selected by the filter `F`
    /// in the simulation according to the implementation of [`Sample<O>`] for `C`.
    ///
    /// The sampling will occur once every `sample_interval` steps.
    /// Specifically at the end of the step, after all user-defined systems have run.
    ///
    /// Note that it is currently not possible to record more than one time series
    /// with the same pair of component (`C`) and value (`O`).
    ///
    /// # Errors
    ///
    /// - [`SimulationBuildError::TimeSeriesRecordingConflict`]
    ///
    /// # Panics
    ///
    /// This method will panic if:
    ///
    /// - The given `sample_interval` is `0`.
    pub fn record_time_series_filtered<C, F, O>(
        mut self,
        sample_interval: usize,
    ) -> Result<Self, SimulationBuildError>
    where
        C: Sample<O>,
        F: QueryFilter + Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        assert!(sample_interval > 0);

        let world = self.sim.app.world();
        if world.get_resource::<TimeSeries<C, F, O>>().is_some()
        {
            // More than one time series recording for the same C, F, O is not possible.
            return Err(SimulationBuildError::TimeSeriesRecordingConflict);
        }

        self.sim
            .app
            .add_plugins(TimeSeriesPlugin::<C, F, O>::new(sample_interval));
        Ok(self)
    }

    pub fn build(mut self) -> Simulation
    {
        self.sim.reset();

        self.sim
    }
}
