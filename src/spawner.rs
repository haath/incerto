use bevy::prelude::*;

pub struct Spawner<'a>(pub(crate) &'a mut World);

impl Spawner<'_>
{
    /// Spawns a single entity in the simulation.
    pub fn spawn(&mut self, entity: impl Bundle)
    {
        self.0.spawn(entity);
    }
}
