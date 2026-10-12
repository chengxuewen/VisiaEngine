//! The simplestyle-spec six-key subset (GEO-12/13/18). Unknown keys silently ignored, missing keys fall back to defaults.
//! GEO-18: style decisions read through AttrSet typed columns -- serde_json must never appear in this file (the parsing boundary is
//! fixed at lib.rs; the grep gate = pixi run gate-style).

use visiaengine_core::AttrSet;

/// Dual format `#rrggbb` / `rgb(r,g,b)`; invalid strings are None (the GEO-12 color-format face).
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

/// Key lookup: simplestyle primary keys first, v8 paint aliases as fallback (D9/GEO-19).
fn str_first<'a>(a: &'a AttrSet, row: usize, primary: &str, alias: &str) -> Option<&'a str> {
    a.str_value(row, primary)
        .or_else(|| a.str_value(row, alias))
}
fn f64_first(a: &AttrSet, row: usize, primary: &str, alias: &str) -> Option<f64> {
    a.f64(row, primary).or_else(|| a.f64(row, alias))
}

/// Read the six keys through columns (typed, zero Value matching). A missing key = that column is None -> the default value holds.
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
        // both simplestyle/v8 semantics are screen px; v0 coarse mapping 1px~1m (screen-space line width = camera slice, GEO-11 asserts with the default width)
        s.stroke_width_px = (v as f32).max(0.5);
    }
    if let Some(c) = str_first(a, row, "marker-color", "circle-color").and_then(parse_color) {
        s.marker_color = c;
    }
    if let Some(v) = f64_first(a, row, "marker-radius", "circle-radius") {
        s.radius_px = v as f32;
    }
    // GEO-25 three keys (the primary keys of the v8 layout domain; no simplestyle counterpart = the primary key is the canonical form)
    if let Some(v) = a.str_value(row, "text-field") {
        let owned = v.trim();
        let field = owned
            .strip_prefix('{')
            .and_then(|t| t.strip_suffix('}'))
            .unwrap_or(owned);
        if !field.is_empty() {
            s.text_field = Some(field.into());
        }
    }
    if let Some(v) = a.f64(row, "text-size") {
        s.text_size_px = (v as f32).clamp(1.0, 256.0);
    }
    if let Some(c) = a.str_value(row, "text-color").and_then(parse_color) {
        s.text_color = c;
    }
    apply_scalar_ramp(a, row, &mut s);
    s
}

/// Scalar -> color ramp (the `visia:` extension key beyond the GEO-20/D9 boundary, materialized at parse time --
/// zero changes to tess/the render pipeline; the real GIS attribute-coloring shape = per-feature constant color).
/// Any missing precondition (referenced column / interval / endpoint colors / hi <= lo) -> silently disabled, the six-key result holds.
/// The scalar -> color ramp executor (GEO-20); pub = M0 exposure: the reuse base for the io-points/E204 point-cloud band
pub fn apply_scalar_ramp(a: &AttrSet, row: usize, s: &mut crate::StyleRecord) {
    let (Some(col), Some(lo), Some(hi)) = (
        a.str_value(row, "visia:color-column"),
        a.f64(row, "visia:color-lo"),
        a.f64(row, "visia:color-hi"),
    ) else {
        return;
    };
    if hi <= lo {
        return;
    }
    let (Some(low), Some(high)) = (
        a.str_value(row, "visia:color-low").and_then(parse_color),
        a.str_value(row, "visia:color-high").and_then(parse_color),
    ) else {
        return;
    };
    let Some(v) = a.f64(row, col) else {
        return;
    };
    let t = (((v - lo) / (hi - lo)).clamp(0.0, 1.0)) as f32;
    let ramp = [0, 1, 2, 3].map(|i| low[i] + (high[i] - low[i]) * t);
    s.fill = ramp;
    s.marker_color = ramp;
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

// The style.rs dependency-surface contract (GEO-18 forbids serde_json) is machine-gated by pixi run gate-style.
