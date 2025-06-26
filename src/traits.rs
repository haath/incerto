use bevy::prelude::*;

pub trait CollectSingle: Component
{
    type Out;

    fn collect(&self) -> Self::Out;
}
