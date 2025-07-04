use std::hash::Hash;

use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::plugins::step_counter::StepCounter;

// Direction constants for 2D grid movement
const NORTH: IVec2 = IVec2::new(0, -1);
const SOUTH: IVec2 = IVec2::new(0, 1);
const EAST: IVec2 = IVec2::new(1, 0);
const WEST: IVec2 = IVec2::new(-1, 0);
const NORTH_EAST: IVec2 = IVec2::new(1, -1);
const NORTH_WEST: IVec2 = IVec2::new(-1, -1);
const SOUTH_EAST: IVec2 = IVec2::new(1, 1);
const SOUTH_WEST: IVec2 = IVec2::new(-1, 1);

// Direction constants for 3D grid movement (orthogonal only)
const UP: IVec3 = IVec3::new(0, 0, 1);
const DOWN: IVec3 = IVec3::new(0, 0, -1);
const NORTH_3D: IVec3 = IVec3::new(0, -1, 0);
const SOUTH_3D: IVec3 = IVec3::new(0, 1, 0);
const EAST_3D: IVec3 = IVec3::new(1, 0, 0);
const WEST_3D: IVec3 = IVec3::new(-1, 0, 0);

/// A sealed trait for coordinates on a grid.
/// Will be implemented for [`IVec2`] and [`IVec3`].
pub trait GridCoordinates:
    private::Sealed + Clone + Copy + Hash + PartialEq + Eq + Send + Sync + 'static
{
    fn neighbors(&self) -> impl Iterator<Item = Self>;

    fn neighbors_orthogonal(&self) -> impl Iterator<Item = Self>;

    fn in_bounds(&self, bounds: &GridBounds<Self>) -> bool;
}

/// Describes the bounds of a grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridBounds<T: GridCoordinates>
{
    pub min: T,
    pub max: T,
}

impl GridCoordinates for IVec2
{
    fn neighbors(&self) -> impl Iterator<Item = Self>
    {
        const DIRECTIONS: [IVec2; 8] = [
            NORTH_WEST, NORTH, NORTH_EAST, WEST, EAST, SOUTH_WEST, SOUTH, SOUTH_EAST,
        ];
        DIRECTIONS.into_iter().map(move |dir| self + dir)
    }

    fn neighbors_orthogonal(&self) -> impl Iterator<Item = Self>
    {
        const DIRECTIONS: [IVec2; 4] = [NORTH, WEST, EAST, SOUTH];
        DIRECTIONS.into_iter().map(move |dir| self + dir)
    }

    fn in_bounds(&self, bounds: &GridBounds<Self>) -> bool
    {
        bounds.contains(self)
    }
}

impl GridCoordinates for IVec3
{
    fn neighbors(&self) -> impl Iterator<Item = Self>
    {
        // 26 neighbors in 3D (3x3x3 cube minus center)
        (-1..=1).flat_map(move |dx| {
            (-1..=1).flat_map(move |dy| {
                (-1..=1).filter_map(move |dz| {
                    if dx == 0 && dy == 0 && dz == 0
                    {
                        None // Skip center
                    }
                    else
                    {
                        Some(self + Self::new(dx, dy, dz))
                    }
                })
            })
        })
    }

    fn neighbors_orthogonal(&self) -> impl Iterator<Item = Self>
    {
        // 6 orthogonal neighbors in 3D
        const DIRECTIONS: [IVec3; 6] = [WEST_3D, EAST_3D, NORTH_3D, SOUTH_3D, DOWN, UP];
        DIRECTIONS.into_iter().map(move |dir| self + dir)
    }

    fn in_bounds(&self, bounds: &GridBounds<Self>) -> bool
    {
        bounds.contains(self)
    }
}

/// Component representing a position in the spatial grid.
/// Generic over coordinate types that implement the `GridCoordinate` trait.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition<T: GridCoordinates + private::Sealed>(pub T);

// Convenience methods for 2D positions
impl GridPosition<IVec2>
{
    /// Create a new 2D grid position from x, y coordinates.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self
    {
        Self(IVec2::new(x, y))
    }

    pub fn neighbors(&self) -> impl Iterator<Item = Self>
    {
        self.0.neighbors().map(Self)
    }

    pub fn neighbors_orthogonal(&self) -> impl Iterator<Item = Self>
    {
        self.0.neighbors_orthogonal().map(Self)
    }

    #[must_use]
    pub const fn x(&self) -> i32
    {
        self.0.x
    }

    #[must_use]
    pub const fn y(&self) -> i32
    {
        self.0.y
    }
}

// Convenience methods for 3D positions
impl GridPosition<IVec3>
{
    /// Create a new 3D grid position from x, y, z coordinates.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self
    {
        Self(IVec3::new(x, y, z))
    }

    pub fn neighbors(&self) -> impl Iterator<Item = Self>
    {
        self.0.neighbors().map(Self)
    }

