//! # Multi-Grid Ecosystem Simulation
//!
//! This example demonstrates the power of multiple spatial grids by simulating
//! a complex ecosystem with three different entity types, each with their own
//! spatial grid for optimized interactions:
//!
//! * **Prey (Rabbits)** - Use `SpatialGrid<IVec2, Prey>` for flocking behavior
//! * **Predators (Wolves)** - Use `SpatialGrid<IVec2, Predator>` for pack hunting
//! * **Vegetation (Grass)** - Use `SpatialGrid<IVec2, Vegetation>` for growth patterns
//!
//! ## Key Features:
//! * **Multi-scale interactions** - Different interaction ranges for different behaviors
//! * **Cross-grid queries** - Predators hunt prey, prey avoid predators, all consume vegetation
//! * **Emergent patterns** - Herds, pack formation, resource patches emerge naturally
//! * **Performance optimization** - Each entity type has its own spatial index

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]

use std::collections::HashMap;

use bevy::prelude::{IVec2, ParamSet};
use incerto::{
    plugins::{GridBounds2D, GridPosition2D, SpatialGrid, SpatialGridEntity},
    prelude::*,
};
use rand::prelude::*;

// Simulation parameters
const SIMULATION_STEPS: usize = 200;
const WORLD_SIZE: i32 = 30;
const INITIAL_PREY: usize = 50;
const INITIAL_PREDATORS: usize = 8;
const INITIAL_VEGETATION: usize = 120;

/// Prey animals (rabbits) that flock together and graze
#[derive(Component, Debug)]
pub struct Prey
{
    pub energy: u32,
    pub age: u32,
    pub fear_level: f32, // Affects movement when predators are near
}

/// Predator animals (wolves) that hunt in packs
#[derive(Component, Debug)]
pub struct Predator
{
    pub energy: u32,
    pub age: u32,
    pub hunt_cooldown: u32, // Ticks until can hunt again
}

/// Vegetation (grass) that grows and gets consumed
#[derive(Component, Debug)]
pub struct Vegetation
{
    pub growth_level: u32,   // 0-100, higher = more nutritious
    pub regrowth_timer: u32, // Ticks until next growth
}

/// Bundle for spawning prey entities
#[derive(Bundle)]
pub struct PreyBundle
{
    pub position: GridPosition2D,
    pub prey: Prey,
}

/// Bundle for spawning predator entities
#[derive(Bundle)]
pub struct PredatorBundle
{
    pub position: GridPosition2D,
    pub predator: Predator,
}

/// Bundle for spawning vegetation entities
#[derive(Bundle)]
pub struct VegetationBundle
{
    pub position: GridPosition2D,
    pub vegetation: Vegetation,
}

// Sample implementations for statistics
impl Sample<usize> for Prey
{
    fn sample(components: &[&Self]) -> usize
    {
        components.len()
    }
}

impl Sample<usize> for Predator
{
    fn sample(components: &[&Self]) -> usize
    {
        components.len()
    }
}

impl Sample<usize> for Vegetation
{
    fn sample(components: &[&Self]) -> usize
    {
        components.len()
    }
}

/// Spawn initial ecosystem entities
fn spawn_ecosystem(spawner: &mut Spawner)
{
    let mut rng = rand::rng();

    // Spawn prey (rabbits) in small groups
    for _ in 0..INITIAL_PREY
    {
        let x = rng.random_range(0..WORLD_SIZE);
        let y = rng.random_range(0..WORLD_SIZE);

        spawner.spawn(PreyBundle {
            position: GridPosition2D::new_2d(x, y),
            prey: Prey {
                energy: rng.random_range(50..100),
                age: rng.random_range(0..50),
                fear_level: 0.0,
            },
        });
    }

    // Spawn predators (wolves) scattered
    for _ in 0..INITIAL_PREDATORS
    {
        let x = rng.random_range(0..WORLD_SIZE);
        let y = rng.random_range(0..WORLD_SIZE);

        spawner.spawn(PredatorBundle {
            position: GridPosition2D::new_2d(x, y),
            predator: Predator {
                energy: rng.random_range(80..120),
                age: rng.random_range(0..30),
                hunt_cooldown: 0,
            },
        });
    }

    // Spawn vegetation (grass) patches
    for _ in 0..INITIAL_VEGETATION
    {
        let x = rng.random_range(0..WORLD_SIZE);
        let y = rng.random_range(0..WORLD_SIZE);

        spawner.spawn(VegetationBundle {
            position: GridPosition2D::new_2d(x, y),
            vegetation: Vegetation {
                growth_level: rng.random_range(20..80),
                regrowth_timer: 0,
            },
        });
    }
}

