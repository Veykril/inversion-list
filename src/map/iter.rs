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
