//! Rust crate for heavyweight multi-threaded Monte Carlo simulations.
//!
//! This crate is powered by Bevy, which is a high-performance ECS framework.
//!
//! This means that simulations are set up and executed using Entities and Systems.
//!
//! In-depth knowledge of Bevy's internals is not required however, since we have abstracted away most interactions with Bevy. Instead, we expect the user to only:
//!
//! * Define components.
//! * Spawn entities, each a collection of one or more components.
//! * Implement systems that update the entities on each simulation step.
//!
//! All relevant types should be in the [`prelude`].
//! The primary type used to run experiments is [`MonteCarlo`].

pub mod prelude;

mod error;
mod monte_carlo;
mod monte_carlo_builder;
mod plugins;
mod simulation;
mod spawner;
mod traits;

pub use error::*;
pub use monte_carlo::MonteCarlo;
pub use monte_carlo_builder::MonteCarloBuilder;
pub use spawner::Spawner;
pub use traits::*;
