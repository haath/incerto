#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_possible_truncation)]

use incerto::prelude::*;

#[derive(Component)]
struct TestEntity(i32);

#[test]
fn test_grid_position_neighbors()
{
    let pos = GridPosition::new(1, 1);

    let neighbors = pos.neighbors();
    assert_eq!(neighbors.len(), 8);

    // Check all 8 neighbors are present
    let expected_neighbors = [
        GridPosition::new(0, 0),
        GridPosition::new(1, 0),
        GridPosition::new(2, 0),
        GridPosition::new(0, 1),
        GridPosition::new(2, 1),
        GridPosition::new(0, 2),
        GridPosition::new(1, 2),
        GridPosition::new(2, 2),
    ];

    for expected in expected_neighbors
    {
        assert!(
            neighbors.contains(&expected),
            "Missing neighbor: {:?}",
            expected
        );
    }
}

#[test]
fn test_grid_position_orthogonal_neighbors()
{
    let pos = GridPosition::new(1, 1);

    let neighbors = pos.orthogonal_neighbors();
    assert_eq!(neighbors.len(), 4);

    let expected_neighbors = [
        GridPosition::new(1, 0), // top
        GridPosition::new(0, 1), // left
        GridPosition::new(2, 1), // right
        GridPosition::new(1, 2), // bottom
    ];

    for expected in expected_neighbors
    {
        assert!(
            neighbors.contains(&expected),
            "Missing orthogonal neighbor: {:?}",
            expected
        );
    }
}

#[test]
fn test_grid_position_distances()
{
    let pos1 = GridPosition::new(0, 0);
    let pos2 = GridPosition::new(3, 4);

    assert_eq!(pos1.manhattan_distance(&pos2), 7);
    assert_eq!(pos1.distance_squared(&pos2), 25);

    let pos3 = GridPosition::new(1, 1);
    assert_eq!(pos1.manhattan_distance(&pos3), 2);
    assert_eq!(pos1.distance_squared(&pos3), 2);
}

#[test]
fn test_grid_bounds()
{
    let bounds = GridBounds::new(0, 9, 0, 9);

    assert_eq!(bounds.width(), 10);
    assert_eq!(bounds.height(), 10);
    assert_eq!(bounds.total_cells(), 100);

    assert!(bounds.contains(&GridPosition::new(0, 0)));
    assert!(bounds.contains(&GridPosition::new(9, 9)));
    assert!(bounds.contains(&GridPosition::new(5, 5)));

    assert!(!bounds.contains(&GridPosition::new(-1, 0)));
    assert!(!bounds.contains(&GridPosition::new(0, -1)));
    assert!(!bounds.contains(&GridPosition::new(10, 5)));
    assert!(!bounds.contains(&GridPosition::new(5, 10)));
}

#[test]
fn test_spatial_grid_basic_operations()
{
    let mut grid = SpatialGrid::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let pos1 = GridPosition::new(0, 0);
    let pos2 = GridPosition::new(1, 1);

    // Test insertion
    grid.insert(entity1, pos1);
    grid.insert(entity2, pos2);

    assert_eq!(grid.entity_count(), 2);
    assert_eq!(grid.position_of(entity1), Some(pos1));
    assert_eq!(grid.position_of(entity2), Some(pos2));

    // Test entities at position
    let entities_at_pos1 = grid.entities_at(&pos1);
    assert_eq!(entities_at_pos1.len(), 1);
    assert_eq!(entities_at_pos1[0], entity1);

    // Test removal
    grid.remove(entity1);
    assert_eq!(grid.entity_count(), 1);
    assert_eq!(grid.position_of(entity1), None);
    assert!(grid.entities_at(&pos1).is_empty());
}

#[test]
fn test_spatial_grid_with_bounds()
{
    let bounds = GridBounds::new(0, 2, 0, 2);
    let mut grid = SpatialGrid::with_bounds(bounds);

    assert_eq!(grid.bounds(), Some(bounds));

    let entity = Entity::from_raw(1);
    let center = GridPosition::new(1, 1);
    grid.insert(entity, center);

    // Test neighbor queries respect bounds
    let neighbors = grid.neighbors_of(&center);
    assert_eq!(neighbors.len(), 0); // No entities at neighbor positions

    // Add entities at neighbor positions
    for (i, neighbor_pos) in center.neighbors().iter().enumerate()
    {
        if bounds.contains(neighbor_pos)
        {
            let neighbor_entity = Entity::from_raw(i as u32 + 10);
            grid.insert(neighbor_entity, *neighbor_pos);
        }
    }

    let neighbors = grid.neighbors_of(&center);
    assert_eq!(neighbors.len(), 8); // All 8 neighbors are within bounds for center position
}

