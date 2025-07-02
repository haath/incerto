use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

/// Component representing a position in the spatial grid.
/// Built on top of Bevy's `IVec2` for compatibility with the Bevy ecosystem.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct GridPosition(pub IVec2);

impl GridPosition
{
    // Direction constants for cleaner neighbor calculations
    const NORTH: IVec2 = IVec2::new(0, -1);
    const SOUTH: IVec2 = IVec2::new(0, 1);
    const EAST: IVec2 = IVec2::new(1, 0);
    const WEST: IVec2 = IVec2::new(-1, 0);
    const NORTH_EAST: IVec2 = IVec2::new(1, -1);
    const NORTH_WEST: IVec2 = IVec2::new(-1, -1);
    const SOUTH_EAST: IVec2 = IVec2::new(1, 1);
    const SOUTH_WEST: IVec2 = IVec2::new(-1, 1);

    /// Create a new grid position from x, y coordinates.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self
    {
        Self(IVec2::new(x, y))
    }

    /// Get the x coordinate.
    #[must_use]
    pub const fn x(&self) -> i32
    {
        self.0.x
    }

    /// Get the y coordinate.
    #[must_use]
    pub const fn y(&self) -> i32
    {
        self.0.y
    }

    /// Get all neighboring positions (Moore neighborhood).
    #[must_use]
    pub fn neighbors(&self) -> impl Iterator<Item = Self>
    {
        const DIRECTIONS: [IVec2; 8] = [
            GridPosition::NORTH_WEST,
            GridPosition::NORTH,
            GridPosition::NORTH_EAST,
            GridPosition::WEST,
            GridPosition::EAST,
            GridPosition::SOUTH_WEST,
            GridPosition::SOUTH,
            GridPosition::SOUTH_EAST,
        ];
        let base = self.0;
        DIRECTIONS.into_iter().map(move |dir| Self(base + dir))
    }

    /// Get orthogonal neighboring positions (Von Neumann neighborhood).
    #[must_use]
    pub fn neighbors_orthogonal(&self) -> impl Iterator<Item = Self>
    {
        const DIRECTIONS: [IVec2; 4] = [
            GridPosition::NORTH,
            GridPosition::WEST,
            GridPosition::EAST,
            GridPosition::SOUTH,
        ];
        let base = self.0;
        DIRECTIONS.into_iter().map(move |dir| Self(base + dir))
    }
}

/// Resource that maintains a spatial index for efficient neighbor queries.
#[derive(Resource, Default)]
pub struct SpatialGrid
{
    /// Maps grid positions to entities at those positions.
    position_to_entities: HashMap<GridPosition, HashSet<Entity>>,
    /// Maps entities to their grid positions for fast lookups (optimized for Entity keys).
    entity_to_position: EntityHashMap<GridPosition>,
    /// Grid bounds for validation and iteration.
    bounds: Option<GridBounds>,
}

/// Grid bounds representing the valid area for grid positions.
///
/// Built on top of Bevy's `IRect` for compatibility, but maintains grid-specific semantics
/// where width/height represent cell counts rather than geometric dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct GridBounds(IRect);

impl GridBounds
{
    #[must_use]
    pub fn new(min_x: i32, max_x: i32, min_y: i32, max_y: i32) -> Self
    {
        Self(IRect::new(min_x, min_y, max_x, max_y))
    }

    #[must_use]
    pub fn contains(&self, pos: &GridPosition) -> bool
    {
        self.0.contains(pos.0)
    }

    /// Get the width in grid cells (number of columns).
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Safe: assumes well-formed bounds
    pub fn width(&self) -> u32
    {
        (self.0.width() + 1) as u32
    }

    /// Get the height in grid cells (number of rows).
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // Safe: assumes well-formed bounds
    pub fn height(&self) -> u32
    {
        (self.0.height() + 1) as u32
    }

    /// Get the total number of grid cells.
    #[must_use]
    pub fn total_cells(&self) -> u32
    {
        self.width() * self.height()
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
}

impl From<IRect> for GridBounds
{
    fn from(rect: IRect) -> Self
    {
        Self(rect)
    }
}

impl From<GridBounds> for IRect
{
    fn from(bounds: GridBounds) -> Self
    {
        bounds.0
    }
}

impl SpatialGrid
{
    #[must_use]
    pub fn new() -> Self
    {
        Self::default()
    }

    #[must_use]
    pub fn with_bounds(bounds: GridBounds) -> Self
    {
        Self {
            bounds: Some(bounds),
            ..default()
        }
    }

    pub const fn set_bounds(&mut self, bounds: GridBounds)
    {
        self.bounds = Some(bounds);
    }

    #[must_use]
    pub const fn bounds(&self) -> Option<GridBounds>
    {
        self.bounds
    }

