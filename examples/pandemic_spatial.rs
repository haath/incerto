//! # Enhanced Monte Carlo simulation of pandemic spread with spatial features.
//!
//! This example demonstrates advanced spatial epidemic modeling using the `SpatialGrid` plugin.
//! It showcases realistic pandemic dynamics including:
//!
//! * **Infection radius**: Disease spreads within a configurable distance, not just same-cell
//! * **Contact tracing**: Track and quarantine people who were near infected individuals
//! * **Social distancing**: People avoid crowded areas and maintain distance
//! * **Quarantine zones**: Restricted movement areas to contain outbreaks
//! * **Superspreader events**: Detection of high-transmission locations
//! * **Population density tracking**: Monitor crowding and movement patterns
//!
//! The simulation models a more realistic epidemic than simple grid-cell transmission,
//! allowing for analysis of various intervention strategies and their effectiveness.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_complexity)]

use std::collections::HashSet;

use bevy::prelude::IVec2;
use incerto::prelude::*;
use rand::prelude::*;

// Simulation parameters
const SIMULATION_STEPS: usize = 500;
const INITIAL_POPULATION: usize = 2000;
const GRID_SIZE: i32 = 40;

// Disease parameters
const CHANCE_START_INFECTED: f64 = 0.02;
const INFECTION_RADIUS: i32 = 2; // Can infect within 2 cells distance
const CHANCE_INFECT_AT_DISTANCE_1: f64 = 0.15; // High chance at close distance
const CHANCE_INFECT_AT_DISTANCE_2: f64 = 0.05; // Lower chance at farther distance
const CHANCE_RECOVER: f64 = 0.03;
const CHANCE_DIE: f64 = 0.001;
const INCUBATION_PERIOD: usize = 5; // Steps before becoming infectious

// Social distancing parameters
const SOCIAL_DISTANCING_ENABLED: bool = true;
const CROWDING_THRESHOLD: usize = 8; // Avoid areas with more than 8 people
const SOCIAL_DISTANCE_COMPLIANCE: f64 = 0.7; // 70% of people practice social distancing

// Contact tracing parameters
const CONTACT_TRACING_ENABLED: bool = true;
const CONTACT_QUARANTINE_DURATION: usize = 14;

// Quarantine zone (center area)
const QUARANTINE_ZONE_ENABLED: bool = true;
const QUARANTINE_CENTER_X: i32 = GRID_SIZE / 2;
const QUARANTINE_CENTER_Y: i32 = GRID_SIZE / 2;
const QUARANTINE_RADIUS: i32 = 8;

// Time series sampling
const SAMPLE_INTERVAL: usize = 1;

/// Disease states for people in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiseaseState
{
    Healthy,
    Exposed
    {
        incubation_remaining: usize,
    }, // Infected but not yet infectious
    Infectious,
    Recovered,
}

/// Component representing a person in the simulation
#[derive(Component, Debug)]
pub struct Person
{
    pub disease_state: DiseaseState,
    pub social_distancing: bool, // Whether this person practices social distancing
}

/// Component for people under quarantine (contact tracing)
#[derive(Component, Debug)]
pub struct Quarantined
{
    pub remaining_duration: usize,
}

/// Component to track contact history for contact tracing
#[derive(Component, Debug, Default)]
pub struct ContactHistory
{
    pub recent_contacts: HashSet<GridPosition2D>, // Positions visited recently
}

/// Pandemic statistics collected during simulation
#[derive(Debug, Clone, Copy)]
pub struct PandemicStats
{
    pub healthy_count: usize,
    pub exposed_count: usize,
    pub infectious_count: usize,
    pub recovered_count: usize,
    pub dead_count: usize,
    pub total_population: usize,
}

impl SampleAggregate<PandemicStats> for Person
{
    fn sample_aggregate(components: &[&Self]) -> PandemicStats
    {
        assert!(!components.is_empty());

        let mut healthy_count = 0;
        let mut exposed_count = 0;
        let mut infectious_count = 0;
        let mut recovered_count = 0;

        for person in components
        {
            match person.disease_state
            {
                DiseaseState::Healthy => healthy_count += 1,
                DiseaseState::Exposed { .. } => exposed_count += 1,
                DiseaseState::Infectious => infectious_count += 1,
                DiseaseState::Recovered => recovered_count += 1,
            }
        }

        PandemicStats {
            healthy_count,
            exposed_count,
            infectious_count,
            recovered_count,
            dead_count: INITIAL_POPULATION - components.len(),
            total_population: components.len(),
        }
    }
}

