use core::cmp::Ordering;
use core::iter::once;
use core::ops::Not;
use core::ops::{Range, RangeBounds};
use core::{mem, ops};

use alloc::vec::Vec;

use crate::util::bounds_to_range;
use crate::util::variance::CovariantLifetime;
use crate::OrderedIndex;

use Err as Insert;
use Ok as Within;

#[cfg(test)]
mod test;

mod iter;
pub use self::iter::{IntoIter, Iter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry<Idx, V> {
    pub(crate) range: Range<Idx>,
    pub(crate) value: V,
}

impl<Idx, V> From<Entry<Idx, V>> for (Range<Idx>, V) {
    fn from(Entry { range, value }: Entry<Idx, V>) -> Self {
        (range, value)
    }
}

// These are public APIs that abstract away the internal representation of the inversion map.

pub struct Entries<'im, Idx, V> {
    it: Vec<Entry<Idx, V>>,
    covariant: CovariantLifetime<'im>,
}

impl<'im, Idx, V> Entries<'im, Idx, V> {
    pub fn is_empty(&self) -> bool {
        self.it.is_empty()
    }

    pub fn len(&self) -> usize {
        self.it.len()
    }
}

pub struct EntriesRef<'im, Idx, V> {
    slice: &'im [Entry<Idx, V>],
}

impl<'im, Idx: OrderedIndex, V> EntriesRef<'im, Idx, V> {
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Range<Idx>, &V)> + '_ {
        self.slice
            .iter()
            .map(|Entry { range, value }| (range.clone(), value))
    }
}

pub struct EntriesMut<'im, Idx, V> {
    it: &'im mut [Entry<Idx, V>],
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InversionMap<Idx, V> {
    // FIXME: use MaybeUninit so we can prevent some frequent clones
    pub(crate) ranges: Vec<Entry<Idx, V>>,
}

impl<Idx, V> InversionMap<Idx, V> {
    pub fn new() -> Self {
        InversionMap { ranges: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        InversionMap {
            ranges: Vec::with_capacity(capacity),
        }
    }
}

// region: delegate methods
impl<Idx: OrderedIndex, V> InversionMap<Idx, V> {
    #[inline]
    pub fn capacity(&self) -> usize {
        self.ranges.capacity()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.ranges.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    #[inline]
    pub fn start(&self) -> Option<Idx> {
        self.ranges.first().map(|r| r.range.start)
    }

    #[inline]
    pub fn end(&self) -> Option<Idx> {
        self.ranges.last().map(|r| r.range.end)
    }

    #[inline]
    pub fn first(&self) -> Option<(Range<Idx>, &V)> {
        self.ranges.first().map(|r| (r.range.clone(), &r.value))
    }

    #[inline]
    pub fn last(&self) -> Option<(Range<Idx>, &V)> {
        self.ranges.last().map(|r| (r.range.clone(), &r.value))
    }
}
// endregion

impl<Idx: OrderedIndex, V> InversionMap<Idx, V> {
    /// Checks whether the given index is contained in the map.
    pub fn contains(&self, index: Idx) -> bool {
        self.binary_search(index).is_ok()
    }

    /// Looks up the range and value associated with the given index.
    pub fn lookup(&self, index: Idx) -> Option<(Range<Idx>, &V)> {
        self.binary_search(index)
            .map(|idx| (self.ranges[idx].range.clone(), &self.ranges[idx].value))
            .ok()
    }

    /// Looks up all entries whose ranges overlap with the given range.
    pub fn lookup_range<R: RangeBounds<Idx>>(&self, range: R) -> Option<EntriesRef<'_, Idx, V>> {
        let range = bounds_to_range(range)?;
        let slice = match self.range_binary_search(range) {
            (Within(s), Within(e)) => &self.ranges[s..=e],
            (Insert(s) | Within(s), Insert(e)) => &self.ranges[s..e],
            (Insert(s), Within(e)) => &self.ranges[s + 1..=e],
        };
        slice.is_empty().not().then(|| EntriesRef { slice })
    }

