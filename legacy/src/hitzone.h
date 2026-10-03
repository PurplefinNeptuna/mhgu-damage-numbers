#pragma once
#include "gateway.h"
#include <cmath>
#include <cstring>

namespace damage_numbers {
// At main+0x16dd7c, r0 selects blunt (1) or cut (0). s20/s22 contain
// adjusted HZVs before attack-type weighting, not the weighted damage factor.
// The adapter must additionally verify the enclosing monster/HZV call scope.
inline int16_t melee_hitzone(const RawFrame& frame) {
    if (frame.r[0] > 1) return -1;
    const uint64_t bits = frame.low_d[frame.r[0] ? 11 : 10];
    const uint32_t scalar = uint32_t(bits);
    float value;
    std::memcpy(&value, &scalar, sizeof(value));
    return std::isfinite(value) && value >= 0 && value <= 32767 ? int16_t(value) : -1;
}
} // namespace damage_numbers
