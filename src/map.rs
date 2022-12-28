use core::cmp::Ordering;
use core::marker::PhantomData;
use core::ops::Not;
use core::ops::{Range, RangeBounds};
use core::{iter, mem};

use alloc::{vec, vec::Vec};

use crate::util::bounds_to_range;
use crate::OrderedIndex;

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

pub struct Entries<'im, Idx, V> {
    it: Vec<Entry<Idx, V>>,
    covariant: PhantomData<&'im mut (Idx, V)>,
}

pub struct EntriesRef<'im, Idx, V> {
    it: &'im [Entry<Idx, V>],
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

impl<Idx: OrderedIndex, V> InversionMap<Idx, V> {
    pub fn capacity(&self) -> usize {
        self.ranges.capacity()
    }

    /// Checks whether the given index is contained in the map.
    pub fn contains(&self, index: Idx) -> bool {
        self.binary_search(index).is_ok()
    }

    /// Looks up the range and value associated with the given index.
    pub fn lookup(&self, index: Idx) -> Option<(&Range<Idx>, &V)> {
        self.binary_search(index)
            .map(|idx| (&self.ranges[idx].range, &self.ranges[idx].value))
            .ok()
    }

    /// Looks up all entries whose ranges overlap with the given range.
    pub fn lookup_range<R: RangeBounds<Idx>>(&self, range: R) -> Option<EntriesRef<'_, Idx, V>> {
        let Range { start, end } = bounds_to_range(range)?;
        let slice = match self.d_binary_search(start, end) {
            (Ok(s), Ok(e)) => &self.ranges[s..=e],
            (Ok(s), Err(e)) => &self.ranges[s..e],
            (Err(s), Ok(e)) => &self.ranges[s + 1..=e],
            (Err(s), Err(e)) => &self.ranges[s + 1..e],
        };
        slice.is_empty().not().then(|| EntriesRef { it: slice })
    }

    /// Check if the given range intersects with any ranges inside of the inversion list.
    pub fn intersects<R: RangeBounds<Idx>>(&self, range: R) -> bool {
        match bounds_to_range(range) {
            Some(Range { start, end }) => {
                match self.binary_search(start) {
                    Ok(_) => true,
                    Err(idx_s) => {
                        match end.checked_sub(Idx::one()) {
                            Some(end) => match self.binary_search(end) {
                                Ok(_) => true,
                                // check if there is at least one range inside of our range
                                Err(idx_e) => idx_e - idx_s > 1,
                            },
                            None => false,
                        }
                    }
                }
            }
            None => false,
        }
    }

    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn end(&self) -> Option<Idx> {
        self.ranges.last().map(|r| r.range.end)
    }

    pub fn start(&self) -> Option<Idx> {
        self.ranges.first().map(|r| r.range.start)
    }

    /// Returns the complete surrounding range, if any.
    pub fn span(&self) -> Option<Range<Idx>> {
        self.start()
            .and_then(|start| self.end().map(move |end| start..end))
    }
}

impl<Idx: OrderedIndex, V: Clone> InversionMap<Idx, V> {
    /// Inserts a new range with a given value into the map overwriting any ranges that are contained within.
    /// Ranges that partially overlap will be shortened or split accordingly.
    pub fn insert_range<R: RangeBounds<Idx>>(&mut self, range: R, value: V) {
        let Some(range @ Range { start, end}) =  bounds_to_range(range) else { return };
        let entry = Entry { range, value };
        // x = free space
        // 0 = occupied space by a range
        match self.d_binary_search(start, end) {
            // ..xxx000x..xxx..
            //    ^________^
            (Err(idx_s), Err(idx_e)) => {
                self.ranges.splice(idx_s..idx_e, iter::once(entry));
            }
            // ..xx000000xx..
            //      ^__^
            (Ok(idx_s), Ok(idx_e)) if idx_s == idx_e => {
                let end_segment_range_end = mem::replace(&mut self.ranges[idx_s].range.end, start);
                let split_value = self.ranges[idx_s].value.clone();
                // insert the split off tail
                self.ranges.insert(
                    idx_s + 1,
                    Entry {
                        range: Range {
                            start: end,
                            end: end_segment_range_end,
                        },
                        value: split_value,
                    },
                );
                // insert our range
                self.ranges.insert(idx_s + 1, entry);
            }
            // ..xx000x..x000xx..
            //      ^______^
            (Ok(idx_s), Ok(idx_e)) => {
                self.ranges[idx_s].range.end = start;
                self.ranges[idx_e].range.start = end;
                self.ranges.splice(idx_s + 1..idx_e, iter::once(entry));
            }
            // ..xx000x..00xxx..
            //      ^_______^
            (Ok(idx_s), Err(idx_e)) => {
                self.ranges[idx_s].range.end = start;
                self.ranges.splice(idx_s + 1..idx_e, iter::once(entry));
            }
            // ..xxx000x..x0x..
            //    ^________^
            (Err(idx_s), Ok(idx_e)) => {
                self.ranges[idx_e].range.start = end;
                self.ranges.splice(idx_s..idx_e, iter::once(entry));
            }
        };
    }

