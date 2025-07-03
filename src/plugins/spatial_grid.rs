use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

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

/// Sealed trait for grid coordinate types that can be used with the spatial grid system.
///
/// This trait abstracts over 2D and 3D coordinate types, allowing the same spatial grid
/// implementation to work with both `IVec2` and `IVec3` coordinates.
pub trait GridCoordinate:
    Copy
    + Clone
    + PartialEq
    + Eq
    + std::hash::Hash
    + std::fmt::Debug
    + Send
    + Sync
    + 'static
    + private::Sealed
{
    /// The bounds type for this coordinate system (e.g., `IRect` for 2D, custom bounds for 3D).
    type Bounds: Copy + Clone + PartialEq + Eq + std::fmt::Debug + Send + Sync + 'static;

    /// Create a new coordinate from individual components.
    /// For 2D: new(x, y), for 3D: new(x, y, z)
    fn new(x: i32, y: i32, z: i32) -> Self;

    /// Get the x coordinate.
    fn x(self) -> i32;

    /// Get the y coordinate.
    fn y(self) -> i32;

    /// Get the z coordinate (returns 0 for 2D coordinates).
    fn z(self) -> i32;

    /// Calculate Manhattan distance between two coordinates.
    fn manhattan_distance(self, other: Self) -> u32;

    /// Get all neighboring coordinates (Moore neighborhood).
    fn neighbors(self) -> Box<dyn Iterator<Item = Self>>;

    /// Get orthogonal neighboring coordinates (Von Neumann neighborhood).
    fn neighbors_orthogonal(self) -> Box<dyn Iterator<Item = Self>>;

    /// Create bounds from min/max coordinates.
    fn create_bounds(min: Self, max: Self) -> Self::Bounds;

    /// Check if this coordinate is within the given bounds.
    fn within_bounds(self, bounds: &Self::Bounds) -> bool;

    /// Generate all coordinates within a Manhattan distance of this coordinate.
    fn coordinates_within_distance(self, distance: u32) -> Box<dyn Iterator<Item = Self>>;
}

/// Private module to enforce the sealed trait pattern.
mod private
{
    pub trait Sealed {}
    impl Sealed for bevy::prelude::IVec2 {}
    impl Sealed for bevy::prelude::IVec3 {}
}

/// 2D bounds type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds2D(pub IRect);

/// 3D bounds type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds3D
{
    pub min: IVec3,
    pub max: IVec3,
}

impl GridCoordinate for IVec2
{
    type Bounds = Bounds2D;

    fn new(x: i32, y: i32, _z: i32) -> Self
    {
        Self::new(x, y)
    }

    fn x(self) -> i32
    {
        self.x
    }

    fn y(self) -> i32
    {
        self.y
    }

    fn z(self) -> i32
    {
        0
    }

    fn manhattan_distance(self, other: Self) -> u32
    {
        #[allow(clippy::cast_sign_loss)]
        {
            (self - other).abs().element_sum() as u32
        }
    }

    fn neighbors(self) -> Box<dyn Iterator<Item = Self>>
    {
        const DIRECTIONS: [IVec2; 8] = [
            NORTH_WEST, NORTH, NORTH_EAST, WEST, EAST, SOUTH_WEST, SOUTH, SOUTH_EAST,
        ];
        Box::new(DIRECTIONS.into_iter().map(move |dir| self + dir))
    }

    fn neighbors_orthogonal(self) -> Box<dyn Iterator<Item = Self>>
    {
        const DIRECTIONS: [IVec2; 4] = [NORTH, WEST, EAST, SOUTH];
        Box::new(DIRECTIONS.into_iter().map(move |dir| self + dir))
    }

    fn create_bounds(min: Self, max: Self) -> Self::Bounds
    {
        Bounds2D(IRect::new(min.x, min.y, max.x, max.y))
    }

    fn within_bounds(self, bounds: &Self::Bounds) -> bool
    {
        bounds.0.contains(self)
    }

    fn coordinates_within_distance(self, distance: u32) -> Box<dyn Iterator<Item = Self>>
    {
        #[allow(clippy::cast_possible_wrap)]
        let distance_i32 = distance as i32;
        Box::new(
            (self.x - distance_i32..=self.x + distance_i32)
                .flat_map(move |x| {
                    (self.y - distance_i32..=self.y + distance_i32).map(move |y| Self::new(x, y))
                })
                .filter(move |pos| self.manhattan_distance(*pos) <= distance),
        )
    }
}

impl GridCoordinate for IVec3
{
    type Bounds = Bounds3D;

