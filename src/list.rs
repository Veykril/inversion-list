use alloc::vec;
use core::convert::identity;
use core::iter::FromIterator;
use core::ops::{Range, RangeBounds};
use core::{mem, ops};

use crate::util::bounds_to_range;
use crate::{InversionMap, OrderedIndex};

// mod iter;

/// An inversion list is a data structure that describes a set of non-overlapping numeric ranges, stored in increasing order.
///
/// A few notes regarding the naming convention of the functions:
/// - *_strict: These functions usual check that ranges are strictly the same, and not sub/supersets.
/// - *_at: These functions usually take indices into the backing buffer, while the other versions
///         generally take a value that is contained in a range or ranges directly.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InversionList<Idx: OrderedIndex = usize>(InversionMap<Idx, ()>);

impl<Idx: OrderedIndex> InversionList<Idx> {
    pub fn new() -> Self {
        InversionList(InversionMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        InversionList(InversionMap::with_capacity(capacity))
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Checks whether the given usize is inside any of the contained ranges.
    pub fn contains(&self, value: Idx) -> bool {
        self.0.contains(value)
    }

    /// Checks whether this InversionList contains a range that is a "superrange" of the given range.
    pub fn contains_range<R: RangeBounds<Idx>>(&self, range: R) -> bool {
        if let Some(Range { start, end }) = bounds_to_range(range) {
            self.0
                .binary_search(start)
                .map(|idx_s| end <= self.0.ranges[idx_s].range.end)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Checks whether this InversionList contains this exact range.
    pub fn contains_range_strict<R: RangeBounds<Idx>>(&self, range: R) -> bool {
        if let Some(Range { start, end }) = bounds_to_range(range) {
            self.0
                .binary_search(start)
                .map(|idx_s| end == self.0.ranges[idx_s].range.end)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Check if the given range intersects with any ranges inside of the inversion list.
    pub fn intersects<R: RangeBounds<Idx>>(&self, range: R) -> bool {
        self.intersects(range)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks whether `self` is a subset of `other`, meaning whether self's ranges all lie somewhere inside of `other`.
    pub fn is_subset(&self, other: &Self) -> bool {
        unimplemented!()
        // self.iter().all(|range| other.contains_range(range))
    }

    /// Checks whether `self` and `other` are entirely disjoint.
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Checks whether `self` is a subset of `other`, meaning whether self's ranges all lie somewhere inside of `other`.
    pub fn is_subset_strict(&self, other: &Self) -> bool {
        unimplemented!()
        // self.iter().all(|range| other.contains_range_strict(range))
    }

    /// Checks whether `self` is a strict superset of `other`, meaning whether other containts all of self's ranges.
    pub fn is_superset_strict(&self, other: &Self) -> bool {
        other.is_subset_strict(self)
    }

    /// Checks whether `self` and `other` are entirely disjoint.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        unimplemented!()
        // if self.len() <= other.len() {
        //     !self.iter().any(|range| other.intersects(range))
        // } else {
        //     !other.iter().any(|range| self.intersects(range))
        // }
    }

    /// Adds a unit range(index..index + 1) to the inversion list. This is faster than using
    /// [`add_range`] saving a second binary_search.
    ///
    /// If the unit is not part of an existing range, `true` is returned.
    ///
    /// If the unit already exists in a range, `false` is returned.
    ///
    /// # Panics
    ///
    /// Panics if index is equal to usize::MAX.
    pub fn add_unit(&mut self, index: Idx) -> bool {
        self.0.add_unit(index, ())
    }

    pub fn add_range<R: RangeBounds<Idx>>(&mut self, range: R) {
        self.0.add_range(range, |_| ());
    }

    pub fn remove_range<R: RangeBounds<Idx>>(&mut self, range: R) {
        self.0.remove_range(range);
    }

    /// Splits the range that contains `at` in two with `at` being the split point.
    ///
    /// If a range exists that contains `at` the return value are the indices of the
    /// new left and right ranges of the split point. The left range will contain `at`.
    /// If `at` is equal to the start of the range it is in, no split occurs and the left
    /// and right indices will be equal to the index of the range containing the value.
    ///
    /// Split ranges that are right next to each other will not be recognized as one.
    /// Meaning functions like `contains_range` will not work properly if the start and end
    /// points lie in the different parts of the neighbouring ranges. Thus it is important to
    /// either remove these ranges or remerge them.
    pub fn split(&mut self, at: Idx) {
        self.0.split(at);
    }

    /// Merges the ranges at `start` and `end`, discarding all ranges inbetween them.
    ///
    /// # Panics
    ///
    /// Panics if the indices dont point to a valid index into the vec.
    pub fn merge(&mut self, start: usize, end: usize) {
        unimplemented!()
    }

    /// Merges all ranges together that are directly adjacent to each other.
    pub fn collapse(&mut self) {
        unimplemented!()
    }

    /// Inverts all ranges, meaning existing ranges will be removed and parts that were previously
    /// not covered by ranges will now be covered.
    pub fn invert(&mut self) {
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn end(&self) -> Option<Idx> {
        self.0.end()
    }

    pub fn start(&self) -> Option<Idx> {
        self.0.start()
    }

    /// Returns the complete surrounding range, if any.
    pub fn span(&self) -> Option<Range<Idx>> {
        self.0.span()
    }
}

impl<Idx: OrderedIndex> FromIterator<Range<Idx>> for InversionList<Idx> {
    fn from_iter<T: IntoIterator<Item = Range<Idx>>>(iter: T) -> Self {
        let mut res = InversionList::new();
        for range in iter {
            res.add_range(range);
        }
        res
    }
}

impl<Idx: OrderedIndex> ops::BitAnd<&InversionList<Idx>> for &InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitand(self, rhs: &InversionList<Idx>) -> Self::Output {
        unimplemented!()
        // let mut res = InversionList::new();

        // let (base, iter) = if self.len() < rhs.len() {
        //     (rhs, self.iter())
        // } else {
        //     (self, rhs.iter())
        // };

        // for range in iter {
        //     let start = base.0.binary_search(range.start).unwrap_or_else(identity);
        //     let end = base
        //         .0
        //         .binary_search(range.end)
        //         .unwrap_or_else(|idx| idx - 1 /*can this ever underflow?*/);
        //     debug_assert!(start <= end);
        //     res.add_range(range.start.max(base[start].start)..range.end.min(base[start].end));
        //     for range in base.get((start + 1)..end).into_iter().flatten() {
        //         // could just copy slices here for efficiency
        //         res.add_range(range.clone());
        //     }
        //     res.add_range(range.start.max(base[end].start)..range.end.min(base[end].end));
        // }

        // res
    }
}

impl<Idx: OrderedIndex> ops::BitAnd<InversionList<Idx>> for &InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitand(self, rhs: InversionList<Idx>) -> Self::Output {
        <&InversionList<Idx>>::bitand(self, &rhs)
    }
}

impl<Idx: OrderedIndex> ops::BitAnd<&InversionList<Idx>> for InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitand(self, rhs: &InversionList<Idx>) -> Self::Output {
        <&InversionList<Idx>>::bitand(&self, rhs)
    }
}

impl<Idx: OrderedIndex> ops::BitAnd<InversionList<Idx>> for InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitand(self, rhs: InversionList<Idx>) -> Self::Output {
        <&InversionList<Idx>>::bitand(&self, &rhs)
    }
}

impl<Idx: OrderedIndex> ops::BitAndAssign<InversionList<Idx>> for InversionList<Idx> {
    fn bitand_assign(&mut self, rhs: InversionList<Idx>) {
        *self &= &rhs;
    }
}

impl<Idx: OrderedIndex> ops::BitAndAssign<&InversionList<Idx>> for InversionList<Idx> {
    fn bitand_assign(&mut self, rhs: &InversionList<Idx>) {
        *self = &*self & rhs;
    }
}

impl<Idx: OrderedIndex> ops::BitOr<&InversionList<Idx>> for &InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitor(self, rhs: &InversionList<Idx>) -> Self::Output {
        unimplemented!()
        // let (mut res, iter) = if self.len() < rhs.len() {
        //     (rhs.clone(), self.iter())
        // } else {
        //     (self.clone(), rhs.iter())
        // };

        // for range in iter {
        //     res.add_range(range);
        // }

        // res
    }
}

impl<Idx: OrderedIndex> ops::BitOr<InversionList<Idx>> for &InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitor(self, rhs: InversionList<Idx>) -> Self::Output {
        <&InversionList<Idx>>::bitor(self, &rhs)
    }
}

impl<Idx: OrderedIndex> ops::BitOr<&InversionList<Idx>> for InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitor(self, rhs: &InversionList<Idx>) -> Self::Output {
        <&InversionList<Idx>>::bitor(&self, rhs)
    }
}

