use core::iter::{Chain, FusedIterator, IntoIterator};
use core::ops::Range;

use crate::list::InversionList;
use crate::{map, OrderedIndex};

impl<Ty: OrderedIndex> InversionList<Ty> {
    /// An iterator over the inner ranges contained in this list.
    pub fn iter(&self) -> Iter<Ty> {
        Iter {
            iter: self.0.iter(),
        }
    }

    /// Visits the elements representing the difference, i.e., the elements that are in self but not in other, in ascending order.
    pub fn difference<'this>(&'this self, other: &'this Self) -> Difference<'this, Ty> {
        Difference {
            iter: self.into_iter(),
            other,
        }
    }

    /// Visits the elements representing the symmetric difference, i.e., the elements that are in self or in other but not in both, in ascending order.
    pub fn symmetric_difference<'this>(
        &'this self,
        other: &'this Self,
    ) -> SymmetricDifference<'this, Ty> {
        SymmetricDifference {
            iter: self.difference(other).chain(other.difference(self)),
        }
    }

    /// Visits the elements representing the intersection, i.e., the elements that are both in self and other, in ascending order.
    pub fn intersection<'this>(&'this self, other: &'this Self) -> Intersection<'this, Ty> {
        let (iter, other) = if self.len() <= other.len() {
            (self.iter(), other)
        } else {
            (other.iter(), self)
        };
        Intersection { iter, other }
    }

    /// Visits the elements representing the union, i.e., all the elements in self or other, without duplicates, in ascending order.
    pub fn union<'this>(&'this self, other: &'this Self) -> Union<'this, Ty> {
        Union {
            iter: if self.len() >= other.len() {
                self.iter().chain(other.difference(self))
            } else {
                other.iter().chain(self.difference(other))
            },
        }
    }
}

pub struct Iter<'a, Ty: OrderedIndex> {
    iter: map::Iter<'a, Ty, ()>,
}

impl<Ty: OrderedIndex> Iterator for Iter<'_, Ty> {
    type Item = Range<Ty>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, ())| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Ty: OrderedIndex> FusedIterator for Iter<'_, Ty> {}
impl<Ty: OrderedIndex> ExactSizeIterator for Iter<'_, Ty> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, Ty: OrderedIndex> IntoIterator for &'a InversionList<Ty> {
    type Item = Range<Ty>;
    type IntoIter = Iter<'a, Ty>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntoIter<Ty: OrderedIndex> {
    iter: map::IntoIter<Ty, ()>,
}

impl<Ty: OrderedIndex> Iterator for IntoIter<Ty> {
    type Item = Range<Ty>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, ())| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Ty: OrderedIndex> FusedIterator for IntoIter<Ty> {}
impl<Ty: OrderedIndex> ExactSizeIterator for IntoIter<Ty> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Ty: OrderedIndex> IntoIterator for InversionList<Ty> {
    type Item = Range<Ty>;
    type IntoIter = IntoIter<Ty>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.0.into_iter(),
        }
    }
}

pub struct Difference<'a, Ty: OrderedIndex> {
    pub(crate) iter: Iter<'a, Ty>,
    pub(crate) other: &'a InversionList<Ty>,
}

impl<Ty: OrderedIndex> Iterator for Difference<'_, Ty> {
    type Item = Range<Ty>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.iter.next()?;
            if !self.other.contains_range(next.clone()) {
                break Some(next);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<Ty: OrderedIndex> FusedIterator for Difference<'_, Ty> {}

pub struct SymmetricDifference<'a, Ty: OrderedIndex> {
    pub(crate) iter: Chain<Difference<'a, Ty>, Difference<'a, Ty>>,
}

impl<Ty: OrderedIndex> Iterator for SymmetricDifference<'_, Ty> {
    type Item = Range<Ty>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Ty: OrderedIndex> FusedIterator for SymmetricDifference<'_, Ty> {}

pub struct Union<'a, Ty: OrderedIndex> {
    pub(crate) iter: Chain<Iter<'a, Ty>, Difference<'a, Ty>>,
}

impl<Ty: OrderedIndex> Iterator for Union<'_, Ty> {
    type Item = Range<Ty>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Ty: OrderedIndex> FusedIterator for Union<'_, Ty> {}

pub struct Intersection<'a, Ty: OrderedIndex> {
    pub(crate) iter: Iter<'a, Ty>,
    pub(crate) other: &'a InversionList<Ty>,
}

impl<Ty: OrderedIndex> Iterator for Intersection<'_, Ty> {
    type Item = Range<Ty>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.iter.next()?;
            if self.other.contains_range(next.clone()) {
                break Some(next);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<Ty: OrderedIndex> FusedIterator for Intersection<'_, Ty> {}