/// Prey flocking and movement system - uses only prey spatial grid
fn prey_behavior(
    prey_grid: Query<&SpatialGrid<IVec2, Prey>, With<SpatialGridEntity>>,
    predator_grid: Query<&SpatialGrid<IVec2, Predator>, With<SpatialGridEntity>>,
    mut prey_query: Query<(&mut GridPosition2D, &mut Prey)>,
)
{
    let Ok(prey_spatial) = prey_grid.single()
    else
    {
        return;
    };
    let Ok(predator_spatial) = predator_grid.single()
    else
    {
        return;
    };

    let mut rng = rand::rng();

    for (mut position, mut prey) in &mut prey_query
    {
        let current_pos = *position;

        // Reset fear
        prey.fear_level *= 0.9; // Decay fear over time

        // Check for nearby predators (CROSS-GRID INTERACTION!)
        let nearby_predators: Vec<_> = predator_spatial.neighbors_of(&current_pos).collect();
        if !nearby_predators.is_empty()
        {
            prey.fear_level = (prey.fear_level + 50.0).min(100.0);
        }

        // Flocking behavior - stay near other prey
        let nearby_prey: Vec<_> = prey_spatial.neighbors_of(&current_pos).collect();

        let mut target_x = current_pos.x() as f32;
        let mut target_y = current_pos.y() as f32;

        // If afraid, move away from predators
        if prey.fear_level > 10.0 && !nearby_predators.is_empty()
        {
            target_x += rng.random_range(-2.0..2.0);
            target_y += rng.random_range(-2.0..2.0);
        }
        else if nearby_prey.len() < 2
        {
            // If isolated, move randomly to find group
            target_x += rng.random_range(-1.0..1.0);
            target_y += rng.random_range(-1.0..1.0);
        }
        else if nearby_prey.len() > 5
        {
            // If overcrowded, spread out slightly
            target_x += rng.random_range(-0.5..0.5);
            target_y += rng.random_range(-0.5..0.5);
        }

        // Clamp to world bounds
        let new_x = (target_x as i32).clamp(0, WORLD_SIZE - 1);
        let new_y = (target_y as i32).clamp(0, WORLD_SIZE - 1);

        // Update position directly to trigger Changed<GridPosition2D>
        if new_x != current_pos.x() || new_y != current_pos.y()
        {
            *position = GridPosition2D::new_2d(new_x, new_y);
        }

        // Consume energy
        prey.energy = prey.energy.saturating_sub(1);
        prey.age += 1;
    }
}