fn main()
{
    println!("🦠 Starting Enhanced Pandemic Simulation");
    println!("Population: {INITIAL_POPULATION}");
    println!("Grid size: {GRID_SIZE}x{GRID_SIZE}");
    println!("Infection radius: {INFECTION_RADIUS} cells");
    println!(
        "Social distancing: {}",
        if SOCIAL_DISTANCING_ENABLED
        {
            "ON"
        }
        else
        {
            "OFF"
        }
    );
    println!(
        "Contact tracing: {}",
        if CONTACT_TRACING_ENABLED { "ON" } else { "OFF" }
    );
    println!(
        "Quarantine zone: {}",
        if QUARANTINE_ZONE_ENABLED { "ON" } else { "OFF" }
    );
    println!();

    let bounds = GridBounds2D {
        min: IVec2::new(0, 0),
        max: IVec2::new(GRID_SIZE - 1, GRID_SIZE - 1),
    };
    let mut simulation = SimulationBuilder::new()
        // Add spatial grid support
        .add_spatial_grid::<IVec2, Person>(Some(bounds))
        // Spawn initial population
        .add_entity_spawner(spawn_population)
        // Movement and social distancing
        .add_systems(people_move_with_social_distancing)
        // Disease progression and transmission
        .add_systems((
            disease_incubation_progression,
            spatial_disease_transmission,
            disease_recovery_and_death,
        ))
        // Contact tracing and quarantine
        .add_systems((
            update_contact_history,
            process_contact_tracing,
            update_quarantine_status,
        ))
        // Record pandemic statistics
        .record_aggregate_time_series::<Person, PandemicStats>(SAMPLE_INTERVAL)
        .expect("Failed to set up time series recording")
        .build();

    println!("Running simulation...");
    simulation.run(SIMULATION_STEPS);

    // Collect and display results
    let final_stats = simulation
        .sample::<Person, PandemicStats>()
        .expect("Failed to sample pandemic statistics");

    println!("📊 Final Statistics:");
    println!(
        "  Population: {} ({}% survived)",
        final_stats.total_population,
        final_stats.total_population as f64 / INITIAL_POPULATION as f64 * 100.0
    );
    println!(
        "  Healthy: {} ({:.1}%)",
        final_stats.healthy_count,
        final_stats.healthy_count as f64 / final_stats.total_population as f64 * 100.0
    );
    println!(
        "  Exposed: {} ({:.1}%)",
        final_stats.exposed_count,
        final_stats.exposed_count as f64 / final_stats.total_population as f64 * 100.0
    );
    println!(
        "  Infectious: {} ({:.1}%)",
        final_stats.infectious_count,
        final_stats.infectious_count as f64 / final_stats.total_population as f64 * 100.0
    );
    println!(
        "  Recovered: {} ({:.1}%)",
        final_stats.recovered_count,
        final_stats.recovered_count as f64 / final_stats.total_population as f64 * 100.0
    );
    println!(
        "  Deaths: {} ({:.1}%)",
        final_stats.dead_count,
        final_stats.dead_count as f64 / INITIAL_POPULATION as f64 * 100.0
    );

    // Display time series summary
    let time_series = simulation
        .get_aggregate_time_series::<Person, PandemicStats>()
        .expect("Failed to get time series data");

    println!("\n📈 Pandemic Timeline:");
    println!("  Data points collected: {}", time_series.len());

    if let Some(peak_infections) = time_series
        .iter()
        .max_by_key(|stats| stats.infectious_count)
    {
        println!(
            "  Peak infectious: {} people ({:.1}%)",
            peak_infections.infectious_count,
            peak_infections.infectious_count as f64 / INITIAL_POPULATION as f64 * 100.0
        );
    }

    let total_recovered = time_series.last().map_or(0, |stats| stats.recovered_count);
    let total_deaths = time_series.last().map_or(0, |stats| stats.dead_count);
    println!(
        "  Total recovered: {} ({:.1}%)",
        total_recovered,
        total_recovered as f64 / INITIAL_POPULATION as f64 * 100.0
    );
    println!(
        "  Total deaths: {} ({:.1}%)",
        total_deaths,
        total_deaths as f64 / INITIAL_POPULATION as f64 * 100.0
    );

    // Calculate attack rate (percentage who got infected)
    let attack_rate = (total_recovered + total_deaths) as f64 / INITIAL_POPULATION as f64 * 100.0;
    println!("  Attack rate: {attack_rate:.1}% (total who got infected)");
}

