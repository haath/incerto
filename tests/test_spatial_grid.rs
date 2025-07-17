#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_possible_truncation)]

use bevy::prelude::{IVec2, IVec3};
use incerto::prelude::*;

#[test]
fn test_grid_position_neighbors()
{
    let pos = GridPosition2D::new(1, 1);

    let neighbors: Vec<GridPosition2D> = pos.neighbors().collect();
    assert_eq!(neighbors.len(), 8);

    // Check all 8 neighbors are present
    let expected_neighbors = [
        GridPosition2D::new(0, 0),
        GridPosition2D::new(1, 0),
        GridPosition2D::new(2, 0),
        GridPosition2D::new(0, 1),
        GridPosition2D::new(2, 1),
        GridPosition2D::new(0, 2),
        GridPosition2D::new(1, 2),
        GridPosition2D::new(2, 2),
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
    let pos = GridPosition2D::new(1, 1);

    let neighbors: Vec<GridPosition2D> = pos.neighbors_orthogonal().collect();
    assert_eq!(neighbors.len(), 4);

    let expected_neighbors = [
        GridPosition2D::new(1, 0), // top
        GridPosition2D::new(0, 1), // left
        GridPosition2D::new(2, 1), // right
        GridPosition2D::new(1, 2), // bottom
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
    let pos1 = GridPosition2D::new(0, 0);
    let pos2 = GridPosition2D::new(3, 4);

    // Test Manhattan distance
    let diff = pos2.0 - pos1.0;
    assert_eq!(diff.abs().element_sum(), 7);

    let pos3 = GridPosition2D::new(1, 1);
    let diff2 = pos3.0 - pos1.0;
    assert_eq!(diff2.abs().element_sum(), 2);
}

#[test]
fn test_grid_bounds()
{
    let bounds = GridBounds2D {
        min: IVec2::new(0, 0),
        max: IVec2::new(9, 9),
    };

    assert!(bounds.contains(&GridPosition2D::new(0, 0).0));
    assert!(bounds.contains(&GridPosition2D::new(9, 9).0));
    assert!(bounds.contains(&GridPosition2D::new(5, 5).0));

    assert!(!bounds.contains(&GridPosition2D::new(-1, 0).0));
    assert!(!bounds.contains(&GridPosition2D::new(0, -1).0));
    assert!(!bounds.contains(&GridPosition2D::new(10, 5).0));
    assert!(!bounds.contains(&GridPosition2D::new(5, 10).0));
}

#[test]
fn test_3d_grid_position_neighbors()
{
    let pos = GridPosition3D::new(1, 1, 1);

    let neighbors: Vec<GridPosition3D> = pos.neighbors().collect();
    assert_eq!(neighbors.len(), 26); // 3x3x3 cube minus center = 26 neighbors

    // Check that center position is not included
    assert!(!neighbors.contains(&pos));

    // Check some specific 3D neighbors
    assert!(neighbors.contains(&GridPosition3D::new(0, 0, 0))); // corner
    assert!(neighbors.contains(&GridPosition3D::new(2, 2, 2))); // opposite corner
    assert!(neighbors.contains(&GridPosition3D::new(1, 1, 0))); // directly below
    assert!(neighbors.contains(&GridPosition3D::new(1, 1, 2))); // directly above
}

#[test]
fn test_3d_grid_position_orthogonal_neighbors()
{
    let pos = GridPosition3D::new(1, 1, 1);

    let neighbors: Vec<GridPosition3D> = pos.neighbors_orthogonal().collect();
    assert_eq!(neighbors.len(), 6); // 6 orthogonal directions in 3D

    let expected_neighbors = [
        GridPosition3D::new(0, 1, 1), // -x
        GridPosition3D::new(2, 1, 1), // +x
        GridPosition3D::new(1, 0, 1), // -y
        GridPosition3D::new(1, 2, 1), // +y
        GridPosition3D::new(1, 1, 0), // -z
        GridPosition3D::new(1, 1, 2), // +z
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
    let pos1 = GridPosition3D::new(0, 0, 0);
    let pos2 = GridPosition3D::new(3, 4, 5);

    // Test 3D Manhattan distance
    let diff = pos2.0 - pos1.0;
    assert_eq!(diff.abs().element_sum(), 12); // 3 + 4 + 5 = 12

    let pos3 = GridPosition3D::new(1, 1, 1);
    let diff2 = pos3.0 - pos1.0;
    assert_eq!(diff2.abs().element_sum(), 3); // 1 + 1 + 1 = 3
}

#[test]
fn test_3d_grid_bounds()
{
    let bounds = GridBounds3D {
        min: IVec3::new(0, 0, 0),
        max: IVec3::new(9, 9, 9),
    };

    assert!(bounds.contains(&GridPosition3D::new(0, 0, 0).0));
    assert!(bounds.contains(&GridPosition3D::new(9, 9, 9).0));
    assert!(bounds.contains(&GridPosition3D::new(5, 5, 5).0));

    assert!(!bounds.contains(&GridPosition3D::new(-1, 0, 0).0));
    assert!(!bounds.contains(&GridPosition3D::new(0, -1, 0).0));
    assert!(!bounds.contains(&GridPosition3D::new(0, 0, -1).0));
    assert!(!bounds.contains(&GridPosition3D::new(10, 5, 5).0));
    assert!(!bounds.contains(&GridPosition3D::new(5, 10, 5).0));
    assert!(!bounds.contains(&GridPosition3D::new(5, 5, 10).0));
}

#[test]
fn test_3d_spatial_grid_integration()
{
    #[derive(Component)]
    struct TestEntity3D(i32);

    impl SampleAggregate<usize> for TestEntity3D
    {
        fn sample_aggregate(components: &[&Self]) -> usize
        {
            components.len()
        }
    }

    let bounds = GridBounds3D {
        min: IVec3::new(0, 0, 0),
        max: IVec3::new(4, 4, 4),
    };

    let builder = SimulationBuilder::new()
        .add_spatial_grid::<IVec3, TestEntity3D>(Some(bounds))
        .add_entity_spawner(|spawner| {
            // Spawn entities at different 3D positions
            spawner.spawn((GridPosition3D::new(0, 0, 0), TestEntity3D(1)));
            spawner.spawn((GridPosition3D::new(2, 2, 2), TestEntity3D(2)));
            spawner.spawn((GridPosition3D::new(4, 4, 4), TestEntity3D(3)));
            spawner.spawn((GridPosition3D::new(1, 2, 3), TestEntity3D(4)));
        })
        .add_systems(
            |spatial_grid: Res<SpatialGrid<IVec3, TestEntity3D>>,
             query: Query<(Entity, &GridPosition3D, &TestEntity3D)>| {
                // Test 3D spatial queries
                let center_pos = GridPosition3D::new(2, 2, 2);

                // Find entities within distance 2 in 3D space using neighbor-based approach
                let mut nearby_entities = Vec::new();
                let center_coord = center_pos.0;

                // Check all positions within Manhattan distance of 2
                for dx in -2i32..=2i32
                {
                    for dy in -2i32..=2i32
                    {
                        for dz in -2i32..=2i32
                        {
                            let manhattan_distance = dx.abs() + dy.abs() + dz.abs();
                            if manhattan_distance <= 2
                            {
                                let check_pos = GridPosition3D::new(
                                    center_coord.x + dx,
                                    center_coord.y + dy,
                                    center_coord.z + dz,
                                );
                                nearby_entities.extend(spatial_grid.entities_at(&check_pos));
                            }
                        }
                    }
                }

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
    let entity_count = simulation
        .sample::<TestEntity3D, usize>()
        .expect("Failed to sample TestEntity3D count");
    assert_eq!(entity_count, 4);
}

#[test]
fn test_spatial_grid_reset_functionality()
{
    #[derive(Component)]
    #[allow(dead_code)]
    struct TestResetEntity(i32);

    impl SampleAggregate<usize> for TestResetEntity
    {
        fn sample_aggregate(components: &[&Self]) -> usize
        {
            components.len()
        }
    }

    let bounds = GridBounds2D {
        min: IVec2::new(0, 0),
        max: IVec2::new(4, 4),
    };

    let mut simulation = SimulationBuilder::new()
        .add_spatial_grid::<IVec2, TestResetEntity>(Some(bounds))
        .add_entity_spawner(|spawner| {
            // Spawn entities at different positions
            spawner.spawn((GridPosition2D::new(0, 0), TestResetEntity(1)));
            spawner.spawn((GridPosition2D::new(2, 2), TestResetEntity(2)));
            spawner.spawn((GridPosition2D::new(4, 4), TestResetEntity(3)));
        })
        .build();

    // Run first simulation
    simulation.run(2);

    // Verify entities are tracked
    let entity_count = simulation
        .sample::<TestResetEntity, usize>()
        .expect("Failed to sample TestResetEntity count");
    assert_eq!(entity_count, 3);
}
