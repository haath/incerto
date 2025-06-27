# incerto

[![Crates.io Version](https://img.shields.io/crates/v/incerto)](https://crates.io/crates/incerto) [![docs.rs](https://img.shields.io/docsrs/incerto)](https://docs.rs/incerto/latest/incerto/) [![Crates.io License](https://img.shields.io/crates/l/incerto)](https://github.com/haath/incerto/blob/main/LICENSE)

Rust crate for heavyweight multi-threaded Monte Carlo simulations.


## Installation

The crate can be installed from [crates.io](https://crates.io/crates/incerto).
Currently the only dependency is [bevy@0.16](https://github.com/bevyengine/bevy), and there are no cargo features.

```sh
cargo add incerto
```


## Usage

This crate is powered by [Bevy](https://github.com/bevyengine/bevy), which is a high-performance ECS framework.

This means that simulations are set up and executed using [Entities](https://bevy-cheatbook.github.io/programming/ec.html) and [Systems](https://bevy-cheatbook.github.io/programming/systems.html).

In-depth knowledge of Bevy's internals is not required however, since we have abstracted away most interactions with Bevy. Instead, we expect the user to only:

- Define components.
- Spawn entities, each a collection of one or more components.
- Implement systems that update the entities on each simulation step.


```rust
use incerto::prelude::*;

let simulation: Simulation = SimulationBuilder::new()
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


#### Define components

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


#### Spawn entities

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


#### Implement systems

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


#### Running the simulation

The simulation may be executed using the `run()`, `reset()` and `run_new()` methods.

```rust
// Run the simulation for 100 steps.
simulation.run(100);

// Continue the same simulation for another 200 steps.
simulation.run(200);

// Run the simulation for 500 steps from the beginning.
simulation.reset();
simulation.run(500);

// Can also be done like this.
simulation.run_new(500);
```


#### Collecting results

Currently the following ways of fetching simulation results are supported.

- Count the number of entities with a component `C` by calling `simulation.count::<C>()`.
- Read out a value of the sole existing entity with component `C` by implementing `ObserveSingle` for `C` and then calling `simulation.observe_single::<C>()`.
- Read out a value aggregated from multiple existing entities with component `C` by implementing `ObserveMany` for `C` and then calling `simulation.observe_many::<C>()`.


## Performance

When it comes to experiments like Monte Carlo, performance is typically of paramount importance since it defines their limits in terms of scope, size, length and granularity. Hence why I made the decision build this crate on top of bevy. The ECS architecture on offer here is likely the most memory-efficient and parallelizable way one can build such simulations, while still maintaining some agency of high-level programming.

Bevy has proven that it can handle worlds with hundreds of thousands (maybe even millions) of entities without slowing down enough to compromise 3D rendering at 60 frames per second.
And given that this crate adds practically no runtime overhead, your monte carlo experiments will likely be limited only by your hardware and your imagination.

You get to enjoy all the performance gains of the ECS automatically. However there are a few things you may want to keep in mind.

- **Temporal granularity:**
    This is just a fancy way of saying `how much time is each simulated step?`. The crate itself makes no mention of time, and treats each simulation as a series of discrete equitemporal steps. Whether each step represents one minute, one hour, or one day, is up to the user and likely contextual to the kind of experiment being conducted. For example, each step might represent one hour when modelling the weather, or one day when modelling pandemic infection rates.
    As such, there are great performance gains to be found by moving up a level in granularity. If you can manage to model the changes in the simulation in 5-minute steps instead of 1-minute steps, the simulation will magically run in one fifth of the time!
- **System parallelization:**
    Bevy's scheduler will automatically place disjoint systems on separate threads whenever possible.
    Two systems are disjoint when one's queries do not mutate components that the other is also accessing.
    The rule of thumb to achieve this whenever possible, is to design each system such that:
    - It has a singular purpose.
    - Only queries for components that it definitely needs.
- **Singular components:**
    It may be tempting to simplify entity design by putting all of an entity's data in a single component, especially if one is used to object-oriented languages. However, doing so will impact your performance in the long term since it would render system parallelization neigh impossible.
    The general recommendation is to favor composition, meaning that each distinct attribute of an entity should be in a separate component. Imagine, for example, how since a person's age and body temperature are largely independent, systems attempting to read or update these values should be allowed to run in parallel.
- **Entity archetypes:**
    Bevy likes to put similar-looking entities together in groups called *archetypes*, which enables it to more efficiently store such entities in shared tables. So if components are added to or removed from existing entities at runtime the archetype tables have to be remade, which is a drain on performance.
    So in case where an entity's state needs to change often in the simulation, consider using persistent enums instead.



## Planned work

- Allow for filters like `With` and `Without` when observing values.
- Add support for recoding time series during the simulation.
- Add some utilities to the crate for easy access to random values, noise etc


## Credits

The name as well as the initial motivation behind this project came from the brilliant [Incerto](https://www.goodreads.com/series/164555-incerto) book series by Nassim Nicholas Taleb.
