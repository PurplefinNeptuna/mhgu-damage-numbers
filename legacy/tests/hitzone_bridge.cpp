// Expose saved-register melee HZV extraction for ARM fixture comparisons.

#include "hitzone.h"

extern "C" int32_t dn_melee_hitzone(const unsigned char *raw) {
    damage_numbers::RawFrame frame;
    std::memcpy(&frame, raw, sizeof(frame));
    return damage_numbers::melee_hitzone(frame);
}