    fn new(x: i32, y: i32, z: i32) -> Self
    {
        Self::new(x, y, z)
    }

    fn x(self) -> i32
    {
        self.x
    }

    fn y(self) -> i32
    {
        self.y
    }

    fn z(self) -> i32
    {
        self.z
    }

    fn manhattan_distance(self, other: Self) -> u32
    {
        #[allow(clippy::cast_sign_loss)]
        {
            (self - other).abs().element_sum() as u32
        }
    }

    fn neighbors(self) -> Box<dyn Iterator<Item = Self>>
    {
        // 26 neighbors in 3D (3x3x3 cube minus center)
        Box::new((-1..=1).flat_map(move |dx| {
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
        }))
    }

    fn neighbors_orthogonal(self) -> Box<dyn Iterator<Item = Self>>
    {
        // 6 orthogonal neighbors in 3D
        const DIRECTIONS: [IVec3; 6] = [WEST_3D, EAST_3D, NORTH_3D, SOUTH_3D, DOWN, UP];
        Box::new(DIRECTIONS.into_iter().map(move |dir| self + dir))
    }

    fn create_bounds(min: Self, max: Self) -> Self::Bounds
    {
        Bounds3D { min, max }
    }

    fn within_bounds(self, bounds: &Self::Bounds) -> bool
    {
        self.x >= bounds.min.x
            && self.x <= bounds.max.x
            && self.y >= bounds.min.y
            && self.y <= bounds.max.y
            && self.z >= bounds.min.z
            && self.z <= bounds.max.z
    }

    fn coordinates_within_distance(self, distance: u32) -> Box<dyn Iterator<Item = Self>>
    {
        #[allow(clippy::cast_possible_wrap)]
        let distance_i32 = distance as i32;
        Box::new(
            (self.x - distance_i32..=self.x + distance_i32)
                .flat_map(move |x| {
                    (self.y - distance_i32..=self.y + distance_i32).flat_map(move |y| {
                        (self.z - distance_i32..=self.z + distance_i32)
                            .map(move |z| Self::new(x, y, z))
                    })
                })
                .filter(move |pos| self.manhattan_distance(*pos) <= distance),
        )
    }
}

/// Component representing a position in the spatial grid.
/// Generic over coordinate types that implement the `GridCoordinate` trait.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct GridPosition<T: GridCoordinate>(pub T);

impl<T: GridCoordinate> GridPosition<T>
{
    /// Get the x coordinate.
    #[must_use]
    pub fn x(&self) -> i32
    {
        self.0.x()
    }

    /// Get the y coordinate.
    #[must_use]
    pub fn y(&self) -> i32
    {
        self.0.y()
    }

    /// Get the z coordinate.
    #[must_use]
    pub fn z(&self) -> i32
    {
        self.0.z()
    }

    /// Get all neighboring positions (Moore neighborhood).
    pub fn neighbors(&self) -> impl Iterator<Item = Self>
    {
        let neighbors = self.0.neighbors();
        // Convert Box<dyn Iterator> to a concrete type by collecting and iterating
        let neighbors_vec: Vec<T> = neighbors.collect();
        neighbors_vec.into_iter().map(Self)
    }

    /// Get orthogonal neighboring positions (Von Neumann neighborhood).
    pub fn neighbors_orthogonal(&self) -> impl Iterator<Item = Self>
    {
        let neighbors = self.0.neighbors_orthogonal();
        // Convert Box<dyn Iterator> to a concrete type by collecting and iterating
        let neighbors_vec: Vec<T> = neighbors.collect();
        neighbors_vec.into_iter().map(Self)
    }

    /// Calculate Manhattan distance to another position.
    #[must_use]
    pub fn manhattan_distance(&self, other: &Self) -> u32
    {
        self.0.manhattan_distance(other.0)
    }
}

// Convenience methods for 2D positions
impl GridPosition<IVec2>
{
    /// Create a new 2D grid position from x, y coordinates.
    #[must_use]
    pub const fn new_2d(x: i32, y: i32) -> Self
    {
        Self(IVec2::new(x, y))
    }
}

// Convenience methods for 3D positions
impl GridPosition<IVec3>
{
    /// Create a new 3D grid position from x, y, z coordinates.
    #[must_use]
    pub const fn new_3d(x: i32, y: i32, z: i32) -> Self
    {
        Self(IVec3::new(x, y, z))
    }
}

