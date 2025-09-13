use std::hash::Hash;

use crate::prelude::*;

impl<I> Identifier for I where I: Component + Copy + Hash + Eq {}

macro_rules! blanket_impl {
    ($trait: ident, $t: tt) => {
        impl<T> $trait<$t> for T
        where
            T: Component,
            for<'a> &'a T: Into<$t>,
        {
            fn sample(component: &Self) -> $t
            {
                component.into()
            }
        }
    };
}

blanket_impl!(Sample, usize);
blanket_impl!(Sample, u8);
blanket_impl!(Sample, u16);
blanket_impl!(Sample, u32);
blanket_impl!(Sample, u64);
blanket_impl!(Sample, u128);
blanket_impl!(Sample, i8);
blanket_impl!(Sample, i16);
blanket_impl!(Sample, i32);
blanket_impl!(Sample, i64);
blanket_impl!(Sample, i128);
blanket_impl!(Sample, f32);
blanket_impl!(Sample, f64);
