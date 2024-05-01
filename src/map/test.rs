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
