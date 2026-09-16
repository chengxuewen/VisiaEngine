// template.cpp —— headless 例子·C++ 毛坯骨架（B8：抄文件不继承库；新例 cp 后改主题区，
// 首行 // E8xx_name —— 标题 保三方锁。跑不通即 API 未完成 [E3D:D7]。）
#include <cstdio>
#include <vector>

#include <visiaengine/visiaengine.hpp>

int main(void) {
    // ── 骨架区：RAII 引擎（move-only；析构自动 destroy；no-throw=返回码穿壳）──
    auto eng = visiaengine::Engine::create_headless(160, 120);
    if (!eng.valid()) { puts("create failed"); return 1; }

    // ── 主题区：装载/渲染/readback/断言（参 E802：量值探针 -5 形 + 中心亮度 lum>40）──
    if (eng.render() != VE_OK) { printf("render: %s\n", eng.last_error()); return 1; }
    std::vector<uint8_t> buf(160 * 120 * 4);
    if (eng.readback(buf.data(), buf.size()) != VE_OK) { puts("readback"); return 1; }
    // ── 主题区 end ──
    puts("OK E8xx_template");
    return 0;
}