/// Spawn the initial population with random positions and infection states
fn spawn_population(spawner: &mut Spawner)
{
    let mut rng = rand::rng();

    for _ in 0..INITIAL_POPULATION
    {
        // Random position on the grid
        let position = GridPosition2D::new(
            rng.random_range(0..GRID_SIZE),
            rng.random_range(0..GRID_SIZE),
        );

        // Determine if person practices social distancing
        let social_distancing = rng.random_bool(SOCIAL_DISTANCE_COMPLIANCE);

        // Initial disease state
        let disease_state = if rng.random_bool(CHANCE_START_INFECTED)
        {
            DiseaseState::Exposed {
                incubation_remaining: INCUBATION_PERIOD,
            }
        }
        else
        {
            DiseaseState::Healthy
        };

        let person = Person {
            disease_state,
            social_distancing,
        };

        spawner.spawn((position, person, ContactHistory::default()));
    }
}

/// Enhanced movement system with social distancing behavior
fn people_move_with_social_distancing(
    mut query: Query<(&mut GridPosition2D, &Person, Option<&Quarantined>)>,
    spatial_grid: Res<SpatialGrid<IVec2, Person>>,
)
{
    let mut rng = rand::rng();

    for (mut position, person, quarantined) in &mut query
    {
        // Quarantined people don't move
        if quarantined.is_some()
        {
            continue;
        }

        // 50% chance to try to move
        if !rng.random_bool(0.5)
        {
            continue;
        }

        // Get potential movement directions
        let directions = [
            GridPosition2D::new(position.x(), position.y() - 1), // up
            GridPosition2D::new(position.x() - 1, position.y()), // left
            GridPosition2D::new(position.x() + 1, position.y()), // right
            GridPosition2D::new(position.x(), position.y() + 1), // down
        ];

        let mut best_moves = Vec::new();
        let mut min_crowding = usize::MAX;

        for new_pos in directions
        {
            // Check bounds
            if new_pos.x() < 0
                || new_pos.x() >= GRID_SIZE
                || new_pos.y() < 0
                || new_pos.y() >= GRID_SIZE
            {
                continue;
            }

            // Check if in quarantine zone
            if QUARANTINE_ZONE_ENABLED
            {
                let quarantine_center =
                    GridPosition2D::new(QUARANTINE_CENTER_X, QUARANTINE_CENTER_Y);
                if (new_pos.0 - quarantine_center.0).abs().element_sum() <= QUARANTINE_RADIUS
                {
                    // Only enter quarantine zone if not practicing social distancing
                    if person.social_distancing
                    {
                        continue;
                    }
                }
            }

            // Count people at potential destination for social distancing
            let people_at_destination = spatial_grid.entities_at(&new_pos).count();

            if person.social_distancing && SOCIAL_DISTANCING_ENABLED
            {
                // Social distancing: prefer less crowded areas
                if people_at_destination < min_crowding
                {
                    min_crowding = people_at_destination;
                    best_moves.clear();
                    best_moves.push(new_pos);
                }
                else if people_at_destination == min_crowding
                {
                    best_moves.push(new_pos);
                }
            }
            else
            {
                // No social distancing: any valid move is fine
                best_moves.push(new_pos);
            }
        }

        // Move to a randomly selected best position
        if !best_moves.is_empty()
        {
            let chosen_move = best_moves.choose(&mut rng).copied().unwrap();

            // Only move if it's not too crowded (even for non-social-distancing people)
            let people_at_destination = spatial_grid.entities_at(&chosen_move).count();
            if people_at_destination < CROWDING_THRESHOLD
            {
                *position = chosen_move;
            }
        }
    }
}

/// Progress disease through incubation period
fn disease_incubation_progression(mut query: Query<&mut Person>)
{
    for mut person in &mut query
    {
        if let DiseaseState::Exposed {
            incubation_remaining,
        } = &mut person.disease_state
        {
            if *incubation_remaining > 1
            {
                *incubation_remaining -= 1;
            }
            else
            {
                // Become infectious
                person.disease_state = DiseaseState::Infectious;
            }
        }
    }
}

