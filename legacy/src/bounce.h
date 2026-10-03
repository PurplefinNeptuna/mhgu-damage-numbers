#pragma once
#include "model.h"

namespace damage_numbers {
struct BounceAttack { uint32_t identity,flags; uint8_t active,effect,critical; };
// Native feedback already applied Mind's Eye and move/stance exceptions.
// Match only a single active embedded hunter attack to the latest finalized
// contribution on that native thread. Two active attacks are ambiguous.
inline bool bounce_match(uint8_t resolved,const BounceAttack (&attacks)[2],
                         uint32_t identity,uint32_t flags,uint8_t critical,
                         uint8_t raw_effect,uint32_t tags,Owner owner) {
    if (resolved!=0 || raw_effect!=0 || owner!=Owner::LocalPlayer ||
        uint8_t(tags)!=0 || uint8_t(tags>>8)!=1) return false;
    const bool first=attacks[0].active!=0, second=attacks[1].active!=0;
    if (first==second) return false;
    const BounceAttack& attack=attacks[first ? 0 : 1];
    return identity==attack.identity && flags==attack.flags && critical==attack.critical && attack.effect==0;
}
}