    pub fn neighbors_orthogonal(&self) -> impl Iterator<Item = Self>
    {
        self.0.neighbors_orthogonal().map(Self)
    }

    #[must_use]
    pub const fn x(&self) -> i32
    {
        self.0.x
    }

    #[must_use]
    pub const fn y(&self) -> i32
    {
        self.0.y
    }

    #[must_use]
    pub const fn z(&self) -> i32
    {
        self.0.z
    }
}

/// Component that maintains a spatial index for efficient neighbor queries.
/// Generic over coordinate types that implement the `GridCoordinate` trait and component types.
#[derive(Resource)]
pub struct SpatialGrid<T: GridCoordinates, C: Component>
{
    /// Maps grid positions to entities at those positions.
    position_to_entities: HashMap<GridPosition<T>, HashSet<Entity>>,
    /// Maps entities to their grid positions for fast lookups (optimized for Entity keys).
    entity_to_position: EntityHashMap<GridPosition<T>>,
    /// Grid bounds for validation and iteration.
    bounds: Option<GridBounds<T>>,
    /// Phantom data to maintain type association with component C.
    _phantom: std::marker::PhantomData<C>,
}

/// Specific implementations for 2D bounds
impl GridBounds<IVec2>
{
    /// Check if a position is within these bounds.
    ///
    /// # Panics
    ///
    /// If [`Self::min`] is larger than [`Self::max`] along any axis.
    #[must_use]
    pub fn contains(&self, pos: &IVec2) -> bool
    {
        assert!(self.min.x <= self.max.x);
        assert!(self.min.y <= self.max.y);
        (pos.x >= self.min.x && pos.x <= self.max.x) && (pos.y >= self.min.y && pos.y <= self.max.y)
    }
}

/// Specific implementations for 3D bounds
impl GridBounds<IVec3>
{
    /// Check if a position is within these bounds.
    ///
    /// # Panics
    ///
    /// If [`Self::min`] is larger than [`Self::max`] along any axis.
    #[must_use]
    pub fn contains(&self, pos: &IVec3) -> bool
    {
        assert!(self.min.x <= self.max.x);
        assert!(self.min.y <= self.max.y);
        assert!(self.min.z <= self.max.z);
        (pos.x >= self.min.x && pos.x <= self.max.x)
            && (pos.y >= self.min.y && pos.y <= self.max.y)
            && (pos.z >= self.min.z && pos.z <= self.max.z)
    }
}

