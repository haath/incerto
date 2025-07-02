use std::collections::HashMap;

use bevy::prelude::*;

/// Component representing a position in the spatial grid.
/// Built on top of Bevy's `IVec2` for compatibility with the Bevy ecosystem.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct GridPosition(IVec2);

impl GridPosition
{
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self
    {
        Self(IVec2::new(x, y))
    }

    /// Get the 8 neighboring positions (Moore neighborhood).
    #[must_use]
    pub const fn neighbors(&self) -> [Self; 8]
    {
        [
            Self(IVec2::new(self.0.x - 1, self.0.y - 1)), // top-left
            Self(IVec2::new(self.0.x, self.0.y - 1)),     // top
            Self(IVec2::new(self.0.x + 1, self.0.y - 1)), // top-right
            Self(IVec2::new(self.0.x - 1, self.0.y)),     // left
            Self(IVec2::new(self.0.x + 1, self.0.y)),     // right
            Self(IVec2::new(self.0.x - 1, self.0.y + 1)), // bottom-left
            Self(IVec2::new(self.0.x, self.0.y + 1)),     // bottom
            Self(IVec2::new(self.0.x + 1, self.0.y + 1)), // bottom-right
        ]
    }

    /// Get the 4 orthogonal neighboring positions (Von Neumann neighborhood).
    #[must_use]
    pub const fn orthogonal_neighbors(&self) -> [Self; 4]
    {
        [
            Self(IVec2::new(self.0.x, self.0.y - 1)), // top
            Self(IVec2::new(self.0.x - 1, self.0.y)), // left
            Self(IVec2::new(self.0.x + 1, self.0.y)), // right
            Self(IVec2::new(self.0.x, self.0.y + 1)), // bottom
        ]
    }

    /// Calculate Manhattan distance to another position.
    #[must_use]
    pub const fn manhattan_distance(&self, other: &Self) -> u32
    {
        let dx = (self.0.x - other.0.x).unsigned_abs();
        let dy = (self.0.y - other.0.y).unsigned_abs();
        dx + dy
    }

    /// Calculate Euclidean distance squared to another position.
    #[must_use]
    pub const fn distance_squared(&self, other: &Self) -> u32
    {
        let dx = (self.0.x - other.0.x).unsigned_abs();
        let dy = (self.0.y - other.0.y).unsigned_abs();
        dx * dx + dy * dy
    }
}

/// Resource that maintains a spatial index for efficient neighbor queries.
#[derive(Resource, Default)]
pub struct SpatialGrid
{
    /// Maps grid positions to entities at those positions.
    position_to_entities: HashMap<GridPosition, Vec<Entity>>,
    /// Maps entities to their grid positions for fast lookups.
    entity_to_position: HashMap<Entity, GridPosition>,
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
            position_to_entities: HashMap::new(),
            entity_to_position: HashMap::new(),
            bounds: Some(bounds),
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
    pub fn insert(&mut self, entity: Entity, position: GridPosition)
    {
        // Remove entity from old position if it exists
        if let Some(old_pos) = self.entity_to_position.get(&entity)
            && let Some(entities) = self.position_to_entities.get_mut(old_pos)
        {
            entities.retain(|&e| e != entity);
            if entities.is_empty()
            {
                self.position_to_entities.remove(old_pos);
            }
        }

        // Insert at new position
        self.position_to_entities
            .entry(position)
            .or_default()
            .push(entity);
        self.entity_to_position.insert(entity, position);
    }

    /// Remove an entity from the spatial index.
    pub fn remove(&mut self, entity: Entity)
    {
        if let Some(position) = self.entity_to_position.remove(&entity)
            && let Some(entities) = self.position_to_entities.get_mut(&position)
        {
            entities.retain(|&e| e != entity);
            if entities.is_empty()
            {
                self.position_to_entities.remove(&position);
            }
        }
    }

    /// Get all entities at a specific position.
    pub fn entities_at(&self, position: &GridPosition) -> &[Entity]
    {
        self.position_to_entities
            .get(position)
            .map_or(&[], Vec::as_slice)
    }

    /// Get the position of an entity.
    #[must_use]
    pub fn position_of(&self, entity: Entity) -> Option<GridPosition>
    {
        self.entity_to_position.get(&entity).copied()
    }

    /// Get all entities in the 8-connected neighborhood of a position.
    #[must_use]
    pub fn neighbors_of(&self, position: &GridPosition) -> Vec<Entity>
    {
        let mut neighbors = Vec::new();
        for neighbor_pos in position.neighbors()
        {
            if let Some(bounds) = self.bounds
                && !bounds.contains(&neighbor_pos)
            {
                continue;
            }
            neighbors.extend_from_slice(self.entities_at(&neighbor_pos));
        }
        neighbors
    }

    /// Get all entities in the 4-connected orthogonal neighborhood of a position.
    #[must_use]
    pub fn orthogonal_neighbors_of(&self, position: &GridPosition) -> Vec<Entity>
    {
        let mut neighbors = Vec::new();
        for neighbor_pos in position.orthogonal_neighbors()
        {
            if let Some(bounds) = self.bounds
                && !bounds.contains(&neighbor_pos)
            {
                continue;
            }
            neighbors.extend_from_slice(self.entities_at(&neighbor_pos));
        }
        neighbors
    }

    /// Get all entities within a Manhattan distance of a position.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn entities_within_distance(&self, center: &GridPosition, distance: u32) -> Vec<Entity>
    {
        let mut entities = Vec::new();
        let distance_i32 = distance as i32;

        for x in (center.x - distance_i32)..=(center.x + distance_i32)
        {
            for y in (center.y - distance_i32)..=(center.y + distance_i32)
            {
                let pos = GridPosition::new(x, y);
                if pos.manhattan_distance(center) <= distance
                {
                    if let Some(bounds) = self.bounds
                        && !bounds.contains(&pos)
                    {
                        continue;
                    }
                    entities.extend_from_slice(self.entities_at(&pos));
                }
            }
        }
        entities
    }

    /// Clear all entities from the spatial index.
    pub fn clear(&mut self)
    {
        self.position_to_entities.clear();
        self.entity_to_position.clear();
    }

    /// Get all occupied positions in the grid.
    #[must_use]
    pub fn occupied_positions(&self) -> Vec<GridPosition>
    {
        self.position_to_entities.keys().copied().collect()
    }

    /// Get total number of entities in the grid.
    #[must_use]
    pub fn entity_count(&self) -> usize
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
type GridPositionQuery<'world, 'state> = Query<
    'world,
    'state,
    (Entity, &'static GridPosition),
    Or<(Added<GridPosition>, Changed<GridPosition>)>,
>;

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