/// Predator hunting system - uses both predator and prey spatial grids
fn predator_hunting(
    predator_grid: Query<&SpatialGrid<IVec2, Predator>, With<SpatialGridEntity>>,
    prey_grid: Query<&SpatialGrid<IVec2, Prey>, With<SpatialGridEntity>>,
    mut query_set: ParamSet<(
        Query<(&mut GridPosition2D, &mut Predator)>,
        Query<(Entity, &GridPosition2D), With<Prey>>,
    )>,
    mut commands: Commands,
)
{
    let Ok(predator_spatial) = predator_grid.single()
    else
    {
        return;
    };
    let Ok(prey_spatial) = prey_grid.single()
    else
    {
        return;
    };

    let mut rng = rand::rng();
    let mut hunts = Vec::new();

    for (mut position, mut predator) in &mut query_set.p0()
    {
        let current_pos = *position;

        // Reduce hunt cooldown
        predator.hunt_cooldown = predator.hunt_cooldown.saturating_sub(1);

        // Look for prey in neighboring cells (CROSS-GRID INTERACTION!)
        let nearby_prey: Vec<_> = prey_spatial.neighbors_of(&current_pos).collect();

        let mut target_x = current_pos.x() as f32;
        let mut target_y = current_pos.y() as f32;

        if !nearby_prey.is_empty() && predator.hunt_cooldown == 0 && predator.energy > 30
        {
            // Hunt nearby prey
            if let Some(prey_entity) = nearby_prey.choose(&mut rng)
            {
                hunts.push(*prey_entity);
                predator.energy += 40; // Gain energy from successful hunt
                predator.hunt_cooldown = 10; // Cooldown before next hunt
            }
        }
        else
        {
            // Move toward areas with more prey
            let mut best_prey_count = 0;
            let mut best_direction = (0.0, 0.0);

            // Check surrounding areas for prey density
            for dx in -2..=2
            {
                for dy in -2..=2
                {
                    if dx == 0 && dy == 0
                    {
                        continue;
                    }

                    let check_x = (current_pos.x() + dx).clamp(0, WORLD_SIZE - 1);
                    let check_y = (current_pos.y() + dy).clamp(0, WORLD_SIZE - 1);
                    let check_pos = GridPosition2D::new_2d(check_x, check_y);

                    let prey_count = prey_spatial.entities_at(&check_pos).count();
                    if prey_count > best_prey_count
                    {
                        best_prey_count = prey_count;
                        best_direction = (dx as f32, dy as f32);
                    }
                }
            }

            if best_prey_count > 0
            {
                target_x += best_direction.0 * 0.5;
                target_y += best_direction.1 * 0.5;
            }
            else
            {
                // Random movement when no prey detected
                target_x += rng.random_range(-1.0..1.0);
                target_y += rng.random_range(-1.0..1.0);
            }
        }

        // Avoid other predators (pack behavior - maintain some distance)
        let nearby_predators: Vec<_> = predator_spatial.neighbors_of(&current_pos).collect();
        if nearby_predators.len() > 2
        {
            target_x += rng.random_range(-0.5..0.5);
            target_y += rng.random_range(-0.5..0.5);
        }

        // Clamp to world bounds
        let new_x = (target_x as i32).clamp(0, WORLD_SIZE - 1);
        let new_y = (target_y as i32).clamp(0, WORLD_SIZE - 1);

        // Update position directly to trigger Changed<GridPosition2D>
        if new_x != current_pos.x() || new_y != current_pos.y()
        {
            *position = GridPosition2D::new_2d(new_x, new_y);
        }

        // Consume energy (hunting is expensive)
        predator.energy = predator.energy.saturating_sub(2);
        predator.age += 1;
    }

    // Execute hunts (remove caught prey)
    for prey_entity in hunts
    {
        commands.entity(prey_entity).despawn();
    }
}

