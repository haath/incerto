use std::hash::Hash;

use crate::prelude::*;

impl<I> Identifier for I where I: Component + Copy + Hash + Eq {}
