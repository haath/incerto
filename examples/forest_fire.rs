//! # Monte Carlo simulation of forest fire spread.
//!
//! This example showcases a spatial cellular automaton simulation where fire spreads
//! through a forest based on probabilistic rules. The simulation demonstrates:
//!
//! * Spatial grid-based entities using the `SpatialGridPlugin`
//! * Entity state transitions (Healthy → Burning → Burned → Empty)
//! * Neighborhood interactions for fire spreading
//! * Time series collection of fire statistics
//! * Configurable fire spread parameters for Monte Carlo analysis
//!
//! Each cell in the forest can be in one of four states:
//! * **Healthy**: Can catch fire from burning neighbors
//! * **Burning**: Spreads fire to healthy neighbors, burns for a duration
//! * **Burned**: No longer spreads fire, can't burn again
//! * **Empty**: Vacant land that can regrow over time
//!
//! The simulation allows for studying fire spread patterns, firebreak effectiveness,
//! and forest management strategies under different conditions.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_precision_loss)]

use std::collections::HashSet;

use bevy::prelude::IVec2;
use incerto::prelude::*;
use rand::prelude::*;

// Simulation parameters
const SIMULATION_STEPS: usize = 500;
const GRID_WIDTH: i32 = 50;
const GRID_HEIGHT: i32 = 50;

// Fire parameters
const INITIAL_FOREST_DENSITY: f64 = 0.7; // Probability a cell starts as forest
const FIRE_SPREAD_PROBABILITY: f64 = 0.6; // Probability fire spreads to neighbor
const BURN_DURATION: usize = 3; // Steps a cell burns before becoming burned
const REGROWTH_PROBABILITY: f64 = 0.001; // Probability empty cell becomes forest
const INITIAL_FIRE_COUNT: usize = 3; // Number of initial fire sources

// Time series sampling
const SAMPLE_INTERVAL: usize = 1;

/// Represents the state of a forest cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellState
{
    /// Healthy forest that can catch fire
    Healthy,
    /// Currently burning, will spread fire
    Burning
    {
        remaining_burn_time: usize
    },
    /// Already burned, cannot burn again
    Burned,
    /// Empty land that can regrow
    #[default]
    Empty,
}

/// Component representing a single cell in the forest grid.
#[derive(Component, Debug, Default)]
pub struct ForestCell
{
    pub state: CellState,
}

/// Fire statistics collected during simulation.
#[derive(Debug, Clone, Copy)]
pub struct FireStats
{
    pub healthy_count: usize,
    pub burning_count: usize,
    pub burned_count: usize,
    pub empty_count: usize,
    pub total_cells: usize,
}

impl FireStats
{
    #[must_use]
    pub fn fire_activity(&self) -> f64
    {
        self.burning_count as f64 / self.total_cells as f64
    }

    #[must_use]
    pub fn forest_coverage(&self) -> f64
    {
        (self.healthy_count + self.burning_count) as f64 / self.total_cells as f64
    }

    #[must_use]
    pub fn burned_percentage(&self) -> f64
    {
        self.burned_count as f64 / self.total_cells as f64
    }
}

/// Implement sampling to collect fire statistics.
impl SampleAggregate<FireStats> for ForestCell
{
    fn sample_aggregate(components: &[&Self]) -> FireStats
    {
        assert!(!components.is_empty());

        let mut healthy_count = 0;
        let mut burning_count = 0;
        let mut burned_count = 0;
        let mut empty_count = 0;

        for cell in components
        {
            match cell.state
            {
                CellState::Healthy => healthy_count += 1,
                CellState::Burning { .. } => burning_count += 1,
                CellState::Burned => burned_count += 1,
                CellState::Empty => empty_count += 1,
            }
        }

        FireStats {
            healthy_count,
            burning_count,
            burned_count,
            empty_count,
            total_cells: components.len(),
        }
    }
}

