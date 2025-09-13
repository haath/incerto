use bevy::{
    app::ScheduleRunnerPlugin,
    ecs::{query::QueryFilter, system::ScheduleSystem},
    prelude::*,
};

use crate::{
    BuilderError, Identifier, Sample, SampleAggregate,
    plugins::{
        AggregateTimeSeriesPlugin, GridBounds, GridCoordinates, SampleInterval, SpatialGridPlugin,
        StepCounterPlugin, TimeSeries, TimeSeriesPlugin,
    },
    prelude::{GridBounds2D, GridBounds3D},
    simulation::Simulation,
    spawner::Spawner,
};

type SpawnFn = Box<dyn Fn(&mut Spawner)>;

/// Builder type used to construct a [`Simulation`] object.
///
/// The builder is used to logically separate the construction of a simulation with its execution.
/// Once built, a [`Simulation`] object may be reused in order to intermitently run simulation steps,
/// restart the simulation from the beginning, collect results and so on.
pub struct SimulationBuilder
{
    app: App,
    spawners: Vec<SpawnFn>,
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

        app.add_plugins(TaskPoolPlugin::default())
            .add_plugins(ScheduleRunnerPlugin::run_once())
            .add_plugins(StepCounterPlugin);

        app.update();

        Self {
            app,
            spawners: Vec::new(),
        }
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
        self.app.add_systems(Update, systems);
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
        self.app.add_event::<E>();
        self
    }

    /// Add a spatial grid for a specific component type to the simulation.
    ///
    /// This creates a spatial index for entities that have both [`super::GridPosition<T>`] and the specified component `C`.
    /// Multiple spatial grids can coexist, one for each component type `C`.
    ///
    /// The spatial grid can be access by the user using the [`super::SpatialGrid<T, C>`] bevy resource.
    ///
    /// Optionally, if `Some(bounds)` are given, a panic will be raised if an entity has a [`super::GridPosition`]
    /// outside the bounds.
    ///
    /// Example:
    /// ```
    /// # use bevy::prelude::IVec2;
    /// # use incerto::prelude::*;
    /// #[derive(Component)]
    /// struct Person;
    ///
    /// #[derive(Component)]
    /// struct Vehicle;
    ///
    /// let bounds = GridBounds2D {
    ///     min: IVec2::new(0, 0),
    ///     max: IVec2::new(99, 99),
    /// };
    /// let simulation = SimulationBuilder::new()
    ///     .add_spatial_grid::<IVec2, Person>(Some(bounds))
    ///     .add_spatial_grid::<IVec2, Vehicle>(None)
    ///     .build();
    /// ```
    #[must_use]
    pub fn add_spatial_grid<T: GridCoordinates, C: Component>(
        mut self,
        bounds: Option<GridBounds<T>>,
    ) -> Self
    {
        self.app.add_plugins(SpatialGridPlugin::<T, C>::new(bounds));
        self
    }

    /// Adds a 2D spatial grid for a specific component type to the simulation.
    ///
    /// See [`Self::add_spatial_grid`] for details.
    #[must_use]
    pub fn add_spatial_grid_2d<C: Component>(self, bounds: Option<GridBounds2D>) -> Self
    {
        self.add_spatial_grid::<IVec2, C>(bounds)
    }

    /// Adds a 3D spatial grid for a specific component type to the simulation.
    ///
    /// See [`Self::add_spatial_grid`] for details.
    #[must_use]
    pub fn add_spatial_grid_3d<C: Component>(self, bounds: Option<GridBounds3D>) -> Self
    {
        self.add_spatial_grid::<IVec3, C>(bounds)
    }

    /// Add an entity spawner function to the simulation.
    ///
    /// In the beginning of every simulation, each of the spawner functions added here
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
        self.spawners.push(Box::new(entity_spawner));
        self
    }

    /// Sets up the recording of a time series.
    ///
    /// The values in the time series will be values of type `O`
    /// sampled from components `C`, on any entities identified by `I`
    /// in the simulation according to the implementation of [`Sample<O>`] for `C`.
    ///
    /// The sampling will occur once every `sample_interval` steps.
    /// Specifically at the end of the step, after all user-defined systems have run.
    ///
    /// Note that it is currently not allowed to record more than one time series
    /// with the same pair of component (`C`),value (`O`), and identifier (`I`).
    ///
    /// For more details see the documentation of:
    /// * [`Sample`]
    /// * [`Identifier`]
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
    pub fn record_time_series<C, I, O>(
        mut self,
        sample_interval: usize,
    ) -> Result<Self, BuilderError>
    where
        C: Sample<O>,
        I: Identifier,
        O: Send + Sync + 'static,
    {
        assert!(sample_interval > 0);

        let world = self.app.world();
        if world.get_resource::<SampleInterval<C, I, O>>().is_some()
        {
            // More than one time series recording for the same C, F, O is not possible.
            return Err(BuilderError::TimeSeriesRecordingConflict);
        }

        self.app
            .add_plugins(TimeSeriesPlugin::<C, I, O>::new(sample_interval));
        Ok(self)
    }

    /// Sets up the recording of an aggregate time series.
    ///
    /// The values in the time series will be values of type `O`
    /// sampled from all components `C` in the simulation according to the
    /// implementation of [`SampleAggregate<O>`] for `C`.
    ///
    /// The sampling will occur once every `sample_interval` steps.
    /// Specifically at the end of the step, after all user-defined systems have run.
    ///
    /// Note that it is currently not possible to record more than one time series
    /// with the same pair of component (`C`),value (`O`), and filter (`F`).
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
    pub fn record_aggregate_time_series<C, O>(
        self,
        sample_interval: usize,
    ) -> Result<Self, BuilderError>
    where
        C: SampleAggregate<O>,
        O: Send + Sync + 'static,
    {
        self.record_aggregate_time_series_filtered::<C, (), O>(sample_interval)
    }

    /// Sets up the recording of an aggregate time series.
    ///
    /// The values in the time series will be values of type `O`
    /// sampled from components `C`, from all entities selected by the filter `F`
    /// in the simulation according to the implementation of [`SampleAggregate<O>`] for `C`.
    ///
    /// The sampling will occur once every `sample_interval` steps.
    /// Specifically at the end of the step, after all user-defined systems have run.
    ///
    /// Note that it is currently not allowed to record more than one time series
    /// with the same pair of component (`C`),value (`O`), and filter (`F`).
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
    pub fn record_aggregate_time_series_filtered<C, F, O>(
        mut self,
        sample_interval: usize,
    ) -> Result<Self, BuilderError>
    where
        C: SampleAggregate<O>,
        F: QueryFilter + Send + Sync + 'static,
        O: Send + Sync + 'static,
    {
        assert!(sample_interval > 0);

        let world = self.app.world();
        if world.get_resource::<TimeSeries<C, F, O>>().is_some()
        {
            // More than one time series recording for the same C, F, O is not possible.
            return Err(BuilderError::TimeSeriesRecordingConflict);
        }

        self.app
            .add_plugins(AggregateTimeSeriesPlugin::<C, F, O>::new(sample_interval));
        Ok(self)
    }

    /// Adds a bevy [`Resource`] to the simulation.
    ///
    /// This can later be accessed in user-defined systems using [`Res<R>`] and [`ResMut<R>`] arguments.
    #[must_use]
    pub fn add_resource<R: Resource>(mut self, resource: R) -> Self
    {
        self.app.insert_resource(resource);
        self
    }

    pub fn build(mut self) -> Simulation
    {
        // spawn all entities
        let mut spawner = Spawner(self.app.world_mut());
        for spawn_fn in &self.spawners
        {
            spawn_fn(&mut spawner);
        }

        Simulation { app: self.app }
    }
}
