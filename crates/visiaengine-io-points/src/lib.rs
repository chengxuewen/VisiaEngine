//! # visiaengine-io-points
//! 点云（PLY ascii/binary_little_endian）→ CPU 侧数据。零渲染依赖（io-* 族纪律，架构②）。
//! 行为契约：`docs/sdd/io-points.md`（IO-01..06）。M2 RED 骨架：类型齐、解析桩恒 Err（行为必红）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use thiserror::Error;
use visiaengine_core::AttrSet;

/// IO-05 容量守卫（本带唯一点数门；add_points 不受——CAPI-18 宿主自带账）。
pub const MAX_POINTS_CAP: u64 = 4_000_000;

/// IO-03 四类分型报告（重复点不成类=无 dup 字段，IO-03 注记）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PclReport {
    pub dropped_non_finite: u32,
    pub dropped_out_of_domain: u32,
    pub dropped_unsupported: u32,
    pub truncated_points: u32,
    pub kept: u64,
}

/// IO-06 产物（origin-local f32 + f64 origin；色=sRGB 宿主面；meta=云级单行 AttrSet）。
#[derive(Clone, Debug)]
pub struct PclCloud {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub radius_px: f32,
    pub origin: [f64; 3],
    pub meta: AttrSet,
    pub report: PclReport,
}

#[derive(Error, Debug)]
pub enum PclError {
    #[error("非 PLY/头非法: {0}")]
    Malformed(String),
    #[error("格式不支持（v0=ascii+bin_le；be/其余=拒）: {0}")]
    UnsupportedFormat(String),
    #[error("截断且 FastFail: 声明 {declared} 完整 {complete}")]
    Truncated { declared: u64, complete: u64 },
    #[error("超容量帽: declared {declared} > cap {cap}")]
    OverCapacity { declared: u64, cap: u64 },
    #[error("脏数据且 FastFail: {0} 点被丢弃/降级")]
    Dirty(u32),
}

/// IO-01..05 统一入口（bytes 形；path 便捷口在 engine 侧）。lenient=false=FastFail。
/// RED 桩：一切 Err——类型面即契约，行为在 GREEN 片转绿。
pub fn parse_pcl(bytes: &[u8], lenient: bool) -> Result<PclCloud, PclError> {
    let _ = (bytes, lenient);
    Err(PclError::Malformed("RED 桩：解析未实装".into()))
}