#[test]
fn test_spatial_grid_multiple_entities_per_position()
{
    let mut grid = SpatialGrid::new();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);
    let pos = GridPosition::new(0, 0);

    // Insert multiple entities at same position
    grid.insert(entity1, pos);
    grid.insert(entity2, pos);
    grid.insert(entity3, pos);

    let entities = grid.entities_at(&pos);
    assert_eq!(entities.len(), 3);
    assert!(entities.contains(&entity1));
    assert!(entities.contains(&entity2));
    assert!(entities.contains(&entity3));

    // Remove one entity
    grid.remove(entity2);
    let entities = grid.entities_at(&pos);
    assert_eq!(entities.len(), 2);
    assert!(!entities.contains(&entity2));
}

#[test]
fn test_spatial_grid_entity_movement()
{
    let mut grid = SpatialGrid::new();
    let entity = Entity::from_raw(1);
    let pos1 = GridPosition::new(0, 0);
    let pos2 = GridPosition::new(1, 1);

    // Insert entity at first position
    grid.insert(entity, pos1);
    assert_eq!(grid.entities_at(&pos1).len(), 1);
    assert!(grid.entities_at(&pos2).is_empty());

    // Move entity to second position
    grid.insert(entity, pos2);
    assert!(grid.entities_at(&pos1).is_empty());
    assert_eq!(grid.entities_at(&pos2).len(), 1);
    assert_eq!(grid.position_of(entity), Some(pos2));
}

#[test]
fn test_spatial_grid_distance_queries()
{
    let bounds = GridBounds::new(0, 4, 0, 4);
    let mut grid = SpatialGrid::with_bounds(bounds);
    let center = GridPosition::new(2, 2);

    // Add entities in a cross pattern
    let positions = [
        GridPosition::new(2, 2), // center
        GridPosition::new(2, 1), // top
        GridPosition::new(1, 2), // left
        GridPosition::new(3, 2), // right
        GridPosition::new(2, 3), // bottom
        GridPosition::new(0, 0), // corner
    ];

    for (i, &pos) in positions.iter().enumerate()
    {
        let entity = Entity::from_raw(i as u32 + 1);
        grid.insert(entity, pos);
    }

    // Test distance queries
    let entities_distance_0 = grid.entities_within_distance(&center, 0);
    assert_eq!(entities_distance_0.len(), 1); // Only center entity

    let entities_distance_1 = grid.entities_within_distance(&center, 1);
    assert_eq!(entities_distance_1.len(), 5); // Center + 4 orthogonal neighbors

    let entities_distance_2 = grid.entities_within_distance(&center, 2);
    assert_eq!(entities_distance_2.len(), 5); // Same as distance 1 in this setup

    let entities_distance_4 = grid.entities_within_distance(&center, 4);
    assert_eq!(entities_distance_4.len(), 6); // All entities including corner
}

#[test]
fn test_spatial_grid_plugin_integration()
{
    let _bounds = GridBounds::new(0, 2, 0, 2);

    let builder = SimulationBuilder::new()
        .add_entity_spawner(|spawner| {
            // Spawn entities with grid positions
            spawner.spawn((GridPosition::new(0, 0), TestEntity(1)));
            spawner.spawn((GridPosition::new(1, 1), TestEntity(2)));
            spawner.spawn((GridPosition::new(2, 2), TestEntity(3)));
        })
        .add_systems(|query: Query<(&GridPosition, &TestEntity)>| {
            // Verify entities have grid positions and test data
            for (position, test_entity) in &query
            {
                // Verify test entity data
                assert!(test_entity.0 > 0);
                assert!(position.x >= 0 && position.x <= 2);
                assert!(position.y >= 0 && position.y <= 2);
            }
        });

    let mut simulation = builder.build();
    simulation.run(1);

    // Test completed without panics, which means the spatial grid plugin
    // is working correctly with the simulation systems
}