    /// Check if the given range intersects with any ranges inside of the inversion list.
    pub fn intersects<R: RangeBounds<Idx>>(&self, range: R) -> bool {
        let Some(range) = bounds_to_range(range) else {
            // empty range can't intersect
            return false;
        };
        match self.range_binary_search(range) {
            // check if there is at least one range inside of our range
            (Insert(idx_s), Insert(idx_e)) => idx_e - idx_s > 0,
            _ => true,
        }
    }

    /// Returns the complete surrounding range, if any.
    pub fn span(&self) -> Option<Range<Idx>> {
        let start = self.start()?;
        let end = self.end()?;
        Some(start..end)
    }
}

impl<Idx: OrderedIndex, V: Clone> InversionMap<Idx, V> {
    pub fn insert_unit(&mut self, index: Idx, value: V) -> bool {
        match self.binary_search(index) {
            Insert(insert_idx) => {
                self.ranges.insert(
                    insert_idx,
                    Entry {
                        range: index
                            ..index
                                .checked_add(Idx::one())
                                .expect("index is equal to usize::MAX"),
                        value,
                    },
                );
                true
            }
            Within(idx) => {
                let (_, r) = self.split_impl(idx, index, |_, v| (v.clone(), v));
                self.split_impl(r, index, |_, v| (value, v));
                false
            }
        }
    }

    /// Inserts a new range with a given value into the map overwriting any ranges that are contained within.
    /// Ranges that partially overlap will be shortened or split accordingly.
    pub fn insert_range<R: RangeBounds<Idx>>(&mut self, range: R, value: V) {
        self.insert_range_with(range, |_| value.clone());
    }

    /// Inserts a new range with a value produced by `value` into the map. `value` gets passed all
    /// overlapping entries. If start or end overlap with a range, the overlapping range will be
    /// split accordingly.
    pub fn insert_range_with<'im, R: RangeBounds<Idx>>(
        &'im mut self,
        range: R,
        value: impl FnOnce(EntriesRef<'_, Idx, V>) -> V,
    ) {
        let Some(range) = bounds_to_range(range) else {
            return;
        };
        match self.range_binary_search(range.clone()) {
            #[cfg(debug_assertions)]
            (Within(idx_s), Insert(idx_e)) if idx_s == idx_e => {
                unreachable!("range was empty and should've been filtered out")
            }
            (Within(idx_s), Insert(idx_e)) => {
                let slice = &self.ranges[idx_s..idx_e];
                let value = value(EntriesRef { slice });
                match slice {
                    [] => self.ranges.insert(idx_s, Entry { range, value }),
                    [_] => {
                        self.ranges[idx_s].range.end = range.start;
                        self.ranges.insert(idx_e, Entry { range, value });
                    }
                    [_, .., _] => {
                        self.ranges[idx_s].range.end = range.start;
                        self.ranges
                            .splice(idx_s + 1..idx_e, once(Entry { range, value }));
                    }
                }
            }
            (Insert(idx_s), Insert(idx_e)) => {
                let slice = &self.ranges[idx_s..idx_e];
                let value = value(EntriesRef { slice });
                match slice {
                    [] => self.ranges.insert(idx_s, Entry { range, value }),
                    [..] => {
                        self.ranges
                            .splice(idx_s..idx_e, once(Entry { range, value }));
                    }
                }
            }
            (Insert(idx_s), Within(idx_e)) => {
                let slice = &self.ranges[idx_s..=idx_e];
                let value = value(EntriesRef { slice });
                match slice {
                    [] => unreachable!(),
                    [.., _] => {
                        self.ranges[idx_e].range.start = range.end;
                        self.ranges
                            .splice(idx_s..idx_e, once(Entry { range, value }));
                    }
                }
            }
            (Within(idx_s), Within(idx_e)) => {
                let slice = &self.ranges[idx_s..=idx_e];
                let value = value(EntriesRef { slice });
                match slice {
                    [] => unreachable!(),
                    [entry] => {
                        let end_val_clone = entry.value.clone();
                        let end_end = mem::replace(&mut self.ranges[idx_s].range.end, range.start);
                        self.ranges.insert(
                            idx_s + 1,
                            Entry {
                                range: range.end..end_end,
                                value: end_val_clone,
                            },
                        );
                        self.ranges.insert(idx_s + 1, Entry { range, value });
                    }
                    [_, .., _] => {
                        self.ranges[idx_s].range.end = range.start;
                        self.ranges[idx_e].range.start = range.end;
                        self.ranges
                            .splice(idx_s + 1..idx_e, once(Entry { range, value }));
                    }
                }
            }
        };
    }
}

impl<Idx: OrderedIndex, V> InversionMap<Idx, V> {
    /// Adds a unit range(index..index + 1) to the inversion list. This is faster than using
    /// [`Self::add_range`] saving a second binary_search.
    ///
    /// If the unit is not part of an existing range, `true` is returned.
    ///
    /// If the unit already exists in a range, `false` is returned and the range value will be set
    /// to `value`.
    ///
    /// # Panics
    ///
    /// Panics if index is equal to [`OrderedIndex::max_value()`].
    pub fn add_unit(&mut self, index: Idx, value: V) -> bool {
        match self.binary_search(index) {
            Insert(insert_idx) => {
                self.ranges.insert(
                    insert_idx,
                    Entry {
                        range: index
                            ..index
                                .checked_add(Idx::one())
                                .expect("index is equal to usize::MAX"),
                        value,
                    },
                );
                true
            }
            Within(idx) => {
                self.ranges[idx].value = value;
                false
            }
        }
    }

