//! CORE-11/12/13：AttrSet 列式属性契约（[E3D:A1] 移植，闭合 enum 列，无 dyn）。

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
    // 缺失 = None（"不存在"与"值为 0/false"语义分离，样式键判定依赖此）
    assert_eq!(a.f64(r, "absent"), None);
    assert_eq!(a.f64(999, "height"), None, "行越界必须 None 不 panic");
    assert_eq!(a.str_value(r, "height"), None, "跨类型读必须 None 不 panic");
}

// spec: CORE-12
#[test]
fn row_alignment_is_lazy_and_dense() {
    let mut a = AttrSet::new();
    a.add_row();
    let r5 = a.add_row();
    assert_eq!(r5, 1);
    // set 到越界行号 = 隐式扩行（行对齐是存储不变式，非调用方义务）
    assert!(a.set_f64(4, "x", 1.0));
    assert_eq!(a.len(), 5);
    // 未写入的行在列中为 None，读取不越界
    assert_eq!(a.f64(0, "x"), None);
    assert_eq!(a.f64(2, "x"), None);
    assert_eq!(a.f64(4, "x"), Some(1.0));
    // 同行同名重写 = 覆盖（值语义）
    assert!(a.set_f64(4, "x", 2.0));
    assert_eq!(a.f64(4, "x"), Some(2.0));
}

// spec: CORE-13
#[test]
fn first_write_freezes_column_type() {
    let mut a = AttrSet::new();
    let r = a.add_row();
    assert!(a.set_f64(r, "mixed", 1.0));
    // 异型写拒绝（false）且不改数据（首写定型 = 列型稳定，渲染绑定可依赖）
    assert!(!a.set_str(r, "mixed", "now-string"));
    assert_eq!(a.f64(r, "mixed"), Some(1.0));
    assert_eq!(a.str_value(r, "mixed"), None);
}
