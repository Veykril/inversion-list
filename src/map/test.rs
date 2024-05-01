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
    let im = im![0..5 => 0, 5..15 => 0, 20..25 => 0];
    assert_eq!(Ok(0), im.binary_search(0));
    assert_eq!(Ok(0), im.binary_search(1));
    assert_eq!(Ok(1), im.binary_search(5));
    assert_eq!(Err(2), im.binary_search(15));
    assert_eq!(Err(2), im.binary_search(16));
    assert_eq!(Ok(2), im.binary_search(20));
    assert_eq!(Err(3), im.binary_search(25));
}

// #[test]
// fn merge() {
//     let mut im = im![0..5, 5..15, 20..25];
//     im.merge(0, 2);
//     assert_eq!(im, im![0..25]);
// }

#[test]
fn split_inorder() {
    let mut im = im![0..100 => 0];
    im.split(5);
    im.split(15);
    im.split(25);
    assert_eq!(im, im![0..5 => 0, 5..15 => 0, 15..25 => 0, 25..100 => 0,]);
}

#[test]
fn split_outoforder() {
    let mut im = im![0..100 => 0];
    im.split(25);
    im.split(5);
    im.split(15);
    assert_eq!(im, im![0..5 => 0, 5..15 => 0, 15..25 => 0, 25..100 => 0,]);
}

#[test]
fn split_double() {
    let mut im = im![0..100 => 0];
    im.split(50);
    im.split(50);
    assert_eq!(im, im![0..50 => 0, 50..100 => 0]);
}

#[test]
fn split_boundary_left() {
    let mut im = im![0..100 => 0];
    im.split(0);
    assert_eq!(im, im![0..100 => 0]);
}

#[test]
fn split_boundary_right() {
    let mut im = im![0..100 => 0];
    im.split(100);
    assert_eq!(im, im![0..100 => 0]);
}

#[test]
fn split_out_of_bounds() {
    let mut im = im![1..100 => 0];
    im.split(101);
    im.split(1);
    assert_eq!(im, im![1..100 => 0]);
}

// region: add_range

#[test]
fn add_range_in_in() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.add_range_with(6..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..10 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.add_range_with(6..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..20 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.add_range_with(6..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..30 => 1, 100..101 => 0]);
}

#[test]
fn add_range_in_out() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.add_range_with(6..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..12 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.add_range_with(6..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..22 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.add_range_with(6..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..32 => 1, 100..101 => 0]);
}

#[test]
fn add_range_out_in() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.add_range_with(3..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..10 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.add_range_with(3..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..20 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.add_range_with(3..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..30 => 1, 100..101 => 0]);
}