    pub fn add_range<R: RangeBounds<Idx>>(&mut self, range: R, value: V) {
        self.add_range_with(range, |_| value);
    }

    /// Adds a new range with a value produced by `value` into the map. `value` gets passed all
    /// overlapping entries. If start or end overlap with a range, the overlapping range will be
    /// extended accordingly.
    pub fn add_range_with<R: RangeBounds<Idx>>(
        &mut self,
        range: R,
        value: impl FnOnce(EntriesRef<'_, Idx, V>) -> V,
    ) {
        let Some(range) = bounds_to_range(range) else {
            return;
        };

        match self.range_binary_search(range.clone()) {
            #[cfg(debug_assertions)]
            (Within(idx_s), Insert(idx_e)) if idx_s == idx_e => {
                unreachable!("range was empty and should've been filtered out")
            }
            (Within(idx_s) | Insert(idx_s), Insert(idx_e)) => {
                let slice = &self.ranges[idx_s..idx_e];
                let value = value(EntriesRef { slice });
                match slice {
                    // Same indices, surround nothing so insert
                    [] => self.ranges.insert(idx_s, Entry { range, value }),
                    // Surrounding a single element, so replace it
                    [it] => {
                        let start = it.range.start.min(range.start);
                        let end = it.range.end.max(range.end);
                        self.ranges[idx_s] = Entry {
                            range: start..end,
                            value,
                        };
                    }
                    // Surrounding multiple elements, merge them and replace
                    [start, .., end] => {
                        let start = start.range.start.min(range.start);
                        let end = end.range.end.max(range.end);
                        self.ranges.splice(
                            idx_s..idx_e,
                            once(Entry {
                                range: start..end,
                                value,
                            }),
                        );
                    }
                }
            }
            (Within(idx_s), Within(idx_e)) => {
                let slice = &self.ranges[idx_s..=idx_e];
                let value = value(EntriesRef { slice });

                match slice {
                    [] => unreachable!(),
                    [_] => {
                        self.ranges[idx_s].value = value;
                    }
                    [start, .., end] => {
                        let mut entry = Entry { range, value };
                        entry.range.start = start.range.start.min(entry.range.start);
                        entry.range.end = end.range.end.max(entry.range.end);
                        self.ranges.splice(idx_s..=idx_e, once(entry));
                    }
                }
            }
            (Insert(idx_s), Within(idx_e)) => {
                let slice = &self.ranges[idx_s..=idx_e];
                let value = value(EntriesRef { slice });
                match slice {
                    [] => unreachable!(),
                    [_] => {
                        self.ranges[idx_e].range.start = range.start.clone();
                        self.ranges[idx_s].value = value;
                    }
                    [start, .., end] => {
                        let mut entry = Entry { range, value };
                        entry.range.start = start.range.start.min(entry.range.start);
                        entry.range.end = end.range.end.max(entry.range.end);
                        self.ranges.splice(idx_s..=idx_e, once(entry));
                    }
                }
            }
        };
    }
}

impl<Idx: OrderedIndex, V: Clone> InversionMap<Idx, V> {
    /// Removes the range of values overlapping the given range.
    pub fn remove_range<R: RangeBounds<Idx>>(
        &mut self,
        range: R,
        split_boundary_left: impl FnOnce(Range<Idx>, &V) -> V,
        split_boundary_right: impl FnOnce(Range<Idx>, &V) -> V,
    ) {
        let Some(range) = bounds_to_range(range) else {
            return;
        };
        match self.range_binary_search(range.clone()) {
            #[cfg(debug_assertions)]
            (Within(idx_s), Insert(idx_e)) if idx_s == idx_e => {
                unreachable!("range was empty and should've been filtered out")
            }
            (Insert(idx_s), Insert(idx_e)) => {
                _ = self.ranges.drain(idx_s..idx_e);
            }
            (Within(idx_s), Within(idx_e)) => {
                let slice = &self.ranges[idx_s..=idx_e];
                match slice {
                    [] => unreachable!(),
                    [entry] => {
                        let left_range = entry.range.start..range.start;
                        let left = left_range.is_empty().not().then(|| Entry {
                            range: left_range,
                            value: split_boundary_left(entry.range.clone(), &entry.value),
                        });
                        let right_range = range.end..entry.range.end;
                        let right = right_range.is_empty().not().then(|| Entry {
                            range: right_range,
                            value: split_boundary_right(entry.range.clone(), &entry.value),
                        });
                        self.ranges
                            .splice(idx_s..=idx_e, [left, right].into_iter().flatten());
                    }
                    [start, .., end] => {
                        let left_range = start.range.start..range.start;
                        let left = left_range.is_empty().not().then(|| Entry {
                            range: left_range,
                            value: split_boundary_left(start.range.clone(), &start.value),
                        });
                        let right_range = range.end..end.range.end;
                        let right = right_range.is_empty().not().then(|| Entry {
                            range: right_range,
                            value: split_boundary_right(end.range.clone(), &end.value),
                        });
                        self.ranges
                            .splice(idx_s..=idx_e, [left, right].into_iter().flatten());
                    }
                }
            }
            (Insert(idx_s), Within(idx_e)) => {
                let slice = &self.ranges[idx_s..=idx_e];
                match slice {
                    [] => unreachable!(),
                    [.., end] if end.range.end != range.end => {
                        let right = split_boundary_right(end.range.clone(), &end.value);
                        self.ranges[idx_e].range.start = range.end;
                        self.ranges[idx_e].value = right;
                        self.ranges.drain(idx_s..idx_e);
                    }
                    [..] => _ = self.ranges.drain(idx_s..=idx_e),
                }
            }
            (Within(idx_s), Insert(idx_e)) => {
                let slice = &self.ranges[idx_s..idx_e];
                match slice {
                    [] => unreachable!(),
                    [start, ..] if start.range.start != range.start => {
                        let left = split_boundary_left(start.range.clone(), &start.value);
                        self.ranges[idx_s].range.end = range.start;
                        self.ranges[idx_s].value = left;
                        self.ranges.drain(idx_s + 1..idx_e);
                    }
                    [..] => _ = self.ranges.drain(idx_s..idx_e),
                }
            }
        };
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
    pub fn split(&mut self, at: Idx) -> Option<(usize, usize)> {
        self.binary_search(at)
            .ok()
            .map(|idx| self.split_impl(idx, at, |_, v| (v.clone(), v)))
    }

    /// Like [`split`] but allows for the split to be done with a custom function.
    pub fn split_with(
        &mut self,
        at: Idx,
        splitter: impl FnOnce(Range<Idx>, V) -> (V, V),
    ) -> Option<(usize, usize)> {
        self.binary_search(at)
            .ok()
            .map(|idx| self.split_impl(idx, at, splitter))
    }

    // invariant, `at` is inside the range addressed by idx
    // return value is left range and right range indices of the split range.
    // The indices are the same if the split point was at the start of the range.
    fn split_impl(
        &mut self,
        idx: usize,
        at: Idx,
        splitter: impl FnOnce(Range<Idx>, V) -> (V, V),
    ) -> (usize, usize) {
        debug_assert!(self.ranges[idx].range.contains(&at));
        let to_split = &mut self.ranges[idx];
        if to_split.range.start != at {
            let end = mem::replace(&mut to_split.range.end, at);
            // FIXME: The clone should not be necessary here
            let value = to_split.value.clone();
            let (left, right) = splitter(to_split.range.clone(), value);
            to_split.value = left;
            self.ranges.insert(
                idx + 1,
                Entry {
                    range: at..end,
                    value: right,
                },
            );
            (idx, idx + 1)
        } else {
            (idx, idx)
        }
    }
}

impl<Idx: OrderedIndex, V> InversionMap<Idx, V> {
    fn bin_search_ordering(left: Ordering, right: Ordering) -> Ordering {
        use Ordering::*;
        match (left, right) {
            // start > key
            (Greater, _) => Greater,
            // start == key
            (Equal, _) => Equal,
            // start < key && key < end
            (Less, Less) => Equal,
            // start < key && key >= end
            (Less, _) => Less,
        }
    }

