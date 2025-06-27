//! Monte Carlo simulation of the spread of an infectious disease.
//!
//! This example showcases a slightly complicated case, where out of an initial population
//! some get infected, and then by moving around randomly in a grid have a chance to infect others.
//! Additionally, on each simulation step there is a chance that an infected person may spontaneously
//! recover from the disease, or die.
//!
//! The goal is to count the survivors at the end of the simulation, and sample whether the pandemic
//! with the given parameters is capable of wiping out the population before disappearing.
//!
//! Note that the entities in this example interact with each other.

#![allow(clippy::unwrap_used)]
#![allow(clippy::same_functions_in_if_condition)]
use incerto::prelude::*;
use rand::prelude::*;

const SIMULATION_STEPS: usize = 100_000;
const INITIAL_POPULATION: usize = 1_000;
const GRID_SIZE: usize = 50;

const CHANCE_START_INFECTED: f64 = 0.05;

/// The probability that a person will infect another during a simulation step
/// in which they are both in the same grid cell.
const CHANCE_INFECT_OTHER: f64 = 0.02;

/// The probability that an infected person will recover.
const CHANCE_RECOVER: f64 = 0.001;

/// The probability that an infected person will die.
const CHANCE_DIE: f64 = 0.0005;

/// Marker component for every person in the simulation.
#[derive(Component)]
struct Person;

/// A position in a 2D grid.
#[derive(Component, Default, PartialEq, Eq)]
struct Position
{
    x: usize,
    y: usize,
}

/// Marker component indicating that a person is infected.
///
/// Note that adding/removing components to entities at runtime is
/// typically inefficient.
/// It is done here mainly to showcase how to do it by using [`Commands`].
///
/// In a real scenario it would be more performant to mark entities as
/// healthy or infected using an enum.
#[derive(Component)]
struct Infected;

fn main()
{
    // Build the simulation.
    let mut simulation = SimulationBuilder::new()
        .add_entity_spawner(spawn_people)
        .add_systems((
            people_move,
            people_infect_others,
            people_infected_may_die,
            people_infected_may_recover,
        ))
        .build();

    // Run the simulation once.
    simulation.run(SIMULATION_STEPS);

    // Count the number of survivors remaining.
    let survivors = simulation.count::<With<Person>>().unwrap();
    let infected = simulation.count::<With<Infected>>().unwrap();
    let healthy = simulation
        .count::<(With<Person>, Without<Infected>)>()
        .unwrap();

    println!("survivors: {survivors}, infected: {infected}, healthy: {healthy}");
}

/// Spawns the initial population placed uniformly around the grid.
fn spawn_people(spawner: &mut Spawner)
{
    let mut rng = rand::rng();

    for _ in 0..INITIAL_POPULATION
    {
        // each person starts somewhere randomly in the grid
        let position = Position {
            x: rng.random_range(0..GRID_SIZE),
            y: rng.random_range(0..GRID_SIZE),
        };
        // ... with a chance to already be infected
        let is_infected = rng.random_bool(CHANCE_START_INFECTED);

        if is_infected
        {
            spawner.spawn((Person, position, Infected));
        }
        else
        {
            spawner.spawn((Person, position));
        }
    }
}

/// Moves the people around.
///
/// On each step, each person has:
/// - a 50% chance to stay put.
/// - a 25% chance to move one cell left or right
/// - a 25% chance to move one cell up or down
fn people_move(mut query: Query<&mut Position>)
{
    let mut rng = rand::rng();

    for mut position in &mut query
    {
        if rng.random_bool(0.5)
        {
            // do nothing
        }
        else if rng.random_bool(0.5)
        {
            // move left or right
            if rng.random_bool(0.5)
            {
                // move left
                if position.x > 0
                {
                    position.x -= 1;
                }
            }
            else
            {
                // move right
                if position.x < GRID_SIZE - 1
                {
                    position.x += 1;
                }
            }
        }
        else
        {
            // move up or down
            if rng.random_bool(0.5)
            {
                // move left
                if position.y > 0
                {
                    position.y -= 1;
                }
            }
            else
            {
                // move right
                if position.y < GRID_SIZE - 1
                {
                    position.y += 1;
                }
            }
        }
    }
}

/// Each infected person attempts to infected each other healthy person in the same grid cell as them.
fn people_infect_others(
    mut commands: Commands,
    query_infected: Query<&Position, With<Infected>>,
    query_healthy: Query<(Entity, &Position), Without<Infected>>,
)
{
    let mut rng = rand::rng();

    for infected_pos in &query_infected
    {
        for (healthy, healthy_pos) in &query_healthy
        {
            if infected_pos == healthy_pos
            {
                // these two people are in the same grid cell
                if rng.random_bool(CHANCE_INFECT_OTHER)
                {
                    // the healthy person becomes infected
                    commands.entity(healthy).insert(Infected);
                }
            }
        }
    }
}

/// On each step, each infected person may die from the disease.
fn people_infected_may_die(mut commands: Commands, query_infected: Query<Entity, With<Infected>>)
{
    let mut rng = rand::rng();

    for infected in &query_infected
    {
        if rng.random_bool(CHANCE_DIE)
        {
            commands.entity(infected).despawn();
        }
    }
}

/// On each step, each infected person may recover from the disease.
fn people_infected_may_recover(
    mut commands: Commands,
    query_infected: Query<Entity, With<Infected>>,
)
{
    let mut rng = rand::rng();

    for infected in &query_infected
    {
        if rng.random_bool(CHANCE_RECOVER)
        {
            commands.entity(infected).remove::<Infected>();
        }
    }
}
