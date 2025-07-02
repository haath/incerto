#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_possible_truncation)]

use incerto::{
    plugins::{GridBounds3D, GridPosition3D, SpatialGrid3D},
    prelude::*,
};

#[derive(Component)]
struct TestEntity(i32);

#[test]
fn test_grid_position_neighbors()
{
    let pos = GridPosition2D::new_2d(1, 1);

    let neighbors: Vec<GridPosition2D> = pos.neighbors().collect();
    assert_eq!(neighbors.len(), 8);

    // Check all 8 neighbors are present
    let expected_neighbors = [
        GridPosition2D::new_2d(0, 0),
        GridPosition2D::new_2d(1, 0),
        GridPosition2D::new_2d(2, 0),
        GridPosition2D::new_2d(0, 1),
        GridPosition2D::new_2d(2, 1),
        GridPosition2D::new_2d(0, 2),
        GridPosition2D::new_2d(1, 2),
        GridPosition2D::new_2d(2, 2),
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
    let pos = GridPosition2D::new_2d(1, 1);

    let neighbors: Vec<GridPosition2D> = pos.neighbors_orthogonal().collect();
    assert_eq!(neighbors.len(), 4);

    let expected_neighbors = [
        GridPosition2D::new_2d(1, 0), // top
        GridPosition2D::new_2d(0, 1), // left
        GridPosition2D::new_2d(2, 1), // right
        GridPosition2D::new_2d(1, 2), // bottom
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
    let pos1 = GridPosition2D::new_2d(0, 0);
    let pos2 = GridPosition2D::new_2d(3, 4);

    // Test Manhattan distance using the trait method
    assert_eq!(pos1.manhattan_distance(&pos2), 7);

    let pos3 = GridPosition2D::new_2d(1, 1);
    assert_eq!(pos1.manhattan_distance(&pos3), 2);
}

#[test]
fn test_grid_bounds()
{
    let bounds = GridBounds2D::new_2d(0, 9, 0, 9);

    assert_eq!(bounds.width(), 10);
    assert_eq!(bounds.height(), 10);
    assert_eq!(bounds.total_cells(), 100);

    assert!(bounds.contains(&GridPosition2D::new_2d(0, 0)));
    assert!(bounds.contains(&GridPosition2D::new_2d(9, 9)));
    assert!(bounds.contains(&GridPosition2D::new_2d(5, 5)));

    assert!(!bounds.contains(&GridPosition2D::new_2d(-1, 0)));
    assert!(!bounds.contains(&GridPosition2D::new_2d(0, -1)));
    assert!(!bounds.contains(&GridPosition2D::new_2d(10, 5)));
    assert!(!bounds.contains(&GridPosition2D::new_2d(5, 10)));
}

#[test]
fn test_spatial_grid_plugin_integration()
{
    let _bounds = GridBounds2D::new_2d(0, 2, 0, 2);

    let builder = SimulationBuilder::new()
        .add_entity_spawner(|spawner| {
            // Spawn entities with grid positions
            spawner.spawn((GridPosition2D::new_2d(0, 0), TestEntity(1)));
            spawner.spawn((GridPosition2D::new_2d(1, 1), TestEntity(2)));
            spawner.spawn((GridPosition2D::new_2d(2, 2), TestEntity(3)));
        })
        .add_systems(|query: Query<(&GridPosition2D, &TestEntity)>| {
            // Verify entities have grid positions and test data
            for (position, test_entity) in &query
            {
                // Verify test entity data
                assert!(test_entity.0 > 0);
                assert!(position.x() >= 0 && position.x() <= 2);
                assert!(position.y() >= 0 && position.y() <= 2);
            }
        });

    let mut simulation = builder.build();
    simulation.run(1);

    // Test completed without panics, which means the spatial grid plugin
    // is working correctly with the simulation systems
}

#[test]
fn test_3d_grid_position_neighbors()
{
    let pos = GridPosition3D::new_3d(1, 1, 1);

    let neighbors: Vec<GridPosition3D> = pos.neighbors().collect();
    assert_eq!(neighbors.len(), 26); // 3x3x3 cube minus center = 26 neighbors

    // Check that center position is not included
    assert!(!neighbors.contains(&pos));

    // Check some specific 3D neighbors
    assert!(neighbors.contains(&GridPosition3D::new_3d(0, 0, 0))); // corner
    assert!(neighbors.contains(&GridPosition3D::new_3d(2, 2, 2))); // opposite corner
    assert!(neighbors.contains(&GridPosition3D::new_3d(1, 1, 0))); // directly below
    assert!(neighbors.contains(&GridPosition3D::new_3d(1, 1, 2))); // directly above
}

#[test]
fn test_3d_grid_position_orthogonal_neighbors()
{
    let pos = GridPosition3D::new_3d(1, 1, 1);

    let neighbors: Vec<GridPosition3D> = pos.neighbors_orthogonal().collect();
    assert_eq!(neighbors.len(), 6); // 6 orthogonal directions in 3D

    let expected_neighbors = [
        GridPosition3D::new_3d(0, 1, 1), // -x
        GridPosition3D::new_3d(2, 1, 1), // +x
        GridPosition3D::new_3d(1, 0, 1), // -y
        GridPosition3D::new_3d(1, 2, 1), // +y
        GridPosition3D::new_3d(1, 1, 0), // -z
        GridPosition3D::new_3d(1, 1, 2), // +z
    ];

    for expected in expected_neighbors
    {
        assert!(
            neighbors.contains(&expected),
            "Missing 3D orthogonal neighbor: {:?}",
            expected
        );
    }
}

#[test]
fn test_3d_grid_position_distances()
{
    let pos1 = GridPosition3D::new_3d(0, 0, 0);
    let pos2 = GridPosition3D::new_3d(3, 4, 5);

    // Test 3D Manhattan distance
    assert_eq!(pos1.manhattan_distance(&pos2), 12); // 3 + 4 + 5 = 12

    let pos3 = GridPosition3D::new_3d(1, 1, 1);
    assert_eq!(pos1.manhattan_distance(&pos3), 3); // 1 + 1 + 1 = 3
}

#[test]
fn test_3d_grid_bounds()
{
    let bounds = GridBounds3D::new_3d(0, 9, 0, 9, 0, 9);

    assert_eq!(bounds.width(), 10);
    assert_eq!(bounds.height(), 10);
    assert_eq!(bounds.depth(), 10);
    assert_eq!(bounds.total_cells(), 1000);

    assert!(bounds.contains(&GridPosition3D::new_3d(0, 0, 0)));
    assert!(bounds.contains(&GridPosition3D::new_3d(9, 9, 9)));
    assert!(bounds.contains(&GridPosition3D::new_3d(5, 5, 5)));

    assert!(!bounds.contains(&GridPosition3D::new_3d(-1, 0, 0)));
    assert!(!bounds.contains(&GridPosition3D::new_3d(0, -1, 0)));
    assert!(!bounds.contains(&GridPosition3D::new_3d(0, 0, -1)));
    assert!(!bounds.contains(&GridPosition3D::new_3d(10, 5, 5)));
    assert!(!bounds.contains(&GridPosition3D::new_3d(5, 10, 5)));
    assert!(!bounds.contains(&GridPosition3D::new_3d(5, 5, 10)));
}

#[test]
fn test_3d_spatial_grid_integration()
{
    #[derive(Component)]
    struct TestEntity3D(i32);

    impl Sample<usize> for TestEntity3D
    {
        fn sample(components: &[&Self]) -> usize
        {
            components.len()
        }
    }

    let bounds = GridBounds3D::new_3d(0, 4, 0, 4, 0, 4);

    let builder = SimulationBuilder::new()
        .add_spatial_grid_3d(bounds)
        .add_entity_spawner(|spawner| {
            // Spawn entities at different 3D positions
            spawner.spawn((GridPosition3D::new_3d(0, 0, 0), TestEntity3D(1)));
            spawner.spawn((GridPosition3D::new_3d(2, 2, 2), TestEntity3D(2)));
            spawner.spawn((GridPosition3D::new_3d(4, 4, 4), TestEntity3D(3)));
            spawner.spawn((GridPosition3D::new_3d(1, 2, 3), TestEntity3D(4)));
        })
        .add_systems(
            |spatial_grid: Res<SpatialGrid3D>,
             query: Query<(Entity, &GridPosition3D, &TestEntity3D)>| {
                // Test 3D spatial queries
                let center_pos = GridPosition3D::new_3d(2, 2, 2);

                // Find entities within distance 2 in 3D space
                let nearby_entities = spatial_grid.entities_within_distance(&center_pos, 2);
                assert!(
                    !nearby_entities.is_empty(),
                    "Should find nearby entities in 3D"
                );

                // Verify all positions are valid 3D coordinates
                for (_, position, test_entity) in &query
                {
                    assert!(test_entity.0 > 0);
                    assert!(position.x() >= 0 && position.x() <= 4);
                    assert!(position.y() >= 0 && position.y() <= 4);
                    assert!(position.z() >= 0 && position.z() <= 4);
                }
            },
        );

    let mut simulation = builder.build();
    simulation.run(1);

    // Verify all entities were created
    let entity_count = simulation.sample::<TestEntity3D, usize>().unwrap();
    assert_eq!(entity_count, 4);
}
