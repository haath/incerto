# incerto

Rust crate for heavyweight multi-threaded Monte Carlo simulations.


## Usage

This crate is powered by [Bevy](https://github.com/bevyengine/bevy), which is a high-performance ECS framework.

This means that simulations are set up and executed using [Entities](https://bevy-cheatbook.github.io/programming/ec.html) and [Systems](https://bevy-cheatbook.github.io/programming/systems.html).

In-depth knowledge of Bevy's internals is not required however, since we have abstracted away most interactions with Bevy. Instead, we expect the user to only:

- Define components.
- Spawn entities, each a collection of one or more components.
- Implement systems that update the entities on each simulation step.


```rust
use incerto::prelude::*;

let monte_carlo: MonteCarlo = MonteCarloBuilder::new()
                                // add one or more entity spawners
                                .add_entity_spawner(...)
                                .add_entity_spawner(...)
                                // add one or more systems
                                .add_systems(...)
                                .add_systems(...)
                                // finish
                                .build();
```

It is recommended to start with the [examples](examples/coin_toss.rs).


### Define components

Components will be the primary data type in the simulation.
They can be anything, so long as they can derive the `Component` trait.

```rust
#[derive(Component, Default)]
struct Counter
{
    count: usize,
}
```

Empty components, typically called *Markers*, are also sometimes useful to pick out specific entities.

```rust
#[derive(Component)]
struct GroupA;

#[derive(Component)]
struct GroupB;
```


### Spawn entities

Entities are spawned at the beginning of each simulation using user-provided functions like this one.

```rust
fn spawn_coin_tosser(spawner: &mut Spawner)
{
    spawner.spawn(Counter::default());
}
```

Note that entities are in fact collections of one or more components, as such the `spawn()` function accepts a [Bundle](https://bevy-cheatbook.github.io/programming/bundle.html). A bundle can be a single component like above, or a tuple with multiple components.

```rust
fn spawn_coin_tossers_in_groups(spawner: &mut Spawner)
{
    for _ in 0..100
    {
        spawner.spawn((GroupA, Counter::default()));
    }
    for _ in 0..100
    {
        spawner.spawn((GroupB, Counter::default()));
    }
}
```


### Implement systems

Systems are the processing logic of the simulation.
During each step, every user-defined system is executed once.
Systems use [queries](https://bevy-cheatbook.github.io/programming/queries.html) to interact with and update entities in the simulation.

```rust
/// Increment all counters by one in each simulation step.
fn counters_increment_system(mut query: Query<&mut Counter>)
{
    for mut counter in &mut query
    {
        counter.count += 1;
    }
}
```

Queries may use the `With` and `Without` keywords to filter their scope.

```rust
fn counters_increment_group_a(mut query: Query<&mut Counter, With<GroupA>>) { ... }

fn counters_increment_group_b(mut query: Query<&mut Counter, With<GroupB>>) { ... }
```

They may also select multiple components, mutably or immutably. (note the use of `mut`, `&` and `&mut`)

```rust
fn update_multiple_components(mut query: Query<(&mut Counter, &OtherComponent)>) { ... }

fn read_only_system(query: Query<&Counter>) { ... }
```

Systems may also work with multiple queries. This allows for entities in the simulation to interact with each other.

```rust
fn multiple_queries(
    read_from_group_a: Query<&Counter, With<GroupA>>,
    mut write_to_group_b: Query<&mut Counter, With<GroupB>>,
) { ... }
```


### Running the simulation

The simulation may be executed using the `run()`, `reset()` and `run_new()` methods.

```rust
// Run the simulation for 100 steps.
monte_carlo.run(100);

// Continue the same simulation for another 200 steps.
monte_carlo.run(200);

// Run the simulation for 500 steps from the beginning.
monte_carlo.reset();
monte_carlo.run(500);

// Can also be done like this.
monte_carlo.run_new(500);
```


### Collecting results

Currently the following ways of fetching simulation results are supported.

- Count the number of remaining entities with a component `C` by calling `monte_carlo.count::<C>()`.
- Read out a value of the sole existing entity with component `C` by implementing `ObserveSingle` for `C` and then calling `monte_carlo.observe_single::<C>()`.
- Read out a value aggregated from multiple existing entities with component `C` by implementing `ObserveMany` for `C` and then calling `monte_carlo.observe_many::<C>()`.


## Planned work

- Allow for filters like `With` and `Without` when observing values.
- Add support for recoding time series during the simulation.


## Credits

The name as well as the initial motivation behind this project came from the brilliant [Incerto](https://www.goodreads.com/series/164555-incerto) book series by Nassim Nicholas Taleb.
