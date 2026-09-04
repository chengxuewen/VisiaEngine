//! simplestyle-spec 六键子集（GEO-12/13）。未知键静默忽略、缺失回默认。

use serde_json::Value as Json;

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

pub(crate) fn style_from_props(props: &Json) -> crate::StyleRecord {
    let mut s = crate::StyleRecord::default();
    let get = |k: &str| props.get(k);
    if let Some(c) = get("fill").and_then(Json::as_str).and_then(parse_color) {
        s.fill = c;
    }
    if let Some(v) = get("fill-opacity").and_then(Json::as_f64) {
        s.fill_opacity = v.clamp(0.0, 1.0) as f32;
    }
    if let Some(c) = get("stroke").and_then(Json::as_str).and_then(parse_color) {
        s.stroke = c;
    }
    if let Some(v) = get("stroke-width").and_then(Json::as_f64) {
        // simplestyle 语义为屏幕 px；v0 粗映射 1px≈1m（屏幕空间线宽=相机片，GEO-11 以默认宽断言）
        s.stroke_width_m = (v as f32).max(0.5);
    }
    if let Some(c) = get("marker-color")
        .and_then(Json::as_str)
        .and_then(parse_color)
    {
        s.marker_color = c;
    }
    if let Some(v) = get("marker-radius").and_then(Json::as_f64) {
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
