# IO-*：文字装载面（visiaengine-io-text）行为契约

> fixture 真源：`resources/data/DejaVuSans.ttf`（宽许可数据资产，许可文本随置同目录；
> 不链入 crate，仅测试/演示消费）。色域无关（本面零色处理——色住调用方 LabelMark）。
> 坐标产物：`LabelQuad.top_left_px=(右, 上)` 相对基线锚（y 向上为正，渲染端 vs_label 翻屏序）。

## IO-07: FontFace 装载契约
`FontFace::from_bytes(bytes)`：空=`FaceError::Empty`；非 TTF/parse 失败=`FaceError::Parse`（**无默认字体**——装载成功才有文字管线，时序门=CAPI-21/22 面消费）。**无内嵌字体**（CJK 全式 15MB 级不可接受=宿主注入，与 load_gltf/load_pcl 同数据形）。`rasterize(ch, size_px)` 透传 fontdue `(Metrics, Vec<u8>)`（位图=w×h 行主序**自上而下**）；域闸 `size_px>0∧有限` 住 IO-08 入口。依赖锁形 `default-features=false, features=["hashbrown"]`（关 SIMD=wasm 实证陷阱 [fontdue #25/#72]；保 hashbrown=no_std 下 std/hashbrown 二选一必居其一 [docs feature-flags]）。

## IO-08: GlyphCache R8 atlas（shelf 装箱）
key=`(char, (size_px·10).round())`（f32 不做键；同字不同号=不同槽，`top=ymin+height` 随字号变）。`get`：命中返同槽（确定性）；未中→rasterize+装箱+置脏；**零覆盖字形（space 类，w0h0）记 advance 不占 atlas 不置脏**。装箱=shelf 游标 + **1px 缝**（线性采样防邻字渗色）；图集满→`get` 内部**全清重烘一次再试**（fontdue 确定性=透明降级，抖而不崩；单字超整图容量=最终 None，调用方跳过该字）。`take_dirty()`=纹理重传信号（取后清）。atlas 边长常数 `GLYPH_ATLAS_PX=512`（R8 256KB，64² 均字 ≈1024 槽；容量上限改数=此一处）。域闸 `size_px≤0/非有限`→None。**降级注记 [ponytail]**：全量 reclaim 非逐字形 LRU——逐字 LRU 待 profile 证实重烘疼再上（换 `reset()` 一处）。

## IO-09: LTR layout（advance 累加 + bearing）
`layout(text, face, cache, size_px) → (Vec<LabelQuad>, pen)`：**逐 `char` 码点序**（组合标记不拆/不分字素簇=档①声明边界，CJK 逐字天然正解，拉丁带音符组合序列为已知粗糙面）。pen 从 0 累加 `advance_width`；quad 的 `top_left_px=[pen_before+xmin, ymin+height]`（**上正**约定）；零覆盖=占 advance 不出 quad；**空串/`size≤0`=(空,0)**。怪字容灾：cache 返 None（超容量怪形）→pen 加经验宽 `size*0.5` 继续排（**不整串弃**）。总宽=含尾 advance 的 pen（MapLibre 同构：居中平移按此宽）。确定性锁：同 cache 同输入两次=逐字段等（测试前缀锁）。