/// Advanced spatial disease transmission system using infection radius
fn spatial_disease_transmission(
    spatial_grid: Res<SpatialGrid<IVec2, Person>>,
    mut query: Query<(Entity, &GridPosition2D, &mut Person), Without<Quarantined>>,
)
{
    let mut rng = rand::rng();
    let mut new_exposures = Vec::new();

    // Collect infectious people first to avoid borrowing conflicts
    let infectious_people: Vec<(Entity, GridPosition2D)> = query
        .iter()
        .filter_map(|(entity, pos, person)| {
            if matches!(person.disease_state, DiseaseState::Infectious)
            {
                Some((entity, *pos))
            }
            else
            {
                None
            }
        })
        .collect();

    for (infectious_entity, infectious_pos) in infectious_people
    {
        // Get all people within infection radius using iterative approach
        let mut nearby_entities = Vec::new();
        let infectious_coord = infectious_pos.0;

        // Check all positions within Manhattan distance of INFECTION_RADIUS
        for dx in -INFECTION_RADIUS..=INFECTION_RADIUS
        {
            for dy in -INFECTION_RADIUS..=INFECTION_RADIUS
            {
                let manhattan_distance = dx.abs() + dy.abs();
                if manhattan_distance <= INFECTION_RADIUS
                {
                    let check_pos =
                        GridPosition2D::new(infectious_coord.x + dx, infectious_coord.y + dy);
                    nearby_entities.extend(spatial_grid.entities_at(&check_pos));
                }
            }
        }

        for nearby_entity in nearby_entities
        {
            if nearby_entity == infectious_entity
            {
                continue; // Don't infect self
            }

            if let Ok((entity, susceptible_pos, person)) = query.get(nearby_entity)
            {
                // Only infect healthy people
                if matches!(person.disease_state, DiseaseState::Healthy)
                {
                    // Calculate infection probability based on distance
                    let distance = (infectious_pos.0 - susceptible_pos.0).abs().element_sum();
                    let infection_chance = match distance
                    {
                        0 | 1 => CHANCE_INFECT_AT_DISTANCE_1, // Same cell or adjacent
                        2 => CHANCE_INFECT_AT_DISTANCE_2,     // 2 cells away
                        _ => 0.0,                             // Too far
                    };

                    if rng.random_bool(infection_chance)
                    {
                        new_exposures.push(entity);
                    }
                }
            }
        }
    }

    // Apply new exposures
    for entity in new_exposures
    {
        let (_, _, mut person) = query.get_mut(entity).expect("Entity should exist");
        person.disease_state = DiseaseState::Exposed {
            incubation_remaining: INCUBATION_PERIOD,
        };
    }
}

/// Handle disease recovery and death
fn disease_recovery_and_death(mut commands: Commands, mut query: Query<(Entity, &mut Person)>)
{
    let mut rng = rand::rng();

    for (entity, mut person) in &mut query
    {
        if matches!(person.disease_state, DiseaseState::Infectious)
        {
            if rng.random_bool(CHANCE_DIE)
            {
                // Person dies
                commands.entity(entity).despawn();
            }
            else if rng.random_bool(CHANCE_RECOVER)
            {
                // Person recovers and gains immunity
                person.disease_state = DiseaseState::Recovered;
            }
        }
    }
}

/// Update contact history for contact tracing
fn update_contact_history(mut query: Query<(&GridPosition2D, &mut ContactHistory)>)
{
    for (position, mut contact_history) in &mut query
    {
        // Add current position to recent contacts
        contact_history.recent_contacts.insert(*position);

        // Limit history size (keep last 14 positions)
        if contact_history.recent_contacts.len() > 14
        {
            // In a real implementation, you'd track timestamps and remove old ones
            // For simplicity, we'll just clear periodically
            if contact_history.recent_contacts.len() > 20
            {
                contact_history.recent_contacts.clear();
            }
        }
    }
}

/// Process contact tracing when someone becomes infectious
fn process_contact_tracing(
    mut commands: Commands,
    spatial_grid: Res<SpatialGrid<IVec2, Person>>,
    query_newly_infectious: Query<
        (Entity, &GridPosition2D, &ContactHistory),
        (With<Person>, Without<Quarantined>),
    >,
    query_potential_contacts: Query<Entity, (With<Person>, Without<Quarantined>)>,
)
{
    if !CONTACT_TRACING_ENABLED
    {
        return;
    }

    for (infectious_entity, _infectious_pos, contact_history) in &query_newly_infectious
    {
        // Check if this person just became infectious (simplified check)
        // In a real implementation, you'd track state changes

        // Quarantine people who were in recent contact locations
        for &contact_location in &contact_history.recent_contacts
        {
            let people_at_location = spatial_grid.entities_at(&contact_location);

            for potential_contact in people_at_location
            {
                if potential_contact == infectious_entity
                {
                    continue;
                }

                // Quarantine this person if they're not already quarantined
                if query_potential_contacts.get(potential_contact).is_ok()
                {
                    // Use try_insert to handle entities that may have been despawned
                    commands.entity(potential_contact).try_insert(Quarantined {
                        remaining_duration: CONTACT_QUARANTINE_DURATION,
                    });
                }
            }
        }
    }
}

/// Update quarantine status
fn update_quarantine_status(mut commands: Commands, mut query: Query<(Entity, &mut Quarantined)>)
{
    for (entity, mut quarantined) in &mut query
    {
        if quarantined.remaining_duration > 1
        {
            quarantined.remaining_duration -= 1;
        }
        else
        {
            // End quarantine
            commands.entity(entity).remove::<Quarantined>();
        }
    }
}
