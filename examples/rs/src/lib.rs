//! The example-registration shell: the lib face carries only the small pieces shared across examples (the BBox framing fit); example bodies are in ../E*_*.rs.
//! The birth-preset ruling (v1.3 F4) of "eliminating examples-only conditional branches with an empty lib" is this file; it now really carries shared pieces.

pub mod viewer;

/// World-space AABB (row-vector convention `[p,1]*M`, translation in row 3 -- isomorphic with io-gltf/D7).
/// Purpose: the window examples' load-time framing fit -- the root fix of the E201 gray-screen case (the historical lesson: interactive camera parameters were never pixel-verified).
#[derive(Clone, Copy, Debug, Default)]
pub struct BBox {
    min: [f64; 3],
    max: [f64; 3],
    seen: bool,
}

impl BBox {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// transform a point through the world 4x4 (row vectors), then merge per axis.
    pub fn push(&mut self, p: [f32; 3], world: &[[f64; 4]; 4]) {
        let mut q = [0f64; 3];
        for (j, qj) in q.iter_mut().enumerate() {
            *qj = (p[0] as f64) * world[0][j]
                + (p[1] as f64) * world[1][j]
                + (p[2] as f64) * world[2][j]
                + world[3][j];
        }
        if self.seen {
            for (mn, v) in self.min.iter_mut().zip(q) {
                *mn = (*mn).min(v);
            }
            for (mx, v) in self.max.iter_mut().zip(q) {
                *mx = (*mx).max(v);
            }
        } else {
            self.min = q;
            self.max = q;
            self.seen = true;
        }
    }

    #[must_use]
    pub fn center(&self) -> [f64; 3] {
        if !self.seen {
            return [0.0; 3];
        }
        let c: [f64; 3] = std::array::from_fn(|j| (self.min[j] + self.max[j]) / 2.0);
        c
    }

    /// The largest half-axis (an empty box falls back to 0.5, the fit math stays valid).
    #[must_use]
    pub fn radius(&self) -> f64 {
        if !self.seen {
            return 0.5;
        }
        let half: [f64; 3] = std::array::from_fn(|j| (self.max[j] - self.min[j]) / 2.0);
        half.into_iter().fold(0.0f64, f64::max).max(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::BBox;

    const I4: [[f64; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    #[test]
    fn bbox_center_radius_and_translation_row() {
        let mut b = BBox::new();
        b.push([0.0, 0.0, 0.0], &I4);
        let mut moved = I4;
        moved[3] = [2.0, 0.0, 1.0, 1.0]; // translation lives in row 3 (the row-vector convention)
        b.push([1.0, 1.0, 0.0], &moved);
        assert_eq!(b.center(), [1.5, 0.5, 0.5]);
        assert!((b.radius() - 1.5).abs() < 1e-9);
        assert_eq!(BBox::new().radius(), 0.5); // empty-box fallback
    }
}
