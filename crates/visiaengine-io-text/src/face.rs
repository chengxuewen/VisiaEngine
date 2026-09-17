//! IO-07：font face 装载契约（非 TTF/parse 失败=Err；size 域>0）。
//! 骨架占位（N1 填实体）：包 fontdue::Font。

/// 宿主注入的字体字节的引擎侧包装（无默认字体；装载成功才激活文字管线）。
#[derive(Debug)]
pub struct FontFace {
    _private: (),
}

/// N0 冒烟：fontdue feature 组合可编（真实包装在 N1）。
#[must_use]
pub fn fontdue_smoke(bytes: &[u8]) -> bool {
    fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).is_ok()
}
