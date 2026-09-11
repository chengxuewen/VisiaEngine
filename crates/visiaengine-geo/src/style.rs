//! simplestyle-spec 六键子集（GEO-12/13/18）。未知键静默忽略、缺失回默认。
//! GEO-18：样式判定经 AttrSet typed 列读取——本文件禁现 serde_json（解析边界
//! 固定 lib.rs；grep 门禁=pixi run gate-style）。

use visiaengine_core::AttrSet;

/// `#rrggbb` / `rgb(r,g,b)` 双格式；非法串 None（GEO-12 色格式面）。
#[must_use]
pub fn parse_color(raw: &str) -> Option<[f32; 4]> {
    if let Some(hex) = raw.strip_prefix('#') {
        if hex.len() != 6 {
            return None;
        }
        let v = u32::from_str_radix(hex, 16).ok()?;
        return Some([
            ((v >> 16) & 0xFF) as f32 / 255.0,
            ((v >> 8) & 0xFF) as f32 / 255.0,
            (v & 0xFF) as f32 / 255.0,
            1.0,
        ]);
    }
    let inner = raw.strip_prefix("rgb(")?.strip_suffix(')')?;
    let mut it = inner.split(',');
    let n = |s: &str| s.trim().parse::<f32>().ok().map(|v| v / 255.0);
    Some([n(it.next()?)?, n(it.next()?)?, n(it.next()?)?, 1.0])
}

/// 键取：simplestyle 主键优先，v8 paint 别名回退（D9/GEO-19）。
fn str_first<'a>(a: &'a AttrSet, row: usize, primary: &str, alias: &str) -> Option<&'a str> {
    a.str_value(row, primary)
        .or_else(|| a.str_value(row, alias))
}
fn f64_first(a: &AttrSet, row: usize, primary: &str, alias: &str) -> Option<f64> {
    a.f64(row, primary).or_else(|| a.f64(row, alias))
}

/// 经列读六键（typed，零 Value 匹配）。缺键=该列 None→默认值保持。
pub(crate) fn style_from_attrs(a: &AttrSet, row: usize) -> crate::StyleRecord {
    let mut s = crate::StyleRecord::default();
    if let Some(c) = str_first(a, row, "fill", "fill-color").and_then(parse_color) {
        s.fill = c;
    }
    if let Some(v) = a.f64(row, "fill-opacity") {
        s.fill_opacity = v.clamp(0.0, 1.0) as f32;
    }
    if let Some(c) = str_first(a, row, "stroke", "line-color").and_then(parse_color) {
        s.stroke = c;
    }
    if let Some(v) = f64_first(a, row, "stroke-width", "line-width") {
        // simplestyle/v8 语义均为屏幕 px；v0 粗映射 1px≈1m（屏幕空间线宽=相机片，GEO-11 以默认宽断言）
        s.stroke_width_m = (v as f32).max(0.5);
    }
    if let Some(c) = str_first(a, row, "marker-color", "circle-color").and_then(parse_color) {
        s.marker_color = c;
    }
    if let Some(v) = f64_first(a, row, "marker-radius", "circle-radius") {
        s.radius_m = v as f32;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::parse_color;

    // spec: GEO-12
    #[test]
    fn hex_color_parsed() {
        let c = parse_color("#ff0000").unwrap();
        assert!((c[0] - 1.0).abs() < 1e-6 && c[1] < 1e-6 && c[2] < 1e-6);
        assert!(parse_color("#xyz123").is_none());
    }

    // spec: GEO-12
    #[test]
    fn rgb_function_parsed() {
        let c = parse_color("rgb(255, 128, 0)").unwrap();
        assert!((c[0] - 1.0).abs() < 1e-6 && (c[1] - 0.502).abs() < 0.01);
        assert!(parse_color("nonsense").is_none());
    }
}

// style.rs 依赖面契约（GEO-18 禁 serde_json）由 pixi run gate-style 机器把关。
