#![cfg_attr(not(test), no_std)]
extern crate alloc;

pub mod list;
pub mod map;

pub use self::list::InversionList;
pub use self::map::InversionMap;

mod util;

pub trait OrderedIndex: Sized + Copy + PartialOrd + Ord + Eq + core::fmt::Debug {
    fn one() -> Self;
    fn min_value() -> Self;
    fn max_value() -> Self;
    fn checked_add(self, v: Self) -> Option<Self>;
    fn checked_sub(self, v: Self) -> Option<Self>;
}

macro_rules! impl_prim {
    ($($ty:ty)*) => {
        $(
            impl OrderedIndex for $ty {
                fn one() -> Self { 1 }
                fn min_value() -> Self { Self::MIN }
                fn max_value() -> Self { Self::MAX }
                fn checked_add(self, v: Self) -> Option<Self> {
                    Self::checked_add(self, v)
                }
                fn checked_sub(self, v: Self) -> Option<Self> {
                    Self::checked_sub(self, v)
                }
            }
        )*
    };
}
impl_prim! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }
