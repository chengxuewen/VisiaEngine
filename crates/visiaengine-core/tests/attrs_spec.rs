//! CORE-11/12/13: the AttrSet columnar-attribute contract ([E3D:A1] port, closed enum columns, no dyn).

use visiaengine_core::AttrSet;

// spec: CORE-11
#[test]
fn three_typed_sets_and_gets_with_missing_none() {
    let mut a = AttrSet::new();
    let r = a.add_row();
    assert!(a.set_f64(r, "height", 12.5));
    assert!(a.set_str(r, "label", "Tower A"));
    assert!(a.set_bool(r, "active", true));
    assert_eq!(a.f64(r, "height"), Some(12.5));
    assert_eq!(a.str_value(r, "label"), Some("Tower A"));
    assert_eq!(a.bool(r, "active"), Some(true));
    // missing = None (the semantic split between "does not exist" and "value is 0/false", style-key decisions rely on this)
    assert_eq!(a.f64(r, "absent"), None);
    assert_eq!(
        a.f64(999, "height"),
        None,
        "row out of bounds must be None, no panic"
    );
    assert_eq!(
        a.str_value(r, "height"),
        None,
        "cross-type read must be None, no panic"
    );
}

// spec: CORE-12
#[test]
fn row_alignment_is_lazy_and_dense() {
    let mut a = AttrSet::new();
    a.add_row();
    let r5 = a.add_row();
    assert_eq!(r5, 1);
    // setting to an out-of-bounds row number = implicit row expansion (row alignment is a storage invariant, not a caller duty)
    assert!(a.set_f64(4, "x", 1.0));
    assert_eq!(a.len(), 5);
    // rows never written are None in the columns, reads do not overrun
    assert_eq!(a.f64(0, "x"), None);
    assert_eq!(a.f64(2, "x"), None);
    assert_eq!(a.f64(4, "x"), Some(1.0));
    // rewriting the same name on the same row = overwrite (value semantics)
    assert!(a.set_f64(4, "x", 2.0));
    assert_eq!(a.f64(4, "x"), Some(2.0));
}

// spec: CORE-13
#[test]
fn first_write_freezes_column_type() {
    let mut a = AttrSet::new();
    let r = a.add_row();
    assert!(a.set_f64(r, "mixed", 1.0));
    // mismatched-type writes are rejected (false) and the data is unchanged (first-write fixes the type = column type stays stable, render bindings may rely on it)
    assert!(!a.set_str(r, "mixed", "now-string"));
    assert_eq!(a.f64(r, "mixed"), Some(1.0));
    assert_eq!(a.str_value(r, "mixed"), None);
}