/// Vegetation growth and grazing system - uses vegetation spatial grid
fn vegetation_dynamics(
    vegetation_grid: Query<&SpatialGrid<IVec2, Vegetation>, With<SpatialGridEntity>>,
    prey_grid: Query<&SpatialGrid<IVec2, Prey>, With<SpatialGridEntity>>,
    mut vegetation_query: Query<(Entity, &GridPosition2D, &mut Vegetation)>,
    mut commands: Commands,
)
{
    let Ok(vegetation_spatial) = vegetation_grid.single()
    else
    {
        return;
    };
    let Ok(prey_spatial) = prey_grid.single()
    else
    {
        return;
    };

    let mut rng = rand::rng();
    let mut consumed_vegetation = Vec::new();

    // Vegetation growth and consumption
    for (veg_entity, position, mut vegetation) in &mut vegetation_query
    {
        // Check for grazing prey (CROSS-GRID INTERACTION!)
        let grazing_prey: Vec<_> = prey_spatial.entities_at(position).collect();

        if !grazing_prey.is_empty() && vegetation.growth_level > 10
        {
            // Vegetation gets consumed
            let consumption = rng.random_range(10..30).min(vegetation.growth_level);
            vegetation.growth_level = vegetation.growth_level.saturating_sub(consumption);

            // Note: Prey feeding happens in a separate system to avoid query conflicts

            if vegetation.growth_level == 0
            {
                consumed_vegetation.push(veg_entity);
            }
        }
        else
        {
            // Natural growth
            vegetation.regrowth_timer = vegetation.regrowth_timer.saturating_sub(1);
            if vegetation.regrowth_timer == 0
            {
                vegetation.growth_level = (vegetation.growth_level + 1).min(100);
                vegetation.regrowth_timer = rng.random_range(5..15);
            }

            // Spread to nearby empty areas occasionally
            if vegetation.growth_level > 50 && rng.random_bool(0.05)
            {
                let spread_x = position.x() + rng.random_range(-1..=1);
                let spread_y = position.y() + rng.random_range(-1..=1);

                if spread_x >= 0 && spread_x < WORLD_SIZE && spread_y >= 0 && spread_y < WORLD_SIZE
                {
                    let spread_pos = GridPosition2D::new_2d(spread_x, spread_y);

                    // Check if area is empty of vegetation
                    let existing_veg = vegetation_spatial.entities_at(&spread_pos).count();
                    if existing_veg == 0
                    {
                        commands.spawn(VegetationBundle {
                            position: spread_pos,
                            vegetation: Vegetation {
                                growth_level: 30,
                                regrowth_timer: rng.random_range(10..20),
                            },
                        });
                    }
                }
            }
        }
    }

    // Remove fully consumed vegetation
    for entity in consumed_vegetation
    {
        commands.entity(entity).despawn();
    }
}

/// Prey feeding system - separate from vegetation to avoid query conflicts
fn prey_feeding(
    vegetation_grid: Query<&SpatialGrid<IVec2, Vegetation>, With<SpatialGridEntity>>,
    mut prey_query: Query<(&GridPosition2D, &mut Prey)>,
    vegetation_query: Query<&Vegetation>,
)
{
    let Ok(vegetation_spatial) = vegetation_grid.single()
    else
    {
        return;
    };

    for (prey_pos, mut prey) in &mut prey_query
    {
        // Check for vegetation at prey location and feed from the first available
        for veg_entity in vegetation_spatial.entities_at(prey_pos)
        {
            if let Ok(vegetation) = vegetation_query.get(veg_entity)
            {
                if vegetation.growth_level > 10
                {
                    let nutrition = (vegetation.growth_level / 4).min(20);
                    prey.energy = (prey.energy + nutrition).min(100);
                    break;
                }
            }
        }
    }
}

/// Natural lifecycle - entities die of old age or starvation
fn lifecycle_system(
    prey_query: Query<(Entity, &Prey)>,
    predator_query: Query<(Entity, &Predator)>,
    mut commands: Commands,
)
{
    let mut rng = rand::rng();

    // Prey lifecycle
    for (entity, prey) in &prey_query
    {
        if prey.energy == 0 || (prey.age > 100 && rng.random_bool(0.1))
        {
            commands.entity(entity).despawn();
        }
    }

    // Predator lifecycle
    for (entity, predator) in &predator_query
    {
        if predator.energy == 0 || (predator.age > 150 && rng.random_bool(0.05))
        {
            commands.entity(entity).despawn();
        }
    }
}