impl<T: GridCoordinates, C: Component> SpatialGrid<T, C>
{
    #[must_use]
    pub fn new(bounds: Option<GridBounds<T>>) -> Self
    {
        Self {
            position_to_entities: HashMap::default(),
            entity_to_position: EntityHashMap::default(),
            bounds,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub const fn bounds(&self) -> Option<GridBounds<T>>
    {
        self.bounds
    }

    /// Add an entity at a specific grid position.
    pub(crate) fn insert(&mut self, entity: Entity, position: GridPosition<T>)
    {
        // Remove entity from old position if it exists
        self.remove(entity);

        // Insert at new position
        self.position_to_entities
            .entry(position)
            .or_default()
            .insert(entity);
        self.entity_to_position.insert(entity, position);
    }

    /// Remove an entity from the spatial index.
    ///
    /// Returns the position where the entity was located, if it was found.
    pub(crate) fn remove(&mut self, entity: Entity) -> Option<GridPosition<T>>
    {
        if let Some(position) = self.entity_to_position.remove(&entity)
            && let Some(entities) = self.position_to_entities.get_mut(&position)
        {
            entities.remove(&entity);
            if entities.is_empty()
            {
                self.position_to_entities.remove(&position);
            }
            Some(position)
        }
        else
        {
            None
        }
    }

    /// Get all entities at a specific position.
    pub fn entities_at(&self, position: &GridPosition<T>) -> impl Iterator<Item = Entity> + '_
    {
        self.position_to_entities
            .get(position)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Get the position of an entity.
    #[must_use]
    pub fn position_of(&self, entity: Entity) -> Option<GridPosition<T>>
    {
        self.entity_to_position.get(&entity).copied()
    }

    /// Get all entities in the neighborhood of a position (Moore neighborhood).
    pub fn neighbors_of(&self, position: &GridPosition<T>) -> impl Iterator<Item = Entity>
    {
        position
            .0
            .neighbors()
            .filter(|neighbor_pos| {
                self.bounds
                    .is_none_or(|bounds| neighbor_pos.in_bounds(&bounds))
            })
            .map(|p| GridPosition(p))
            .flat_map(|neighbor_pos| {
                self.position_to_entities
                    .get(&neighbor_pos)
                    .into_iter()
                    .flat_map(|set| set.iter().copied())
            })
    }

    /// Get all entities in the orthogonal neighborhood of a position (Von Neumann neighborhood).
    pub fn orthogonal_neighbors_of(
        &self,
        position: &GridPosition<T>,
    ) -> impl Iterator<Item = Entity>
    {
        position
            .0
            .neighbors_orthogonal()
            .filter(|neighbor_pos| {
                self.bounds
                    .is_none_or(|bounds| neighbor_pos.in_bounds(&bounds))
            })
            .map(|p| GridPosition(p))
            .flat_map(|neighbor_pos| {
                self.position_to_entities
                    .get(&neighbor_pos)
                    .into_iter()
                    .flat_map(|set| set.iter().copied())
            })
    }

    /// Clear all entities from the spatial index.
    pub fn clear(&mut self)
    {
        self.position_to_entities.clear();
        self.entity_to_position.clear();
    }

    /// Check if a position is empty (has no entities).
    #[must_use]
    pub fn is_empty(&self, position: &GridPosition<T>) -> bool
    {
        self.position_to_entities
            .get(position)
            .is_none_or(HashSet::is_empty)
    }

    /// Get total number of entities in the grid.
    #[must_use]
    pub fn num_entities(&self) -> usize
    {
        self.entity_to_position.len()
    }
}

/// Plugin that maintains a spatial index for entities with `GridPosition` components.
/// Generic over coordinate types that implement the `GridCoordinate` trait and component types.
pub struct SpatialGridPlugin<T: GridCoordinates, C: Component>
{
    bounds: Option<GridBounds<T>>,
    _phantom: std::marker::PhantomData<(T, C)>,
}

impl<T: GridCoordinates, C: Component> SpatialGridPlugin<T, C>
{
    pub const fn new(bounds: Option<GridBounds<T>>) -> Self
    {
        Self {
            bounds,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn init(app: &mut App, bounds: Option<GridBounds<T>>)
    {
        // Spawn the spatial grid entity directly
        let spatial_grid = SpatialGrid::<T, C>::new(bounds);
        app.world_mut().insert_resource(spatial_grid);
    }
}

impl<T: GridCoordinates, C: Component> Plugin for SpatialGridPlugin<T, C>
{
    fn build(&self, app: &mut App)
    {
        Self::init(app, self.bounds);

        // System to maintain the spatial index
        app.add_systems(
            PreUpdate,
            (
                spatial_grid_reset_system::<T, C>,
                spatial_grid_update_system::<T, C>,
                spatial_grid_cleanup_system::<T, C>,
            )
                .chain(),
        );
    }
}

/// System that resets the spatial grid at the beginning of each simulation.
pub fn spatial_grid_reset_system<T: GridCoordinates, C: Component>(
    mut spatial_grid: ResMut<SpatialGrid<T, C>>,
    step_counter: Res<StepCounter>,
)
{
    // Reset the spatial grid whenever the step counter is 0
    // This should occur on the first step of every simulation
    if **step_counter == 0
    {
        spatial_grid.clear();
    }
}

/// Query for entities with `GridPosition` components that have been added or changed.
type GridPositionQuery<'world, 'state, T, C> =
    Query<'world, 'state, (Entity, &'static GridPosition<T>), (Changed<GridPosition<T>>, With<C>)>;

/// System that updates the spatial grid when entities with `GridPosition` are added or moved.
pub fn spatial_grid_update_system<T: GridCoordinates, C: Component>(
    mut spatial_grid: ResMut<SpatialGrid<T, C>>,
    query: GridPositionQuery<T, C>,
)
{
    for (entity, position) in &query
    {
        spatial_grid.remove(entity);
        spatial_grid.insert(entity, *position);
    }
}

/// System that removes entities from the spatial grid when they no longer have `GridPosition`.
pub fn spatial_grid_cleanup_system<T: GridCoordinates, C: Component>(
    mut spatial_grid: ResMut<SpatialGrid<T, C>>,
    mut removed: RemovedComponents<GridPosition<T>>,
)
{
    for entity in removed.read()
    {
        spatial_grid.remove(entity);
    }
}

// Type aliases for convenience
/// 2D spatial grid using `IVec2` coordinates.
pub type SpatialGrid2D<C> = SpatialGrid<IVec2, C>;

/// 3D spatial grid using `IVec3` coordinates.
pub type SpatialGrid3D<C> = SpatialGrid<IVec3, C>;

/// 2D grid position using `IVec2` coordinates.
pub type GridPosition2D = GridPosition<IVec2>;

/// 3D grid position using `IVec3` coordinates.
pub type GridPosition3D = GridPosition<IVec3>;

/// 2D grid bounds using `IRect`.
pub type GridBounds2D = GridBounds<IVec2>;

/// 3D grid bounds using custom `Bounds3D`.
pub type GridBounds3D = GridBounds<IVec3>;

/// Private module to enforce the sealed trait pattern.
mod private
{
    pub trait Sealed {}
    impl Sealed for bevy::prelude::IVec2 {}
    impl Sealed for bevy::prelude::IVec3 {}
}