    pub(crate) fn binary_search(&self, key: Idx) -> Result<usize, usize> {
        self.ranges.binary_search_by(move |Entry { range, .. }| {
            Self::bin_search_ordering(range.start.cmp(&key), key.cmp(&range.end))
        })
    }

    pub(crate) fn range_binary_search(
        &self,
        Range { start, end }: Range<Idx>,
    ) -> (Result<usize, usize>, Result<usize, usize>) {
        let start @ (Within(idx) | Insert(idx)) = self.binary_search(start);
        let Some(end) = end.checked_sub(Idx::one()) else {
            debug_assert!(false);
            return (Insert(0), Insert(0));
        };
        let end = self.ranges[idx..].binary_search_by(move |Entry { range, .. }| {
            Self::bin_search_ordering(range.start.cmp(&end), end.cmp(&range.end))
        });

        (
            start,
            match end {
                Within(e) => Within(idx + e),
                Insert(e) => Insert(idx + e),
            },
        )
    }
}

impl<Idx: OrderedIndex, V> ops::BitAnd<&InversionMap<Idx, V>> for &InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitAnd<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitand(self, rhs: &InversionMap<Idx, V>) -> Self::Output {
        let mut res = InversionMap::new();

        let (base, iter) = if self.len() < rhs.len() {
            (rhs, self.iter())
        } else {
            (self, rhs.iter())
        };