/// Resource that maintains a spatial index for efficient neighbor queries.
/// Generic over coordinate types that implement the `GridCoordinate` trait.
#[derive(Resource)]
pub struct SpatialGrid<T: GridCoordinate>
{
    /// Maps grid positions to entities at those positions.
    position_to_entities: HashMap<GridPosition<T>, HashSet<Entity>>,
    /// Maps entities to their grid positions for fast lookups (optimized for Entity keys).
    entity_to_position: EntityHashMap<GridPosition<T>>,
    /// Grid bounds for validation and iteration.
    bounds: Option<GridBounds<T>>,
}

/// Grid bounds representing the valid area for grid positions.
/// Generic over coordinate types that implement the `GridCoordinate` trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridBounds<T: GridCoordinate>(pub T::Bounds);

impl<T: GridCoordinate> GridBounds<T>
{
    /// Create new bounds from min and max coordinates.
    #[must_use]
    pub fn new(min: GridPosition<T>, max: GridPosition<T>) -> Self
    {
        Self(T::create_bounds(min.0, max.0))
    }

    /// Check if a position is within these bounds.
    #[must_use]
    pub fn contains(&self, pos: &GridPosition<T>) -> bool
    {
        pos.0.within_bounds(&self.0)
    }
}

// Specific implementations for 2D bounds
impl GridBounds<IVec2>
{
    /// Create 2D bounds from coordinate values.
    #[must_use]
    pub fn new_2d(min_x: i32, max_x: i32, min_y: i32, max_y: i32) -> Self
    {
        Self(Bounds2D(IRect::new(min_x, min_y, max_x, max_y)))
    }

    /// Get the width in grid cells (number of columns).
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Safe: assumes well-formed bounds
    pub fn width(&self) -> u32
    {
        (self.0.0.width() + 1) as u32
    }

    /// Get the height in grid cells (number of rows).
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Safe: assumes well-formed bounds
    pub fn height(&self) -> u32
    {
        (self.0.0.height() + 1) as u32
    }

    /// Get the total number of grid cells.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn total_cells(&self) -> u32
    {
        self.width() * self.height()
    }

    /// Get the minimum x coordinate.
    #[must_use]
    pub const fn min_x(&self) -> i32
    {
        self.0.0.min.x
    }

    /// Get the maximum x coordinate.
    #[must_use]
    pub const fn max_x(&self) -> i32
    {
        self.0.0.max.x
    }

    /// Get the minimum y coordinate.
    #[must_use]
    pub const fn min_y(&self) -> i32
    {
        self.0.0.min.y
    }

    /// Get the maximum y coordinate.
    #[must_use]
    pub const fn max_y(&self) -> i32
    {
        self.0.0.max.y
    }
}

// Specific implementations for 3D bounds
impl GridBounds<IVec3>
{
    /// Create 3D bounds from coordinate values.
    #[must_use]
    pub const fn new_3d(
        min_x: i32,
        max_x: i32,
        min_y: i32,
        max_y: i32,
        min_z: i32,
        max_z: i32,
    ) -> Self
    {
        Self(Bounds3D {
            min: IVec3::new(min_x, min_y, min_z),
            max: IVec3::new(max_x, max_y, max_z),
        })
    }

    /// Get the width in grid cells (number of columns).
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Safe: assumes well-formed bounds
    pub const fn width(&self) -> u32
    {
        (self.0.max.x - self.0.min.x + 1) as u32
    }

    /// Get the height in grid cells (number of rows).
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Safe: assumes well-formed bounds
    pub const fn height(&self) -> u32
    {
        (self.0.max.y - self.0.min.y + 1) as u32
    }

    /// Get the depth in grid cells (number of layers).
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Safe: assumes well-formed bounds
    pub const fn depth(&self) -> u32
    {
        (self.0.max.z - self.0.min.z + 1) as u32
    }

    /// Get the total number of grid cells.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn total_cells(&self) -> u32
    {
        self.width() * self.height() * self.depth()
    }

    /// Get the minimum x coordinate.
    #[must_use]
    pub const fn min_x(&self) -> i32
    {
        self.0.min.x
    }

    /// Get the maximum x coordinate.
    #[must_use]
    pub const fn max_x(&self) -> i32
    {
        self.0.max.x
    }

    /// Get the minimum y coordinate.
    #[must_use]
    pub const fn min_y(&self) -> i32
    {
        self.0.min.y
    }

    /// Get the maximum y coordinate.
    #[must_use]
    pub const fn max_y(&self) -> i32
    {
        self.0.max.y
    }

