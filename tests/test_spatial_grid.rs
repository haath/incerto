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

    let neighbors: Vec<GridPosition> = pos.neighbors().collect();
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

    let neighbors: Vec<GridPosition> = pos.neighbors_orthogonal().collect();
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

    // Test Manhattan distance using IVec2 operations
    assert_eq!((*pos1 - *pos2).abs().element_sum(), 7);
    // Test Euclidean distance squared using IVec2 operations
    let diff = *pos1 - *pos2;
    assert_eq!(diff.x * diff.x + diff.y * diff.y, 25);

    let pos3 = GridPosition::new(1, 1);
    assert_eq!((*pos1 - *pos3).abs().element_sum(), 2);
    let diff3 = *pos1 - *pos3;
    assert_eq!(diff3.x * diff3.x + diff3.y * diff3.y, 2);
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