        for (range, value) in iter {
            let (start, end) = base.range_binary_search(range.clone());
            let start = start.unwrap_or_else(core::convert::identity);
            let end = end.unwrap_or_else(core::convert::identity);
            debug_assert!(start <= end);
            let base_entry = &base.ranges[start];
            res.add_range(
                range.start.max(base_entry.range.start)..range.end.min(base_entry.range.end),
                (&base_entry.value) & value,
            );
            for entry in base.ranges.get((start + 1)..end).into_iter().flatten() {
                // could just copy slices here for efficiency
                res.add_range(entry.range.clone(), (&entry.value) & value);
            }
            let base_entry = &base.ranges[end];
            res.add_range(
                range.start.max(base_entry.range.start)..range.end.min(base_entry.range.end),
                (&base_entry.value) & value,
            );
        }

        res
    }
}

impl<Idx: OrderedIndex, V> ops::BitAnd<InversionMap<Idx, V>> for &InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitAnd<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitand(self, rhs: InversionMap<Idx, V>) -> Self::Output {
        <&InversionMap<Idx, V>>::bitand(self, &rhs)
    }
}

impl<Idx: OrderedIndex, V> ops::BitAnd<&InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitAnd<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitand(self, rhs: &InversionMap<Idx, V>) -> Self::Output {
        <&InversionMap<Idx, V>>::bitand(&self, rhs)
    }
}

