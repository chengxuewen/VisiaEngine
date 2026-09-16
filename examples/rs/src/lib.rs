//! 例子注册壳：lib 面只放跨例共享小件（BBox 取景拟合）；例子本体见 ../E*_*.rs。
//! 「空 lib 消灭 examples-only 条件分支」的出生预置裁决（v1.3 F4）即此文件；现承载共用件非意外。

/// 世界系 AABB（行向量约定 `[p,1]·M`，平移在第 3 行——io-gltf/D7 同构）。
/// 用途：窗口例装载期取景拟合——E201 灰屏案的根治（历史教训：交互相机参数从未被像素验证）。
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

    /// 点经 world 4x4（行向量）变换后逐轴并入。
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

    /// 最大半轴（空盒回退 0.5，拟合系数恒安全）。
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
        moved[3] = [2.0, 0.0, 1.0, 1.0]; // 平移住第 3 行（行向量约定）
        b.push([1.0, 1.0, 0.0], &moved);
        assert_eq!(b.center(), [1.5, 0.5, 0.5]);
        assert!((b.radius() - 1.5).abs() < 1e-9);
        assert_eq!(BBox::new().radius(), 0.5); // 空盒回退
    }
}