    /// Get the minimum z coordinate.
    #[must_use]
    pub const fn min_z(&self) -> i32
    {
        self.0.min.z
    }

    /// Get the maximum z coordinate.
    #[must_use]
    pub const fn max_z(&self) -> i32
    {
        self.0.max.z
    }
}

impl From<IRect> for GridBounds<IVec2>
{
    fn from(rect: IRect) -> Self
    {
        Self(Bounds2D(rect))
    }
}

impl From<GridBounds<IVec2>> for IRect
{
    fn from(bounds: GridBounds<IVec2>) -> Self
    {
        bounds.0.0
    }
}

impl<T: GridCoordinate> SpatialGrid<T>
{
    #[must_use]
    pub fn new(bounds: Option<GridBounds<T>>) -> Self
    {
        Self {
            position_to_entities: HashMap::default(),
            entity_to_position: EntityHashMap::default(),
            bounds,
        }
    }

    pub const fn set_bounds(&mut self, bounds: GridBounds<T>)
    {
        self.bounds = Some(bounds);
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
    pub fn neighbors_of<'a>(
        &'a self,
        position: &'a GridPosition<T>,
    ) -> impl Iterator<Item = Entity> + 'a
    {
        position
            .neighbors()
            .filter(move |neighbor_pos| {
                self.bounds
                    .is_none_or(|bounds| bounds.contains(neighbor_pos))
            })
            .flat_map(move |neighbor_pos| {
                self.position_to_entities
                    .get(&neighbor_pos)
                    .into_iter()
                    .flat_map(|set| set.iter().copied())
            })
    }

    /// Get all entities in the orthogonal neighborhood of a position (Von Neumann neighborhood).
    pub fn orthogonal_neighbors_of<'a>(
        &'a self,
        position: &'a GridPosition<T>,
    ) -> impl Iterator<Item = Entity> + 'a
    {
        position
            .neighbors_orthogonal()
            .filter(move |neighbor_pos| {
                self.bounds
                    .is_none_or(|bounds| bounds.contains(neighbor_pos))
            })
            .flat_map(move |neighbor_pos| {
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
/// Generic over coordinate types that implement the `GridCoordinate` trait.
pub struct SpatialGridPlugin<T: GridCoordinate>
{
    bounds: Option<GridBounds<T>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: GridCoordinate> SpatialGridPlugin<T>
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
        let spatial_grid = SpatialGrid::new(bounds);
        app.insert_resource(spatial_grid);
    }
}

impl<T: GridCoordinate> Plugin for SpatialGridPlugin<T>
{
    fn build(&self, app: &mut App)
    {
        Self::init(app, self.bounds);

        // System to maintain the spatial index
        app.add_systems(
            PreUpdate,
            (
                spatial_grid_update_system::<T>,
                spatial_grid_cleanup_system::<T>,
            )
                .chain(),
        );
    }
}

/// Query for entities with `GridPosition` components that have been added or changed.
type GridPositionQuery<'world, 'state, T> =
    Query<'world, 'state, (Entity, &'static GridPosition<T>), Changed<GridPosition<T>>>;

/// System that updates the spatial grid when entities with `GridPosition` are added or moved.
pub fn spatial_grid_update_system<T: GridCoordinate>(
    mut spatial_grid: ResMut<SpatialGrid<T>>,
    query: GridPositionQuery<T>,
)
{
    for (entity, position) in &query
    {
        spatial_grid.insert(entity, *position);
    }
}

/// System that removes entities from the spatial grid when they no longer have `GridPosition`.
pub fn spatial_grid_cleanup_system<T: GridCoordinate>(
    mut spatial_grid: ResMut<SpatialGrid<T>>,
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
pub type SpatialGrid2D = SpatialGrid<IVec2>;

/// 3D spatial grid using `IVec3` coordinates.
pub type SpatialGrid3D = SpatialGrid<IVec3>;

/// 2D grid position using `IVec2` coordinates.
pub type GridPosition2D = GridPosition<IVec2>;

/// 3D grid position using `IVec3` coordinates.
pub type GridPosition3D = GridPosition<IVec3>;

/// 2D grid bounds using `IRect`.
pub type GridBounds2D = GridBounds<IVec2>;

/// 3D grid bounds using custom `Bounds3D`.
pub type GridBounds3D = GridBounds<IVec3>;

/// 2D spatial grid plugin.
pub type SpatialGridPlugin2D = SpatialGridPlugin<IVec2>;

/// 3D spatial grid plugin.
pub type SpatialGridPlugin3D = SpatialGridPlugin<IVec3>;