impl<Idx: OrderedIndex, V> ops::BitAnd<InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitAnd<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitand(self, rhs: InversionMap<Idx, V>) -> Self::Output {
        <&InversionMap<Idx, V>>::bitand(&self, &rhs)
    }
}

impl<Idx: OrderedIndex, V> ops::BitAndAssign<InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitAnd<Output = V>,
{
    fn bitand_assign(&mut self, rhs: InversionMap<Idx, V>) {
        *self &= &rhs;
    }
}

impl<Idx: OrderedIndex, V> ops::BitAndAssign<&InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitAnd<Output = V>,
{
    fn bitand_assign(&mut self, rhs: &InversionMap<Idx, V>) {
        *self = &*self & rhs;
    }
}

impl<Idx: OrderedIndex, V: Clone> ops::BitOr<&InversionMap<Idx, V>> for &InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitOr<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitor(self, rhs: &InversionMap<Idx, V>) -> Self::Output {
        let (mut res, iter) = if self.len() < rhs.len() {
            (rhs.clone(), self.iter())
        } else {
            (self.clone(), rhs.iter())
        };

        for (range, v) in iter {
            res.add_range_with(range, |entries| {
                entries.iter().fold(v.clone(), |acc, (_, v)| &acc | v)
            });
        }

        res
    }
}

impl<Idx: OrderedIndex, V: Clone> ops::BitOr<InversionMap<Idx, V>> for &InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitOr<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitor(self, rhs: InversionMap<Idx, V>) -> Self::Output {
        <&InversionMap<Idx, V>>::bitor(self, &rhs)
    }
}

impl<Idx: OrderedIndex, V: Clone> ops::BitOr<&InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitOr<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitor(self, rhs: &InversionMap<Idx, V>) -> Self::Output {
        <&InversionMap<Idx, V>>::bitor(&self, rhs)
    }
}

impl<Idx: OrderedIndex, V: Clone> ops::BitOr<InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitOr<Output = V>,
{
    type Output = InversionMap<Idx, V>;
    fn bitor(self, rhs: InversionMap<Idx, V>) -> Self::Output {
        <&InversionMap<Idx, V>>::bitor(&self, &rhs)
    }
}

impl<Idx: OrderedIndex, V: Clone> ops::BitOrAssign<InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitOr<Output = V>,
{
    fn bitor_assign(&mut self, rhs: InversionMap<Idx, V>) {
        *self |= &rhs;
    }
}

impl<Idx: OrderedIndex, V: Clone> ops::BitOrAssign<&InversionMap<Idx, V>> for InversionMap<Idx, V>
where
    for<'a> &'a V: ops::BitOr<Output = V>,
{
    fn bitor_assign(&mut self, rhs: &InversionMap<Idx, V>) {
        *self = &*self | rhs;
    }
}