    pub fn insert_range_with<'im, R: RangeBounds<Idx>>(
        &'im mut self,
        range: R,
        value: impl FnOnce(Entries<'im, Idx, V>) -> V,
    ) {
        let Some(range @ Range { start, end}) = bounds_to_range(range) else { return };
        // x = free space
        // 0 = occupied space by a range
        match self.d_binary_search(start, end) {
            // ..xxx000x..xxx..
            //    ^________^
            (Err(idx_s), Err(idx_e)) => {
                let it = self.ranges.drain(idx_s..idx_e).collect();
                self.ranges.insert(
                    idx_s,
                    Entry {
                        range,
                        value: value(Entries {
                            it,
                            covariant: PhantomData,
                        }),
                    },
                );
            }
            // ..xx000000xx..
            //      ^__^
            (Ok(idx_s), Ok(idx_e)) if idx_s == idx_e => {
                let end_segment_range_end = mem::replace(&mut self.ranges[idx_s].range.end, start);
                let split_value = self.ranges[idx_s].value.clone();
                let value = value(Entries {
                    it: vec![Entry {
                        range: Range { start, end },
                        value: split_value.clone(),
                    }],
                    covariant: PhantomData,
                });
                // insert the split off tail
                self.ranges.insert(
                    idx_s + 1,
                    Entry {
                        range: Range {
                            start: end,
                            end: end_segment_range_end,
                        },
                        value: split_value,
                    },
                );
                // insert our range
                self.ranges.insert(idx_s + 1, Entry { range, value });
            }
            // ..xx000x..x000xx..
            //      ^______^
            (Ok(idx_s), Ok(idx_e)) => {
                let x = mem::replace(&mut self.ranges[idx_s].range.end, start);
                let x2 = mem::replace(&mut self.ranges[idx_e].range.start, end);
                let mut overlap: Vec<_> = self.ranges.drain(idx_s + 1..idx_e).collect();
                overlap.insert(
                    0,
                    Entry {
                        range: Range { start, end: x },
                        value: self.ranges[idx_s].value.clone(),
                    },
                );
                overlap.push(Entry {
                    range: Range { start: x2, end },
                    value: self.ranges[idx_e].value.clone(),
                });

                let value = value(Entries {
                    it: overlap,
                    covariant: PhantomData,
                });
                self.ranges.insert(idx_s + 1, Entry { range, value });
            }
            // ..xx000x..00xxx..
            //      ^_______^
            (Ok(idx_s), Err(idx_e)) => {
                let x = mem::replace(&mut self.ranges[idx_s].range.end, start);

                let mut overlap: Vec<_> = self.ranges.drain(idx_s + 1..idx_e).collect();
                overlap.insert(
                    0,
                    Entry {
                        range: Range { start, end: x },
                        value: self.ranges[idx_s].value.clone(),
                    },
                );

                let value = value(Entries {
                    it: overlap,
                    covariant: PhantomData,
                });
                self.ranges.insert(idx_s + 1, Entry { range, value });
            }
            // ..xxx000x..x0x..
            //    ^________^
            (Err(idx_s), Ok(idx_e)) => {
                let x2 = mem::replace(&mut self.ranges[idx_e].range.start, end);
                let mut overlap: Vec<_> = self.ranges.drain(idx_s..idx_e).collect();
                overlap.push(Entry {
                    range: Range { start: x2, end },
                    value: self.ranges[idx_e].value.clone(),
                });

                let value = value(Entries {
                    it: overlap,
                    covariant: PhantomData,
                });
                self.ranges.insert(idx_s + 1, Entry { range, value });
            }
        };
    }
}

