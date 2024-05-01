use core::iter::FromIterator;
use core::ops::{Range, RangeBounds};
use core::{mem, ops};

use alloc::vec::Vec;

use crate::map::{EntriesRef, Entry};
use crate::{InversionMap, OrderedIndex};

mod iter;

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

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

    pub fn first(&self) -> Option<Range<Idx>> {
        self.0.first().map(|(range, _)| range)
    }

    pub fn last(&self) -> Option<Range<Idx>> {
        self.0.last().map(|(range, _)| range)
    }

    /// Checks whether the given usize is inside any of the contained ranges.
    pub fn contains(&self, value: Idx) -> bool {
        self.0.contains(value)
    }

    /// Checks whether this InversionList contains a range that is a "superrange" of the given range.
    pub fn contains_range<R: RangeBounds<Idx>>(&self, range: R) -> bool {
        self.lookup_range(range).map_or(false, |it| !it.is_empty())
    }

    /// Looks up the range the given index is part of if it is contained within the list.
    pub fn lookup(&self, index: Idx) -> Option<Range<Idx>> {
        self.0.lookup(index).map(|(range, _)| range)
    }

    /// Looks up all entries whose ranges overlap with the given range.
    pub fn lookup_range<R: RangeBounds<Idx>>(&self, range: R) -> Option<EntriesRef<'_, Idx, ()>> {
        self.0.lookup_range(range)
    }

    /// Check if the given range intersects with any ranges inside of the inversion list.
    pub fn intersects<R: RangeBounds<Idx>>(&self, range: R) -> bool {
        self.0.intersects(range)
    }
    /// Checks whether `self` is a subset of `other`, meaning whether self's ranges all lie somewhere inside of `other`.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.iter().all(|range| other.contains_range(range))
    }

    /// Checks whether `self` and `other` are entirely disjoint.
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Checks whether `self` and `other` are entirely disjoint.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        if self.len() <= other.len() {
            !self.iter().any(|range| other.intersects(range))
        } else {
            !other.iter().any(|range| self.intersects(range))
        }
    }

    /// Adds a unit range(index..index + 1) to the inversion list. This is faster than using
    /// [`insert_range`] saving a second binary_search.
    ///
    /// If the unit is not part of an existing range, `true` is returned.
    ///
    /// If the unit already exists in a range, `false` is returned.
    ///
    /// # Panics
    ///
    /// Panics if index is equal to usize::MAX.
    pub fn insert_unit(&mut self, index: Idx) -> bool {
        self.0.insert_unit(index, ())
    }

    pub fn insert_range<R: RangeBounds<Idx>>(&mut self, range: R) {
        self.0.insert_range_with(range, |_| ());
    }

    /// This is the same as [`insert_unit`] with the exception that this function splits apart the
    /// range this is being inserted into if there is already a range covering this offset.
    pub fn add_unit(&mut self, index: Idx) -> bool {
        self.0.add_unit(index, ())
    }

    /// This is the same as [`insert_range`] with the exception that this function splits apart the
    /// range this is being inserted into if there is already a range covering this range.
    pub fn add_range<R: RangeBounds<Idx>>(&mut self, range: R) {
        self.0.add_range_with(range, |_| ());
    }

    pub fn remove_range<R: RangeBounds<Idx>>(&mut self, range: R) {
        self.0.remove_range(range, |_, _| (), |_, _| ());
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
        self.0.ranges[start].range.end = self.0.ranges[end].range.end;
        self.0.ranges.drain(start + 1..=end);
    }

    /// Merges all ranges together that are directly adjacent to each other.
    pub fn collapse(&mut self) {
        let ranges = &mut self.0.ranges;
        let mut i = 1;
        while i < ranges.len() {
            if ranges[i - 1].range.end == ranges[i].range.start {
                ranges[i - 1].range.end = ranges[i].range.end;
                ranges.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Inverts all ranges, meaning existing ranges will be removed and parts that were previously
    /// not covered by ranges will now be covered.
    pub fn invert(&mut self) {
        let prev_len = self.0.len();
        let mut old = mem::replace(&mut self.0.ranges, Vec::with_capacity(prev_len)).into_iter();

        let mut last = match old.next() {
            Some(range) if range.range.start == Idx::min_value() => range.range.end,
            Some(range) => {
                self.0.ranges.push(Entry {
                    range: Idx::min_value()..range.range.start,
                    value: (),
                });
                range.range.end
            }
            None => return,
        };
        for range in old {
            if range.range.start != last {
                self.0.ranges.push(Entry {
                    range: last..range.range.start,
                    value: (),
                });
            }
            last = range.range.end;
        }
    }
}

impl<Idx: OrderedIndex> FromIterator<Range<Idx>> for InversionList<Idx> {
    fn from_iter<T: IntoIterator<Item = Range<Idx>>>(iter: T) -> Self {
        let mut res = InversionList::new();
        for range in iter {
            res.insert_range(range);
        }
        res
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
        let mut ranges = Vec::with_capacity(self.capacity());
        let mut iter = self.iter();
        let Some(range) = iter.next() else {
            return InversionList(InversionMap {
                ranges: alloc::vec![Entry {
                    range: Idx::min_value()..Idx::max_value(),
                    value: ()
                }],
            });
        };
        let mut last = if range.start == Idx::min_value() {
            if range.end == Idx::max_value() {
                return InversionList::new();
            }
            range.end
        } else {
            ranges.push(Entry {
                range: Idx::min_value()..range.start,
                value: (),
            });
            range.end
        };
        for range in iter {
            ranges.push(Entry {
                range: last..range.start,
                value: (),
            });
            last = range.end
        }
        ranges.push(Entry {
            range: last..Idx::max_value(),
            value: (),
        });
        InversionList(InversionMap { ranges })
    }
}
