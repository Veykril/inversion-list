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

// region: add_range

#[test]
fn add_range_in_in() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.add_range_with(6..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..10 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.add_range_with(6..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..20 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.add_range_with(6..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..30 => 1, 100..101 => 0]);
}

#[test]
fn add_range_in_out() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.add_range_with(6..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..12 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.add_range_with(6..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..22 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.add_range_with(6..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..32 => 1, 100..101 => 0]);
}

#[test]
fn add_range_out_in() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.add_range_with(3..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..10 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.add_range_with(3..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..20 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.add_range_with(3..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..30 => 1, 100..101 => 0]);
}

#[test]
fn add_range_out_out() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.add_range_with(10..12, |entries| {
        assert_eq!(entries.len(), 0);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..10 => 0, 10..12 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.add_range_with(3..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..12 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.add_range_with(3..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..22 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.add_range_with(3..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..32 => 1, 100..101 => 0]);
}

#[test]
fn add_range_ignore_max_range() {
    // test to make sure we dont overflow
    let mut il = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    il.add_range(!0..!0, 1);
    assert_eq!(il, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

#[test]
fn add_range_ignore_min_range() {
    // test to make sure we dont underflow
    let mut il = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    il.add_range(0..0, 1);
    assert_eq!(il, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

// endregion

// region:insert_range

#[test]
fn insert_range_in_in() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.insert_range_with(6..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(
        il,
        im![0..1 => 0, 5..6 => 0, 6..8 => 1, 8..10 => 0, 100..101 => 0]
    );

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.insert_range_with(6..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(
        il,
        im![0..1 => 0, 5..6 => 0, 6..18 => 1, 18..20 => 0, 100..101 => 0]
    );

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.insert_range_with(6..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(
        il,
        im![0..1 => 0, 5..6 => 0, 6..28 => 1, 28..30 => 0, 100..101 => 0]
    );
}

#[test]
fn insert_range_in_out() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.insert_range_with(6..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..6 => 0, 6..12 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.insert_range_with(6..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..6 => 0, 6..22 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.insert_range_with(6..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..6 => 0, 6..32 => 1, 100..101 => 0]);
}

#[test]
fn insert_range_out_in() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.insert_range_with(3..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..8 => 1, 8..10 => 0, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.insert_range_with(3..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..18 => 1, 18..20 => 0, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.insert_range_with(3..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..28 => 1, 28..30 => 0, 100..101 => 0]);
}

#[test]
fn insert_range_out_out() {
    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.insert_range_with(10..12, |entries| {
        assert_eq!(entries.len(), 0);
        1
    });
    assert_eq!(il, im![0..1 => 0, 5..10 => 0, 10..12 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    il.insert_range_with(3..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..12 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    il.insert_range_with(3..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..22 => 1, 100..101 => 0]);

    let mut il = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    il.insert_range_with(3..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(il, im![0..1 => 0, 3..32 => 1, 100..101 => 0]);
}

#[test]
fn insert_range_ignore_max_range() {
    // test to make sure we dont overflow
    let mut il = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    il.insert_range(!0..!0, 1);
    assert_eq!(il, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

#[test]
fn insert_range_ignore_min_range() {
    // test to make sure we dont underflow
    let mut il = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    il.insert_range(0..0, 1);
    assert_eq!(il, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

// endregion

#[test]
fn remove_range_in_in() {
    let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    il.remove_range(5..45, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![1..5 => 1, 45..50 => 2]);
    let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    il.remove_range(5..40, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![1..5 => 1, 40..50 => 0]);
}

#[test]
fn remove_range_in_out() {
    let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    il.remove_range(5..35, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![1..5 => 1, 40..50 => 0]);
}

#[test]
fn remove_range_out_in() {
    let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    il.remove_range(15..45, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![1..10 => 0, 45..50 => 2]);
}

#[test]
fn remove_range_out_out() {
    let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    il.remove_range(15..35, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![1..10 => 0, 40..50 => 0]);
    let mut il = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    il.remove_range(15..30, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![1..10 => 0, 40..50 => 0]);
}

#[test]
fn remove_range_subset() {
    let mut il = im![0..100 => 0];
    il.remove_range(50..75, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![0..50 => 1, 75..100 => 2]);
}

#[test]
fn remove_range_superset() {
    let mut il = im![0..100 => 0];
    il.remove_range(0..175, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![]);
}

#[test]
fn remove_range_end() {
    let mut il = im![0..100 => 0];
    il.remove_range(50..100, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![0..50 => 1]);
}

#[test]
fn remove_range_start() {
    let mut il = im![0..100 => 0];
    il.remove_range(0..50, |_, _| 1, |_, _| 2);
    assert_eq!(il, im![50..100 => 2]);
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
    assert!(il.intersects(2..8));
    assert!(il.intersects(0..11));
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
