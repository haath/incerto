//! # Simple 3D Air Traffic Control Simulation
//!
//! This example demonstrates 3D spatial grid functionality with aircraft moving
//! through different altitude levels in a simple airspace.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::expect_used)]

use bevy::prelude::*;
use incerto::prelude::*;
use rand::prelude::*;

// Simulation parameters
const SIMULATION_STEPS: usize = 50;
const AIRSPACE_SIZE: i32 = 20; // 20x20x10 airspace
const AIRSPACE_HEIGHT: i32 = 10;
const NUM_AIRCRAFT: usize = 15;

/// Aircraft component with basic properties
#[derive(Component, Debug)]
pub struct Aircraft
{
    pub id: usize,
    pub aircraft_type: AircraftType,
    pub target_altitude: i32,
    pub speed: i32, // cells per step
}

#[derive(Debug, Clone, Copy)]
pub enum AircraftType
{
    Commercial,
    PrivateJet,
    Cargo,
}

impl AircraftType
{
    const fn symbol(self) -> &'static str
    {
        match self
        {
            Self::Commercial => "✈️",
            Self::PrivateJet => "🛩️",
            Self::Cargo => "🛫",
        }
    }
}

/// Sample trait for counting aircraft
impl Sample<usize> for Aircraft
{
    fn sample(components: &[&Self]) -> usize
    {
        components.len()
    }
}

/// Spawn aircraft at random positions and altitudes
fn spawn_aircraft(spawner: &mut Spawner)
{
    let mut rng = rand::rng();

    for i in 0..NUM_AIRCRAFT
    {
        let x = rng.random_range(0..AIRSPACE_SIZE);
        let y = rng.random_range(0..AIRSPACE_SIZE);
        let z = rng.random_range(0..AIRSPACE_HEIGHT);

        let aircraft_type = match rng.random_range(0..3)
        {
            0 => AircraftType::Commercial,
            1 => AircraftType::PrivateJet,
            _ => AircraftType::Cargo,
        };

        let target_altitude = rng.random_range(0..AIRSPACE_HEIGHT);

        let aircraft = Aircraft {
            id: i,
            aircraft_type,
            target_altitude,
            speed: 1,
        };

        let position = GridPosition3D::new(x, y, z);
        spawner.spawn((aircraft, position));
    }
}

/// Move aircraft towards their target altitudes and in random horizontal directions
fn move_aircraft(mut query: Query<(&mut GridPosition3D, &Aircraft)>)
{
    let mut rng = rand::rng();

    for (mut position, aircraft) in &mut query
    {
        // Move towards target altitude
        if position.z() < aircraft.target_altitude
        {
            *position = GridPosition3D::new(position.x(), position.y(), position.z() + 1);
        }
        else if position.z() > aircraft.target_altitude
        {
            *position = GridPosition3D::new(position.x(), position.y(), position.z() - 1);
        }

        // Random horizontal movement
        if rng.random_bool(0.7)
        {
            let dx = rng.random_range(-1..=1);
            let dy = rng.random_range(-1..=1);

            let new_x = (position.x() + dx).clamp(0, AIRSPACE_SIZE - 1);
            let new_y = (position.y() + dy).clamp(0, AIRSPACE_SIZE - 1);

            *position = GridPosition3D::new(new_x, new_y, position.z());
        }
    }
}

/// Check for aircraft conflicts (too close in 3D space)
fn check_conflicts(
    spatial_grid: Res<SpatialGrid3D<Aircraft>>,
    query: Query<(Entity, &GridPosition3D, &Aircraft)>,
)
{
    let mut conflicts = 0;

    for (entity, position, aircraft) in &query
    {
        // Check for nearby aircraft (within 1 cell in any direction)
        let nearby_aircraft = spatial_grid.neighbors_of(position);

        for nearby_entity in nearby_aircraft
        {
            if nearby_entity != entity
                && let Ok((_, nearby_pos, nearby_aircraft)) = query.get(nearby_entity)
            {
                let distance = (position.0 - nearby_pos.0).length_squared();
                if distance <= 1
                {
                    conflicts += 1;
                    println!(
                        "⚠️  CONFLICT: Aircraft {} {} and {} {} too close at distance {}",
                        aircraft.id,
                        aircraft.aircraft_type.symbol(),
                        nearby_aircraft.id,
                        nearby_aircraft.aircraft_type.symbol(),
                        distance
                    );
                }
            }
        }
    }

    if conflicts > 0
    {
        println!("   Total conflicts detected: {}", conflicts / 2); // Divide by 2 since each conflict is counted twice
    }
}

/// Display airspace status
fn display_airspace(query: Query<(&GridPosition3D, &Aircraft)>)
{
    println!("\n📡 Airspace Status:");

    // Count aircraft by altitude
    let mut altitude_counts = vec![0; AIRSPACE_HEIGHT as usize];
    let mut aircraft_positions = Vec::new();

    for (position, aircraft) in &query
    {
        altitude_counts[position.z() as usize] += 1;
        aircraft_positions.push((position, aircraft));
    }

    for altitude in 0..AIRSPACE_HEIGHT
    {
        let count = altitude_counts[altitude as usize];
        if count > 0
        {
            print!("   FL{altitude:02}: {count} aircraft ");

            // Show aircraft at this altitude
            for (pos, aircraft) in &aircraft_positions
            {
                if pos.z() == altitude
                {
                    print!("{} ", aircraft.aircraft_type.symbol());
                }
            }
            println!();
        }
    }

    println!("   Airspace: {AIRSPACE_SIZE}x{AIRSPACE_SIZE}x{AIRSPACE_HEIGHT} cells");
}

fn main()
{
    println!("✈️ 3D Air Traffic Control Simulation");
    println!("Airspace: {AIRSPACE_SIZE}x{AIRSPACE_SIZE}x{AIRSPACE_HEIGHT} cells");
    println!("Aircraft: {NUM_AIRCRAFT}");
    println!("Duration: {SIMULATION_STEPS} steps\n");

    // Create 3D airspace bounds
    let bounds = GridBounds3D {
        min: IVec3::ZERO,
        max: IVec3::new(AIRSPACE_SIZE, AIRSPACE_SIZE, AIRSPACE_HEIGHT),
    };

    let mut simulation = SimulationBuilder::new()
        .add_spatial_grid::<IVec3, Aircraft>(Some(bounds))
        .add_entity_spawner(spawn_aircraft)
        .add_systems((move_aircraft, check_conflicts, display_airspace))
        .build();

    // Run simulation
    for step in 1..=SIMULATION_STEPS
    {
        println!("🕐 Step {step}/{SIMULATION_STEPS}");
        simulation.run(1);

        if step.is_multiple_of(10)
        {
            let aircraft_count = simulation
                .sample::<Aircraft, usize>()
                .expect("Failed to sample aircraft count");
            println!("   📊 Total aircraft tracked: {aircraft_count}");
        }

        // Add small delay for readability
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    println!("\n✅ Air Traffic Control simulation completed!");

    let final_count = simulation
        .sample::<Aircraft, usize>()
        .expect("Failed to sample final aircraft count");
    println!("📈 Final aircraft count: {final_count}");
}
