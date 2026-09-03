# visia-render-wgpu 行为契约（SDD）

> 条款格式：`## WGPU-NN: 名称`。双向追溯：`scripts/spec-trace.sh`。

## WGPU-01: instance_create_headless
`create_instance()` 以 `Backends::PRIMARY` 构造 wgpu Instance：不 panic、不依赖 DISPLAY/表面即为通过（本机无 GPU 时构造本身仍合法）。

## WGPU-02: adapter_enumeration_typed
`available_adapters()` 返回 `Vec<wgpu::AdapterInfo>` 类型面：无适配器时空 vec 合法，panic 非法（驱动缺失是可报告状态，不是崩溃理由）。

（WGPU-03..05 随 S4 填充。）

## L2 窗口 smoke（叙述性条款，**不占编号、不入双向 grep**）

`examples/clear.rs` 接受 `--frames N` 自动退出；CI 以 `xvfb-run -a pixi run smoke-clear`
（N=3）断言 exit 0 = winit→surface→帧循环全链路 E2E 通。本机无 DISPLAY 时该层仅在 CI 验证。
