//! REND-42 (N2.2): entity animation — keyframe origin sampling, pure fns.
//!
//! Zero engine state, zero ABI: the host drives t per frame and applies the
//! sampled origin through its existing position path (add_mesh origin /
//! set_group_offset). Easing lives with the HOST (time-axis orchestration is
//! host policy); the sampler interpolates linearly between keyframes with
//! exact endpoints (t ≤ first → first origin verbatim; t ≥ last → last
//! verbatim — fly_sample/mix_rig endpoint-exactness convention, REND-16).

/// Linear interpolation of an origin keyframe table.
///
/// * `keyframes` — ascending-time `(t, origin)` pairs in the KEYFRAME TIME
///   DOMAIN (not normalized; callers scale their own clock). Non-empty
///   contract, debug_assert-guarded.
/// * Endpoint-exact: clamped t outside the table returns the boundary origin
///   VERBATIM (bitwise; same law as mix_rig t=0/t=1 fast paths).
#[must_use]
pub fn anim_origin(keyframes: &[(f64, [f64; 3])], t: f64) -> [f64; 3] {
    debug_assert!(
        !keyframes.is_empty(),
        "anim_origin: empty keyframe table (contract: caller passes >=1)"
    );
    let Some(&(t0, first)) = keyframes.first() else {
        return [0.0; 3];
    };
    if t <= t0 {
        return first;
    }
    let Some(&(tn, last)) = keyframes.last() else {
        return first;
    };
    if t >= tn {
        return last;
    }
    // Find the segment [ti, ti+1) containing t (linear scan — tables are
    // tiny; ponytail: binary search when tables reach hundreds of keys).
    let mut i = 0;
    while i + 2 < keyframes.len() && keyframes[i + 1].0 <= t {
        i += 1;
    }
    let (ta, oa) = keyframes[i];
    let (tb, ob) = keyframes[i + 1];
    if tb <= ta {
        // Degenerate segment (non-ascending table): clamp to the later
        // origin — documented domain, release-safe.
        return ob;
    }
    let k = (t - ta) / (tb - ta);
    let l = |x: f64, y: f64| x + (y - x) * k;
    [l(oa[0], ob[0]), l(oa[1], ob[1]), l(oa[2], ob[2])]
}
