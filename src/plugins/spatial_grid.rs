use std::collections::HashMap;

use bevy::prelude::*;

/// Component representing a position in the spatial grid.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition
{
    pub x: i32,
    pub y: i32,
}

impl GridPosition
{
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self
    {
        Self { x, y }
    }

    /// Get the 8 neighboring positions (Moore neighborhood).
    #[must_use]
    pub const fn neighbors(&self) -> [Self; 8]
    {
        [
            Self::new(self.x - 1, self.y - 1), // top-left
            Self::new(self.x, self.y - 1),     // top
            Self::new(self.x + 1, self.y - 1), // top-right
            Self::new(self.x - 1, self.y),     // left
            Self::new(self.x + 1, self.y),     // right
            Self::new(self.x - 1, self.y + 1), // bottom-left
            Self::new(self.x, self.y + 1),     // bottom
            Self::new(self.x + 1, self.y + 1), // bottom-right
        ]
    }

    /// Get the 4 orthogonal neighboring positions (Von Neumann neighborhood).
    #[must_use]
    pub const fn orthogonal_neighbors(&self) -> [Self; 4]
    {
        [
            Self::new(self.x, self.y - 1), // top
            Self::new(self.x - 1, self.y), // left
            Self::new(self.x + 1, self.y), // right
            Self::new(self.x, self.y + 1), // bottom
        ]
    }

    /// Calculate Manhattan distance to another position.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn manhattan_distance(&self, other: &Self) -> u32
    {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }

    /// Calculate Euclidean distance squared to another position.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn distance_squared(&self, other: &Self) -> u32
    {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy) as u32
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridBounds
{
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl GridBounds
{
    #[must_use]
    pub const fn new(min_x: i32, max_x: i32, min_y: i32, max_y: i32) -> Self
    {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    #[must_use]
    pub const fn contains(&self, pos: &GridPosition) -> bool
    {
        pos.x >= self.min_x && pos.x <= self.max_x && pos.y >= self.min_y && pos.y <= self.max_y
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn width(&self) -> u32
    {
        (self.max_x - self.min_x + 1) as u32
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn height(&self) -> u32
    {
        (self.max_y - self.min_y + 1) as u32
    }

    #[must_use]
    pub const fn total_cells(&self) -> u32
    {
        self.width() * self.height()
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

/// System that updates the spatial grid when entities with `GridPosition` are added or moved.
#[allow(clippy::type_complexity)]
pub fn spatial_grid_update_system(
    mut spatial_grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &GridPosition), Or<(Added<GridPosition>, Changed<GridPosition>)>>,
)
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