impl<Idx: OrderedIndex> ops::BitOr<InversionList<Idx>> for InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn bitor(self, rhs: InversionList<Idx>) -> Self::Output {
        <&InversionList<Idx>>::bitor(&self, &rhs)
    }
}

impl<Idx: OrderedIndex> ops::BitOrAssign<InversionList<Idx>> for InversionList<Idx> {
    fn bitor_assign(&mut self, rhs: InversionList<Idx>) {
        *self |= &rhs;
    }
}

impl<Idx: OrderedIndex> ops::BitOrAssign<&InversionList<Idx>> for InversionList<Idx> {
    fn bitor_assign(&mut self, rhs: &InversionList<Idx>) {
        *self = &*self | rhs;
    }
}

impl<Idx: OrderedIndex> ops::Not for InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn not(self) -> InversionList<Idx> {
        !&self
    }
}

impl<Idx: OrderedIndex> ops::Not for &InversionList<Idx> {
    type Output = InversionList<Idx>;
    fn not(self) -> InversionList<Idx> {
        unimplemented!()
        // let mut res = InversionList::new();
        // let mut iter = self.iter();
        // if let Some(range) = iter.next() {
        //     let mut last = if range.start == Idx::min_value() {
        //         range.end
        //     } else {
        //         res.add_range(Idx::min_value()..range.start);
        //         range.end
        //     };
        //     for range in iter {
        //         res.add_range(last..range.start);
        //         last = range.end
        //     }
        //     res.add_range(last..Idx::max_value());
        // }
        // res
    }
}
