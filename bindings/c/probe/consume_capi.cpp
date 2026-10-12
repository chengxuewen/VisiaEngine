// C2 ctest probe: full-chain closed-loop self-proof of the umbrella contract (parse -> cargo-step -> find_package -> link -> run),
// independent of Qt. If it does not run through, the consumer contract is unfulfilled (the in-repo native version of the [E3D:D7] spirit).
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