    /// Add an entity at a specific grid position.
    fn insert(&mut self, entity: Entity, position: GridPosition)
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
    fn remove(&mut self, entity: Entity) -> Option<GridPosition>
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
    pub fn entities_at(&self, position: &GridPosition) -> impl Iterator<Item = Entity> + '_
    {
        self.position_to_entities
            .get(position)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Get the position of an entity.
    #[must_use]
    pub fn position_of(&self, entity: Entity) -> Option<GridPosition>
    {
        self.entity_to_position.get(&entity).copied()
    }

    /// Get all entities in the 8-connected neighborhood of a position.
    #[must_use]
    pub fn neighbors_of<'a>(
        &'a self,
        position: &'a GridPosition,
    ) -> impl Iterator<Item = Entity> + 'a
    {
        position
            .neighbors()
            .filter(move |neighbor_pos| {
                self.bounds
                    .map_or(true, |bounds| bounds.contains(neighbor_pos))
            })
            .flat_map(move |neighbor_pos| {
                self.position_to_entities
                    .get(&neighbor_pos)
                    .into_iter()
                    .flat_map(|set| set.iter().copied())
            })
    }

    /// Get all entities in the 4-connected orthogonal neighborhood of a position.
    #[must_use]
    pub fn orthogonal_neighbors_of<'a>(
        &'a self,
        position: &'a GridPosition,
    ) -> impl Iterator<Item = Entity> + 'a
    {
        position
            .neighbors_orthogonal()
            .filter(move |neighbor_pos| {
                self.bounds
                    .map_or(true, |bounds| bounds.contains(neighbor_pos))
            })
            .flat_map(move |neighbor_pos| {
                self.position_to_entities
                    .get(&neighbor_pos)
                    .into_iter()
                    .flat_map(|set| set.iter().copied())
            })
    }

    /// Get all entities within a Manhattan distance of a position.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn entities_within_distance(&self, center: &GridPosition, distance: u32) -> Vec<Entity>
    {
        let distance_i32 = distance as i32;
        let center_pos = *center;

        (center.x - distance_i32..=center.x + distance_i32)
            .flat_map(move |x| {
                (center.y - distance_i32..=center.y + distance_i32)
                    .map(move |y| GridPosition::new(x, y))
            })
            .filter(move |pos| (pos.0 - center_pos.0).abs().element_sum() as u32 <= distance)
            .filter(move |pos| self.bounds.map_or(true, |bounds| bounds.contains(&pos)))
            .flat_map(move |pos| self.entities_at(&pos).collect::<Vec<_>>())
            .collect()
    }

    /// Clear all entities from the spatial index.
    pub fn clear(&mut self)
    {
        self.position_to_entities.clear();
        self.entity_to_position.clear();
    }

    /// Check if a position is empty (has no entities).
    #[must_use]
    pub fn is_empty(&self, position: &GridPosition) -> bool
    {
        self.position_to_entities
            .get(position)
            .map_or(true, |set| set.is_empty())
    }

    /// Get total number of entities in the grid.
    #[must_use]
    pub fn num_entities(&self) -> usize
    {
        self.entity_to_position.len()
    }
}

/// Plugin that maintains a spatial index for entities with `GridPosition` components.
pub struct SpatialGridPlugin
{
    bounds: Option<GridBounds>,
}

impl Default for SpatialGridPlugin
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl SpatialGridPlugin
{
    pub const fn new() -> Self
    {
        Self { bounds: None }
    }

    pub const fn with_bounds(bounds: GridBounds) -> Self
    {
        Self {
            bounds: Some(bounds),
        }
    }

    pub fn init(app: &mut App, bounds: Option<GridBounds>)
    {
        let spatial_grid = bounds.map_or_else(SpatialGrid::new, SpatialGrid::with_bounds);
        app.insert_resource(spatial_grid);
    }
}

impl Plugin for SpatialGridPlugin
{
    fn build(&self, app: &mut App)
    {
        Self::init(app, self.bounds);

        // System to maintain the spatial index
        app.add_systems(
            PreUpdate,
            (spatial_grid_update_system, spatial_grid_cleanup_system).chain(),
        );
    }
}

/// Query for entities with `GridPosition` components that have been added or changed.
type GridPositionQuery<'world, 'state> =
    Query<'world, 'state, (Entity, &'static GridPosition), Changed<GridPosition>>;

/// System that updates the spatial grid when entities with `GridPosition` are added or moved.
pub fn spatial_grid_update_system(mut spatial_grid: ResMut<SpatialGrid>, query: GridPositionQuery)
{
    for (entity, position) in &query
    {
        spatial_grid.insert(entity, *position);
    }
}

/// System that removes entities from the spatial grid when they no longer have `GridPosition`.
pub fn spatial_grid_cleanup_system(
    mut spatial_grid: ResMut<SpatialGrid>,
    mut removed: RemovedComponents<GridPosition>,
)
{
    for entity in removed.read()
    {
        spatial_grid.remove(entity);
    }
}
