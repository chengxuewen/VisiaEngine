// C2 ctest 探针：伞合同全链闭环自证（解析→cargo-step→find_package→链接→运行），
// 不依赖 Qt。跑不通即消费者合同未兑现（[E3D:D7] 精神的本仓内生版）。
#include <cstdio>
#include "visiaengine.h"

int main(void) {
    const uint32_t abi = visiaengine_abi_version();
    if ((abi >> 16) != 1u) {
        (void)printf("FAIL abi major=%u (expect 1)\n", abi >> 16);
        return 1;
    }
    (void)printf("OK capi probe abi=0x%x\n", abi);
    return 0;
}