impl<Idx: OrderedIndex, V> InversionMap<Idx, V> {
    /// Adds a unit range(index..index + 1) to the inversion list. This is faster than using
    /// [`add_range`] saving a second binary_search.
    ///
    /// If the unit is not part of an existing range, `true` is returned.
    ///
    /// If the unit already exists in a range, `false` is returned.
    ///
    /// # Panics
    ///
    /// Panics if index is equal to [`Idx::max_value()`].
    pub fn add_unit(&mut self, index: Idx, value: V) -> bool {
        match self.binary_search(index) {
            Err(insert_idx) => {
                // this creates a new unit range that may be directly adjacent to an existing one
                // have a method that tries to merge them directly as well?
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
            Ok(_) => false,
        }
    }

    pub fn add_range<R: RangeBounds<Idx>>(
        &mut self,
        range: R,
        val_insert: impl FnOnce(EntriesRef<'_, Idx, V>) -> V,
    ) {
        let (start, end) = match bounds_to_range(range) {
            Some(range) => (range.start, range.end),
            None => return,
        };

        let (idx_s, keep_start) = match self.binary_search(start) {
            Ok(idx) => (idx, true),
            // range is outside span, append
            Err(idx) if idx == self.ranges.len() => {
                return self.ranges.push(Entry {
                    range: start..end,
                    value: val_insert(EntriesRef { it: &[] }),
                });
            }
            Err(idx) => (idx, false),
        };
        let (idx_e, keep_end) = match self.binary_search(end) {
            Ok(idx) => (idx, true),
            // range is outside span, prepend
            Err(idx) if idx == 0 => {
                return self.ranges.insert(
                    0,
                    Entry {
                        range: start..end,
                        value: val_insert(EntriesRef { it: &[] }),
                    },
                );
            }
            Err(idx) => (idx, false),
        };

        let val = val_insert(EntriesRef {
            it: &self.ranges[idx_s..idx_e],
        });
        self.ranges[idx_s].value = val;
        self.ranges[idx_s].range = Range {
            start: if keep_start {
                self.ranges[idx_s].range.start
            } else {
                start
            },
            end: if keep_end {
                self.ranges[idx_e].range.end
            } else {
                end
            },
        };
        if idx_s < idx_e {
            if keep_end {
                self.ranges.drain(idx_s + 1..=idx_e);
            } else {
                self.ranges.drain(idx_s + 1..idx_e);
            }
        }
    }

    pub fn remove_range_at(&mut self, idx: usize) -> Option<(Range<Idx>, V)> {
        idx.le(&self.len()).then(|| self.ranges.remove(idx).into())
    }
}

// raw index based methods
impl<Idx: OrderedIndex, V: Clone> InversionMap<Idx, V> {
    pub fn remove_range<R: RangeBounds<Idx>>(&mut self, range: R) {
        let (start, end) = match bounds_to_range(range) {
            Some(range) => (range.start, range.end),
            None => return,
        };

        let (idx_s, idx_e) = match self.binary_search(start) {
            Ok(idx_s) => {
                let (_, idx_s) = self.split_impl(idx_s, start, |_, v| (v.clone(), v));
                match self.binary_search(end) {
                    Ok(idx_e) => {
                        let (_, right) = self.split_impl(idx_e, end, |_, v| (v.clone(), v));
                        (idx_s, right)
                    }
                    Err(idx_e) => (idx_s, idx_e),
                }
            }
            Err(idx_s) => match self.binary_search(end) {
                Ok(idx_e) => {
                    let (_, right) = self.split_impl(idx_e, end, |_, v| (v.clone(), v));
                    (idx_s, right)
                }
                Err(idx_e) => (idx_s, idx_e),
            },
        };
        self.ranges.drain(idx_s..idx_e);
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

    pub(crate) fn binary_search(&self, ref key: Idx) -> Result<usize, usize> {
        self.ranges.binary_search_by(move |Entry { range, .. }| {
            Self::bin_search_ordering(range.start.cmp(key), key.cmp(&range.end))
        })
    }

    pub(crate) fn d_binary_search(
        &self,
        start: Idx,
        ref end: Idx,
    ) -> (Result<usize, usize>, Result<usize, usize>) {
        let start @ (Ok(idx) | Err(idx)) = self.binary_search(start);
        let end = self.ranges[idx..].binary_search_by(move |Entry { range, .. }| {
            Self::bin_search_ordering(range.start.cmp(end), end.cmp(&range.end))
        });

        (
            start,
            match end {
                Ok(e) => Ok(idx + e),
                Err(e) => Err(idx + e),
            },
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! im {
        ($($range:expr => $val:expr),* $(,)?) => {
            InversionMap {
                ranges: alloc::vec![
                    $(Entry { range: $range, value: $val }),*
                ],
            }
        };
    }

    #[test]
    fn binary_search() {
        let il = im![0..5 => 0, 5..15 => 0, 20..25 => 0];
        assert_eq!(Ok(0), il.binary_search(0));
        assert_eq!(Ok(0), il.binary_search(1));
        assert_eq!(Ok(1), il.binary_search(5));
        assert_eq!(Err(2), il.binary_search(15));
        assert_eq!(Err(2), il.binary_search(16));
        assert_eq!(Ok(2), il.binary_search(20));
        assert_eq!(Err(3), il.binary_search(25));
    }

    // #[test]
    // fn merge() {
    //     let mut il = im![0..5, 5..15, 20..25];
    //     il.merge(0, 2);
    //     assert_eq!(il, im![0..25]);
    // }

    #[test]
    fn split_inorder() {
        let mut il = im![0..100 => 0];
        il.split(5);
        il.split(15);
        il.split(25);
        assert_eq!(il, im![0..5 => 0, 5..15 => 0, 15..25 => 0, 25..100 => 0,]);
    }

    #[test]
    fn split_outoforder() {
        let mut il = im![0..100 => 0];
        il.split(25);
        il.split(5);
        il.split(15);
        assert_eq!(il, im![0..5 => 0, 5..15 => 0, 15..25 => 0, 25..100 => 0,]);
    }

    #[test]
    fn split_double() {
        let mut il = im![0..100 => 0];
        il.split(50);
        il.split(50);
        assert_eq!(il, im![0..50 => 0, 50..100 => 0]);
    }

    #[test]
    fn split_boundary_left() {
        let mut il = im![0..100 => 0];
        il.split(0);
        assert_eq!(il, im![0..100 => 0]);
    }

    #[test]
    fn split_boundary_right() {
        let mut il = im![0..100 => 0];
        il.split(100);
        assert_eq!(il, im![0..100 => 0]);
    }

    #[test]
    fn split_out_of_bounds() {
        let mut il = im![1..100 => 0];
        il.split(101);
        il.split(1);
        assert_eq!(il, im![1..100 => 0]);
    }

    #[test]
    fn add_range_start() {
        let mut il = im![0..10 => 0];
        il.add_range(0..45, |_| 1);
        assert_eq!(il, im![0..45 => 1]);
    }

    #[test]
    fn add_range_end() {
        let mut il = im![0..10 => 0, 20..30 => 0];
        il.add_range(5..10, |_| 1);
        il.add_range(15..30, |_| 1);
        assert_eq!(il, im![0..10 => 1, 15..30 => 1]);
        let mut il = im![0..10 => 0, 20..30 => 0];
        il.add_range(15..20, |_| 1);
        assert_eq!(il, im![0..10 => 0, 15..30 => 1]);
    }

    #[test]
    fn add_range_in_in() {
        let mut il = im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
        il.add_range(5..45, |_| 1);
        assert_eq!(il, im![0..50 => 1, 60..70 => 0]);
    }

    #[test]
    fn add_range_in_out() {
        let mut il = im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
        il.add_range(5..35, |_| 1);
        assert_eq!(il, im![0..35 => 1, 40..50 => 0, 60..70 => 0]);
    }

    #[test]
    fn add_range_out_in() {
        let mut il = im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
        il.add_range(15..45, |_| 1);
        assert_eq!(il, im![0..10 => 0, 15..50 => 1, 60..70 => 0]);
    }

    #[test]
    fn add_range_out_out() {
        let mut il = im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
        il.add_range(15..55, |_| 1);
        assert_eq!(il, im![0..10 => 0, 15..55 => 1, 60..70 => 0]);
    }

    #[test]
    fn add_range_ignore_max_range() {
        // test to make sure we dont overflow
        let mut il = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
        il.add_range(!0..!0, |_| 1);
        assert_eq!(il, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
    }

    #[test]
    fn remove_range_in_in() {
        let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
        il.remove_range(5..45);
        assert_eq!(il, im![1..5 => 0, 45..50 => 0]);
        let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
        il.remove_range(5..40);
        assert_eq!(il, im![1..5 => 0, 40..50 => 0]);
    }

    #[test]
    fn remove_range_in_out() {
        let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
        il.remove_range(5..35);
        assert_eq!(il, im![1..5 => 0, 40..50 => 0]);
    }

    #[test]
    fn remove_range_out_in() {
        let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
        il.remove_range(15..45);
        assert_eq!(il, im![1..10 => 0, 45..50 => 0]);
    }

    #[test]
    fn remove_range_out_out() {
        let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
        il.remove_range(15..35);
        assert_eq!(il, im![1..10 => 0, 40..50 => 0]);
    }

    #[test]
    fn remove_range_subset() {
        let mut il = im![0..100 => 0];
        il.remove_range(50..75);
        assert_eq!(il, im![0..50 => 0, 75..100 => 0]);
    }

    #[test]
    fn remove_range_superset() {
        let mut il = im![0..100 => 0];
        il.remove_range(0..175);
        assert_eq!(il, im![]);
    }

    #[test]
    fn remove_range_end() {
        let mut il = im![0..100 => 0];
        il.remove_range(50..100);
        assert_eq!(il, im![0..50 => 0]);
    }

    #[test]
    fn remove_range_start() {
        let mut il = im![0..100 => 0];
        il.remove_range(0..50);
        assert_eq!(il, im![50..100 => 0]);
    }

    // #[test]
    // fn is_subset() {
    //     let il = im![1..10, 15..26, 61..100];
    //     let il2 = im![1..5, 17..22, 77..88];
    //     let il3 = im![1..10, 77..88];
    //     assert!(il.is_subset(&il));
    //     assert!(il2.is_subset(&il));
    //     assert!(il3.is_subset(&il));
    //     assert!(!il.is_subset(&il2));
    //     assert!(!il.is_subset(&il3));

    //     assert!(il.is_superset(&il));
    //     assert!(il.is_superset(&il2));
    //     assert!(il.is_superset(&il3));
    //     assert!(!il2.is_superset(&il));
    //     assert!(!il3.is_superset(&il));
    // }

    // #[test]
    // fn is_subset_strict() {
    //     let il = im![1..10, 15..26, 61..100];
    //     let il2 = im![1..10, 17..22, 77..88];
    //     let il3 = im![1..10, 77..88];
    //     assert!(il.is_subset_strict(&il));
    //     assert!(!il2.is_subset_strict(&il));
    //     assert!(il3.is_subset_strict(&il2));

    //     assert!(il.is_superset_strict(&il));
    //     assert!(!il.is_superset_strict(&il2));
    //     assert!(il2.is_superset_strict(&il3));
    // }

    // #[test]
    // fn is_disjoint() {
    //     let il = im![1..10, 15..26, 61..100];
    //     let il2 = im![1..5, 17..22, 77..88, 100..166];
    //     let il3 = im![1..10, 37..54, 66..100];
    //     let il4 = im![10..15, 44..55, 60..61];
    //     assert!(!il.is_disjoint(&il));
    //     assert!(!il.is_disjoint(&il2));
    //     assert!(!il.is_disjoint(&il3));
    //     assert!(il.is_disjoint(&il4));
    // }

    #[test]
    fn intersects() {
        let il = im![1..10 => 0, 15..26 => 0, 61..100 => 0];
        assert!(il.intersects(5..10));
        assert!(!il.intersects(0..1));
        assert!(il.intersects(12..17));
        assert!(il.intersects(20..30));
    }

    // #[test]
    // fn collapse() {
    //     let mut il = im![1..10, 10..26, 30..33, 33..35, 35..40, 41..45];
    //     il.collapse();
    //     assert_eq!(il, im![1..26, 30..40, 41..45]);
    // }

    // #[test]
    // fn invert() {
    //     let mut il = im![1..10, 10..26, 30..33, 33..35, 35..40, 41..45];
    //     il.invert();
    //     assert_eq!(il, im![0usize..1, 26..30, 40..41]);
    //     let mut il = im![0usize..10, 15..26, 26..33, 34..35, 35..36];
    //     il.invert();
    //     assert_eq!(il, im![10..15, 33..34]);
    // }

    // #[test]
    // fn test_bitand() {
    //     let il = im![0..5, 5..15, 20..25, 50..80];
    //     let il2 = im![0..5, 7..10, 12..18, 19..27, 30..40, 45..55, 57..60, 78..82,];
    //     assert_eq!(
    //         il & il2,
    //         im![0..5, 7..10, 12..15, 20..25, 50..55, 57..60, 78..80]
    //     );
    // }

    // #[test]
    // fn test_bitor() {
    //     let il = im![0..5, 5..15, 20..25, 50..80];
    //     let il2 = im![0..5, 7..10, 12..18, 19..27, 30..40, 45..55, 57..60, 78..82,];
    //     assert_eq!(il | il2, im![0..5, 5..18, 19..27, 30..40, 45..82]);
    // }

    // #[test]
    // fn test_not() {
    //     let il = im![0usize..5, 5..15, 20..25, 50..80];
    //     assert_eq!(!il, im![15..20, 25..50, 80..!0]);
    //     let il = im![5..15, 20..25, 50..80];
    //     assert_eq!(!il, im![0usize..5, 15..20, 25..50, 80..!0]);
    // }
    #[test]
    fn insert_range() {
        let mut il = im![5u8..100 => false];
        il.insert_range(0..5, true);
        assert_eq!(il, im![0..5 => true, 5..100 => false]);

        let mut il = im![5u8..100 => false];
        il.insert_range(100..105, true);
        assert_eq!(il, im![5..100 => false, 100..105 => true]);

        let mut il = im![5u8..10 => false, 15..20 => false];
        il.insert_range(10..15, true);
        assert_eq!(il, im![5u8..10 => false, 10..15 => true, 15..20 => false]);

        let mut il = im![5u8..10 => false, 15..20 => false];
        il.insert_range(10..17, true);
        assert_eq!(il, im![5u8..10 => false, 10..17 => true, 17..20 => false]);

        let mut il = im![5u8..10 => false];
        il.insert_range(7..9, true);
        assert_eq!(il, im![5u8..7 => false, 7..9 => true, 9..10 => false]);

        let mut il = im![5u8..10 => false, 15..20 => false];
        il.insert_range(7..17, true);
        assert_eq!(il, im![5u8..7 => false, 7..17 => true, 17..20 => false]);
        let mut il = im![5u8..10 => false, 12..14 => false, 15..20 => false];
        il.insert_range(7..17, true);
        assert_eq!(il, im![5u8..7 => false, 7..17 => true, 17..20 => false]);

        let mut il = im![5u8..10 => false, 12..14 => false, 15..20 => false];
        il.insert_range(7..22, true);
        assert_eq!(il, im![5u8..7 => false, 7..22 => true]);

        let mut il = im![5u8..10 => false, 12..14 => false, 15..20 => false];
        il.insert_range(2..17, true);
        assert_eq!(il, im![2u8..17 => true, 17..20 => false]);

        let mut il = im![5u8..10 => false, 12..14 => false, 15..20 => false];
        il.insert_range(2..22, true);
        assert_eq!(il, im![2u8..22 => true]);
    }
}
