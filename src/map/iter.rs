use core::iter::{FusedIterator, IntoIterator};
use core::ops::Range;

use crate::map::{Entry, InversionMap};
use crate::OrderedIndex;

impl<Idx: OrderedIndex, V> InversionMap<Idx, V> {
    /// An iterator over the inner ranges contained in this list.
    pub fn iter(&self) -> Iter<Idx, V> {
        Iter {
            iter: self.ranges.iter(),
        }
    }

    // pub fn difference<'this>(&'this self, other: &'this Self) -> Difference<'this, Idx, V> {
    //     Difference {
    //         iter: self.ranges.into_iter(),
    //         other,
    //     }
    // }

    // pub fn symmetric_difference<'this>(
    //     &'this self,
    //     other: &'this Self,
    // ) -> SymmetricDifference<'this, Idx, V> {
    //     SymmetricDifference {
    //         iter: self.difference(other).chain(other.difference(self)),
    //     }
    // }

    // pub fn intersection<'this>(&'this self, other: &'this Self) -> Intersection<'this, Idx, V> {
    //     let (iter, other) = if self.len() <= other.len() {
    //         (self.iter(), other)
    //     } else {
    //         (other.iter(), self)
    //     };
    //     Intersection { iter, other }
    // }

    // pub fn union<'this>(&'this self, other: &'this Self) -> Union<'this, Idx, V> {
    //     Union {
    //         iter: if self.len() >= other.len() {
    //             self.iter().chain(other.difference(self))
    //         } else {
    //             other.iter().chain(self.difference(other))
    //         },
    //     }
    // }
}

pub struct Iter<'il, Idx: OrderedIndex, V> {
    iter: core::slice::Iter<'il, Entry<Idx, V>>,
}

impl<'a, Idx: OrderedIndex, V> Iterator for Iter<'a, Idx, V> {
    type Item = (Range<Idx>, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|entry| (entry.range.clone(), &entry.value))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Idx: OrderedIndex, V> FusedIterator for Iter<'_, Idx, V> {}
impl<Idx: OrderedIndex, V> ExactSizeIterator for Iter<'_, Idx, V> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'il, Idx: OrderedIndex, V> IntoIterator for &'il InversionMap<Idx, V> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = Iter<'il, Idx, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.ranges.iter(),
        }
    }
}

pub struct IntoIter<Idx: OrderedIndex, V> {
    iter: alloc::vec::IntoIter<Entry<Idx, V>>,
}

impl<Idx: OrderedIndex, V> Iterator for IntoIter<Idx, V> {
    type Item = (Range<Idx>, V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|entry| (entry.range, entry.value))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Idx: OrderedIndex, V> FusedIterator for IntoIter<Idx, V> {}
impl<Idx: OrderedIndex, V> ExactSizeIterator for IntoIter<Idx, V> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Idx: OrderedIndex, V> IntoIterator for InversionMap<Idx, V> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = IntoIter<Idx, V>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.ranges.into_iter(),
        }
    }
}

// pub struct Difference<'il, Idx: OrderedIndex = usize, V> {
//     pub(crate) iter: Iter<'il, Idx, V>,
//     pub(crate) other: &'il InversionMap<Idx, V>,
// }

// impl<Idx: OrderedIndex, V> Iterator for Difference<'_, Idx, V> {
//     type Item = Entry<Idx, V>;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             let next = self.iter.next()?;
//             if !self.other.contains_range(next.clone()) {
//                 break Some(next);
//             }
//         }
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let (_, upper) = self.iter.size_hint();
//         (0, upper)
//     }
// }

// impl<Idx: OrderedIndex, V> FusedIterator for Difference<'_, Idx, V> {}

// pub struct SymmetricDifference<'il, Idx: OrderedIndex = usize, V> {
//     pub(crate) iter: Chain<Difference<'il, Idx, V>, Difference<'il, Idx, V>>,
// }

// impl<Idx: OrderedIndex, V> Iterator for SymmetricDifference<'_, Idx, V> {
//     type Item = Entry<Idx, V>;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl<Idx: OrderedIndex, V> FusedIterator for SymmetricDifference<'_, Idx, V> {}

// pub struct Union<'il, Idx: OrderedIndex = usize, V> {
//     pub(crate) iter: Chain<Iter<'il, Idx, V>, Difference<'il, Idx, V>>,
// }

// impl<Idx: OrderedIndex, V> Iterator for Union<'_, Idx, V> {
//     type Item = Entry<Idx, V>;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl<Idx: OrderedIndex, V> FusedIterator for Union<'_, Idx, V> {}

// pub struct Intersection<'il, Idx: OrderedIndex = usize, V> {
//     pub(crate) iter: Iter<'il, Idx, V>,
//     pub(crate) other: &'il InversionMap<Idx, V>,
// }

// impl<Idx: OrderedIndex, V> Iterator for Intersection<'_, Idx, V> {
//     type Item = Entry<Idx, V>;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             let next = self.iter.next()?;
//             if self.other.contains_range(next.clone()) {
//                 break Some(next);
//             }
//         }
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let (_, upper) = self.iter.size_hint();
//         (0, upper)
//     }
// }

// impl<Idx: OrderedIndex, V> FusedIterator for Intersection<'_, Idx, V> {}