#[test]
fn add_range_out_out() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.add_range_with(10..12, |entries| {
        assert_eq!(entries.len(), 0);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..10 => 0, 10..12 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.add_range_with(3..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..12 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.add_range_with(3..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..22 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.add_range_with(3..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..32 => 1, 100..101 => 0]);
}

#[test]
fn add_range_ignore_max_range() {
    // test to make sure we dont overflow
    let mut im = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    im.add_range(!0..!0, 1);
    assert_eq!(im, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

#[test]
fn add_range_ignore_min_range() {
    // test to make sure we dont underflow
    let mut im = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    im.add_range(0..0, 1);
    assert_eq!(im, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

// endregion

// region:insert_range

#[test]
fn insert_range_in_in() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.insert_range_with(6..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(
        im,
        im![0..1 => 0, 5..6 => 0, 6..8 => 1, 8..10 => 0, 100..101 => 0]
    );

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.insert_range_with(6..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(
        im,
        im![0..1 => 0, 5..6 => 0, 6..18 => 1, 18..20 => 0, 100..101 => 0]
    );

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.insert_range_with(6..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(
        im,
        im![0..1 => 0, 5..6 => 0, 6..28 => 1, 28..30 => 0, 100..101 => 0]
    );
}

#[test]
fn insert_range_in_out() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.insert_range_with(6..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..6 => 0, 6..12 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.insert_range_with(6..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..6 => 0, 6..22 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.insert_range_with(6..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..6 => 0, 6..32 => 1, 100..101 => 0]);
}

#[test]
fn insert_range_out_in() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.insert_range_with(3..8, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..8 => 1, 8..10 => 0, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.insert_range_with(3..18, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..18 => 1, 18..20 => 0, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.insert_range_with(3..28, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..28 => 1, 28..30 => 0, 100..101 => 0]);
}

#[test]
fn insert_range_out_out() {
    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.insert_range_with(10..12, |entries| {
        assert_eq!(entries.len(), 0);
        1
    });
    assert_eq!(im, im![0..1 => 0, 5..10 => 0, 10..12 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 100..101 => 0];
    im.insert_range_with(3..12, |entries| {
        assert_eq!(entries.len(), 1);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..12 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 100..101 => 0];
    im.insert_range_with(3..22, |entries| {
        assert_eq!(entries.len(), 2);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..22 => 1, 100..101 => 0]);

    let mut im = im![0..1 => 0, 5..10 => 0, 15..20 => 0, 25..30 => 0, 100..101 => 0];
    im.insert_range_with(3..32, |entries| {
        assert_eq!(entries.len(), 3);
        1
    });
    assert_eq!(im, im![0..1 => 0, 3..32 => 1, 100..101 => 0]);
}

#[test]
fn insert_range_ignore_max_range() {
    // test to make sure we dont overflow
    let mut im = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    im.insert_range(!0..!0, 1);
    assert_eq!(im, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

#[test]
fn insert_range_ignore_min_range() {
    // test to make sure we dont underflow
    let mut im = im![0usize..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0];
    im.insert_range(0..0, 1);
    assert_eq!(im, im![0..10 => 0, 20..30 => 0, 40..50 => 0, 60..70 => 0]);
}

// endregion

#[test]
fn remove_range_in_in() {
    let mut im = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    im.remove_range(5..45, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![1..5 => 1, 45..50 => 2]);
    let mut im = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    im.remove_range(5..40, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![1..5 => 1, 40..50 => 0]);
}

#[test]
fn remove_range_in_out() {
    let mut im = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    im.remove_range(5..35, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![1..5 => 1, 40..50 => 0]);
}

#[test]
fn remove_range_out_in() {
    let mut im = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    im.remove_range(15..45, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![1..10 => 0, 45..50 => 2]);
}

#[test]
fn remove_range_out_out() {
    let mut im = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    im.remove_range(15..35, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![1..10 => 0, 40..50 => 0]);
    let mut im = im![1..10 => 0, 20..30 => 0, 40..50 => 0];
    im.remove_range(15..30, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![1..10 => 0, 40..50 => 0]);
}

#[test]
fn remove_range_subset() {
    let mut im = im![0..100 => 0];
    im.remove_range(50..75, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![0..50 => 1, 75..100 => 2]);
}

#[test]
fn remove_range_superset() {
    let mut im = im![0..100 => 0];
    im.remove_range(0..175, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![]);
}

#[test]
fn remove_range_end() {
    let mut im = im![0..100 => 0];
    im.remove_range(50..100, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![0..50 => 1]);
}

#[test]
fn remove_range_start() {
    let mut im = im![0..100 => 0];
    im.remove_range(0..50, |_, _| 1, |_, _| 2);
    assert_eq!(im, im![50..100 => 2]);
}

// #[test]
// fn is_subset() {
//     let im = im![1..10, 15..26, 61..100];
//     let im2 = im![1..5, 17..22, 77..88];
//     let im3 = im![1..10, 77..88];
//     assert!(im.is_subset(&im));
//     assert!(im2.is_subset(&im));
//     assert!(im3.is_subset(&im));
//     assert!(!im.is_subset(&im2));
//     assert!(!im.is_subset(&im3));

//     assert!(im.is_superset(&im));
//     assert!(im.is_superset(&im2));
//     assert!(im.is_superset(&im3));
//     assert!(!im2.is_superset(&im));
//     assert!(!im3.is_superset(&im));
// }

// #[test]
// fn is_subset_strict() {
//     let im = im![1..10, 15..26, 61..100];
//     let im2 = im![1..10, 17..22, 77..88];
//     let im3 = im![1..10, 77..88];
//     assert!(im.is_subset_strict(&im));
//     assert!(!im2.is_subset_strict(&im));
//     assert!(im3.is_subset_strict(&im2));

//     assert!(im.is_superset_strict(&im));
//     assert!(!im.is_superset_strict(&im2));
//     assert!(im2.is_superset_strict(&im3));
// }

// #[test]
// fn is_disjoint() {
//     let im = im![1..10, 15..26, 61..100];
//     let im2 = im![1..5, 17..22, 77..88, 100..166];
//     let im3 = im![1..10, 37..54, 66..100];
//     let im4 = im![10..15, 44..55, 60..61];
//     assert!(!im.is_disjoint(&im));
//     assert!(!im.is_disjoint(&im2));
//     assert!(!im.is_disjoint(&im3));
//     assert!(im.is_disjoint(&im4));
// }

#[test]
fn intersects() {
    let im = im![1..10 => 0, 15..26 => 0, 61..100 => 0];
    assert!(im.intersects(5..10));
    assert!(!im.intersects(0..1));
    assert!(im.intersects(12..17));
    assert!(im.intersects(20..30));
    assert!(im.intersects(2..8));
    assert!(im.intersects(0..11));
}

// #[test]
// fn collapse() {
//     let mut im = im![1..10, 10..26, 30..33, 33..35, 35..40, 41..45];
//     im.collapse();
//     assert_eq!(im, im![1..26, 30..40, 41..45]);
// }

// #[test]
// fn invert() {
//     let mut im = im![1..10, 10..26, 30..33, 33..35, 35..40, 41..45];
//     im.invert();
//     assert_eq!(im, im![0usize..1, 26..30, 40..41]);
//     let mut im = im![0usize..10, 15..26, 26..33, 34..35, 35..36];
//     im.invert();
//     assert_eq!(im, im![10..15, 33..34]);
// }

#[test]
fn test_bitand() {
    let im = im![0..5 => 0xaaaa, 5..15 => 0xaaaa, 20..25 => 0xaaaa, 50..80 => 0xaaaa];
    let im2 = im![0..5 => 0xaa55, 7..10 => 0xaa55, 12..18 => 0xaa55, 19..27 => 0xaa55, 30..40 => 0xaa55, 45..55 => 0xaa55, 57..60 => 0xaa55, 78..82 => 0xaa55];
    assert_eq!(
        im & im2,
        im![0..5 => 0xaa00, 7..10 => 0xaa00, 12..15 => 0xaa00, 20..25 => 0xaa00, 50..55 => 0xaa00, 57..60 => 0xaa00, 78..80 => 0xaa00]
    );
}

#[test]
fn test_bitor() {
    let im = im![0..5 => 0xaaaa, 5..15 => 0xaaaa, 20..25 => 0xaaaa, 50..80 => 0xaaaa];
    let im2 = im![0..5 => 0x55, 7..10 => 0x55, 12..18 => 0x55, 19..27 => 0x55, 30..40 => 0x55, 45..55 => 0x55, 57..60 => 0x55, 78..82 => 0x55];
    assert_eq!(
        im | im2,
        // FIXME THis is wrong
        im![0..5 => 0xAAFF, 5..18 => 0xAAFF, 19..27 => 0xAAFF, 30..40 => 0x55, 45..82 => 0xAAFF]
    );
}

// #[test]
// fn test_not() {
//     let im = im![0usize..5, 5..15, 20..25, 50..80];
//     assert_eq!(!im, im![15..20, 25..50, 80..!0]);
//     let im = im![5..15, 20..25, 50..80];
//     assert_eq!(!im, im![0usize..5, 15..20, 25..50, 80..!0]);
// }
