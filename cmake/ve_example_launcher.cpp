// visiaengine example launcher（C-1 裁决=stub 方案）：
// IDE Run 的真身——三件事：chdir 仓根（CWD 合同，7 件 examples 按进程目录解析
// resources/）、显示族 DISPLAY 预检（缺=exit 77，ctest SKIP_RETURN_CODE 约定）、
// execv 转发 argv/env（数据路径参数逐件照旧）。零 example 源码改动。
// C++ 形态：根门面 project 仅 LANGUAGES CXX，禁为 launcher 扩语言（纯度锁）。
// 设计=.omo/plans/cmake-example-targets.md v1.1
#include <cstdio>
#include <cstdlib>
#include <unistd.h>

#ifndef VE_TARGET
#error "VE_TARGET required: 真 example 二进制绝对路径"
#endif
#ifndef VE_REPO
#error "VE_REPO required: 仓根绝对路径"
#endif
#ifndef VE_NEEDS_DISPLAY
#define VE_NEEDS_DISPLAY 0
#endif

int main(int argc, char **argv) {
    if (chdir(VE_REPO) != 0) {
        std::perror("ve-launcher: chdir " VE_REPO);
        return 2;
    }
#if VE_NEEDS_DISPLAY
    const char *disp = std::getenv("DISPLAY");
    if (disp == nullptr || disp[0] == '\0') {
        std::fprintf(stderr,
                     "ve-launcher: 此示例需 X11 窗口（DISPLAY 未设置）。"
                     "桌面直接设置后重跑；CI 用 xvfb-run。（ctest 记为 Skipped）\n");
        return 77;
    }
#endif
    // argv[0] 换成真实例路径，其余原样转发（含数据路径参数）
    char **a = static_cast<char **>(calloc(static_cast<size_t>(argc) + 1, sizeof(char *)));
    if (a == nullptr) {
        std::fputs("ve-launcher: OOM\n", stderr);
        return 1;
    }
    a[0] = const_cast<char *>(VE_TARGET);
    for (int i = 1; i < argc; i++) {
        a[i] = argv[i];
    }
    a[argc] = nullptr;
    execv(VE_TARGET, a);
    std::perror("ve-launcher: execv " VE_TARGET);
    return 127;
}