/// Display ecosystem statistics and spatial patterns
fn display_ecosystem_stats(
    prey_query: Query<(&GridPosition2D, &Prey)>,
    predator_query: Query<(&GridPosition2D, &Predator)>,
    vegetation_query: Query<(&GridPosition2D, &Vegetation)>,
)
{
    let prey_count = prey_query.iter().count();
    let predator_count = predator_query.iter().count();
    let vegetation_count = vegetation_query.iter().count();

    let avg_prey_energy: f32 = if prey_count > 0
    {
        prey_query.iter().map(|(_, p)| p.energy as f32).sum::<f32>() / prey_count as f32
    }
    else
    {
        0.0
    };

    let avg_predator_energy: f32 = if predator_count > 0
    {
        predator_query
            .iter()
            .map(|(_, p)| p.energy as f32)
            .sum::<f32>()
            / predator_count as f32
    }
    else
    {
        0.0
    };

    let avg_vegetation_growth: f32 = if vegetation_count > 0
    {
        vegetation_query
            .iter()
            .map(|(_, v)| v.growth_level as f32)
            .sum::<f32>()
            / vegetation_count as f32
    }
    else
    {
        0.0
    };

    // Analyze spatial distribution
    let mut prey_density = HashMap::new();
    for (pos, _) in &prey_query
    {
        let region = (pos.x() / 5, pos.y() / 5); // 5x5 regions
        *prey_density.entry(region).or_insert(0) += 1;
    }

    let max_prey_density = prey_density.values().max().copied().unwrap_or(0);

    println!("\n🌍 Ecosystem Status:");
    println!(
        "   🐰 Prey: {} (avg energy: {:.1})",
        prey_count, avg_prey_energy
    );
    println!(
        "   🐺 Predators: {} (avg energy: {:.1})",
        predator_count, avg_predator_energy
    );
    println!(
        "   🌱 Vegetation: {} (avg growth: {:.1}%)",
        vegetation_count, avg_vegetation_growth
    );
    println!("   📊 Max prey density in region: {}", max_prey_density);

    if prey_count == 0
    {
        println!("   ⚠️  All prey extinct!");
    }
    if predator_count == 0
    {
        println!("   ⚠️  All predators extinct!");
    }
}

fn main()
{
    println!("🌍 Multi-Grid Ecosystem Simulation");
    println!("Demonstrating multiple spatial grids working together:");
    println!("  • Prey Grid: Flocking and grazing behavior");
    println!("  • Predator Grid: Pack hunting coordination");
    println!("  • Vegetation Grid: Growth and resource management");
    println!("World size: {}x{}", WORLD_SIZE, WORLD_SIZE);
    println!("Duration: {} steps\n", SIMULATION_STEPS);

    let bounds = GridBounds2D::new_2d(0, WORLD_SIZE - 1, 0, WORLD_SIZE - 1);

    let mut simulation = SimulationBuilder::new()
        // Create separate spatial grids for each entity type
        .add_spatial_grid::<IVec2, Prey>(bounds)
        .add_spatial_grid::<IVec2, Predator>(bounds)
        .add_spatial_grid::<IVec2, Vegetation>(bounds)
        .add_entity_spawner(spawn_ecosystem)
        .add_systems((
            prey_behavior,
            predator_hunting.after(prey_behavior),
            vegetation_dynamics.after(predator_hunting),
            prey_feeding.after(vegetation_dynamics),
            lifecycle_system.after(prey_feeding),
            display_ecosystem_stats.after(lifecycle_system),
        ))
        .build();

    // Run ecosystem simulation
    for step in 1..=SIMULATION_STEPS
    {
        println!("🕐 Step {}/{}", step, SIMULATION_STEPS);
        simulation.run(1);

        if step % 25 == 0
        {
            let prey_count = simulation.sample::<Prey, usize>().unwrap();
            let predator_count = simulation.sample::<Predator, usize>().unwrap();
            let vegetation_count = simulation.sample::<Vegetation, usize>().unwrap();

            println!(
                "   📈 Population trends: 🐰{} 🐺{} 🌱{}",
                prey_count, predator_count, vegetation_count
            );

            if prey_count == 0 && predator_count == 0
            {
                println!("\n💀 Ecosystem collapse! All animals extinct.");
                break;
            }
        }

        // Add delay for readability
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    println!("\n✅ Ecosystem simulation completed!");

    let final_prey = simulation.sample::<Prey, usize>().unwrap();
    let final_predators = simulation.sample::<Predator, usize>().unwrap();
    let final_vegetation = simulation.sample::<Vegetation, usize>().unwrap();

    println!("📊 Final populations:");
    println!("   🐰 Prey: {}", final_prey);
    println!("   🐺 Predators: {}", final_predators);
    println!("   🌱 Vegetation: {}", final_vegetation);
}