fn main()
{
    println!("🔥 Starting Forest Fire Simulation");
    println!("Grid size: {GRID_WIDTH}x{GRID_HEIGHT}");
    println!(
        "Initial forest density: {:.1}%",
        INITIAL_FOREST_DENSITY * 100.0
    );
    println!(
        "Fire spread probability: {:.1}%",
        FIRE_SPREAD_PROBABILITY * 100.0
    );
    println!("Simulation steps: {SIMULATION_STEPS}");
    println!();

    // Build the simulation
    let bounds = GridBounds2D {
        min: IVec2::new(0, 0),
        max: IVec2::new(GRID_WIDTH - 1, GRID_HEIGHT - 1),
    };
    let mut simulation = SimulationBuilder::new()
        // Add spatial grid support
        .add_spatial_grid::<IVec2, ForestCell>(Some(bounds))
        // Spawn the forest grid
        .add_entity_spawner(spawn_forest_grid)
        // Add fire spread system
        .add_systems(fire_spread_system)
        // Add burn progression system
        .add_systems(burn_progression_system)
        // Add regrowth system
        .add_systems(regrowth_system)
        // Record time series of fire statistics
        .record_aggregate_time_series::<ForestCell, FireStats>(SAMPLE_INTERVAL)
        .expect("Failed to set up time series recording")
        .build();

    // Run the simulation
    println!("Running simulation...");
    simulation.run(SIMULATION_STEPS);

    // Collect and display results
    let final_stats = simulation
        .sample::<ForestCell, FireStats>()
        .expect("Failed to sample fire statistics");

    println!("📊 Final Statistics:");
    println!(
        "  Healthy forest: {} cells ({:.1}%)",
        final_stats.healthy_count,
        final_stats.healthy_count as f64 / final_stats.total_cells as f64 * 100.0
    );
    println!(
        "  Currently burning: {} cells ({:.1}%)",
        final_stats.burning_count,
        final_stats.fire_activity() * 100.0
    );
    println!(
        "  Burned areas: {} cells ({:.1}%)",
        final_stats.burned_count,
        final_stats.burned_percentage() * 100.0
    );
    println!(
        "  Empty land: {} cells ({:.1}%)",
        final_stats.empty_count,
        final_stats.empty_count as f64 / final_stats.total_cells as f64 * 100.0
    );

    // Display time series summary
    let time_series = simulation
        .get_aggregate_time_series::<ForestCell, FireStats>()
        .expect("Failed to get time series data");

    println!("\n📈 Time Series Summary:");
    println!("  Data points collected: {}", time_series.len());

    if let Some(peak_fire) = time_series
        .iter()
        .max_by(|a, b| a.burning_count.cmp(&b.burning_count))
    {
        println!(
            "  Peak fire activity: {} burning cells ({:.1}%)",
            peak_fire.burning_count,
            peak_fire.fire_activity() * 100.0
        );
    }

    let total_burned = time_series.last().map_or(0, |stats| stats.burned_count);
    println!(
        "  Total area burned: {} cells ({:.1}%)",
        total_burned,
        total_burned as f64 / final_stats.total_cells as f64 * 100.0
    );
}

/// Spawn the initial forest grid with random forest coverage.
fn spawn_forest_grid(spawner: &mut Spawner)
{
    let mut rng = rand::rng();

    // Spawn all grid cells
    for x in 0..GRID_WIDTH
    {
        for y in 0..GRID_HEIGHT
        {
            let position = GridPosition2D::new(x, y);

            // Determine initial state
            let state = if rng.random_bool(INITIAL_FOREST_DENSITY)
            {
                CellState::Healthy
            }
            else
            {
                CellState::Empty
            };

            let cell = ForestCell { state };
            spawner.spawn((position, cell));
        }
    }

    // Start some initial fires at random locations
    let healthy_positions: Vec<GridPosition2D> = (0..GRID_WIDTH)
        .flat_map(|x| (0..GRID_HEIGHT).map(move |y| GridPosition2D::new(x, y)))
        .collect();

    // This is a simplified approach - in a real implementation you'd query existing entities
    // For this example, we'll start fires by spawning burning cells at random positions
    for _ in 0..INITIAL_FIRE_COUNT
    {
        if let Some(&pos) = healthy_positions.choose(&mut rng)
        {
            let burning_cell = ForestCell {
                state: CellState::Burning {
                    remaining_burn_time: BURN_DURATION,
                },
            };
            spawner.spawn((pos, burning_cell));
        }
    }
}

/// System that handles fire spreading to neighboring cells using the spatial grid.
fn fire_spread_system(
    spatial_grid: Res<SpatialGrid<IVec2, ForestCell>>,
    query_burning: Query<(Entity, &GridPosition2D), With<ForestCell>>,
    mut query_cells: Query<(&GridPosition2D, &mut ForestCell)>,
)
{
    let mut rng = rand::rng();
    let mut spread_positions = HashSet::new();

    // Find all burning cells
    for (burning_entity, burning_pos) in &query_burning
    {
        // Check if this cell is actually burning
        if let Ok((_, cell)) = query_cells.get(burning_entity)
            && matches!(cell.state, CellState::Burning { .. })
        {
            // Get orthogonal neighbors using the spatial grid
            let neighbors = spatial_grid.orthogonal_neighbors_of(burning_pos);

            for neighbor_entity in neighbors
            {
                if let Ok((neighbor_pos, neighbor_cell)) = query_cells.get(neighbor_entity)
                {
                    // Check if neighbor is healthy and can catch fire
                    if matches!(neighbor_cell.state, CellState::Healthy)
                    {
                        // Fire spreads with probability
                        if rng.random_bool(FIRE_SPREAD_PROBABILITY)
                        {
                            spread_positions.insert(*neighbor_pos);
                        }
                    }
                }
            }
        }
    }

    // Apply fire spread
    for (position, mut cell) in &mut query_cells
    {
        if spread_positions.contains(position)
        {
            cell.state = CellState::Burning {
                remaining_burn_time: BURN_DURATION,
            };
        }
    }
}

/// System that progresses burning cells through their burn cycle.
fn burn_progression_system(mut query: Query<&mut ForestCell>)
{
    for mut cell in &mut query
    {
        if let CellState::Burning {
            remaining_burn_time,
        } = &mut cell.state
        {
            if *remaining_burn_time > 1
            {
                *remaining_burn_time -= 1;
            }
            else
            {
                // Fire burns out, cell becomes burned
                cell.state = CellState::Burned;
            }
        }
    }
}

/// System that handles forest regrowth on empty land.
fn regrowth_system(mut query: Query<&mut ForestCell>)
{
    let mut rng = rand::rng();

    for mut cell in &mut query
    {
        if matches!(cell.state, CellState::Empty) && rng.random_bool(REGROWTH_PROBABILITY)
        {
            cell.state = CellState::Healthy;
        }
    }
}
