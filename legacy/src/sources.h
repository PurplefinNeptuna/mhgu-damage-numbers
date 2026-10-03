#pragma once
#include "model.h"

namespace damage_numbers {
// Source tags follow the native collision dispatch: subtype 1 is hunter,
// subtype 2 monster, subtype 4 palico. Subtype 8 takes the generic impact
// path, including the observed palico barrel family. Unknown families stay
// diagnostic; they must not turn remote weapon projectiles into objects.
inline bool source_outcome(uint32_t tags,Owner owner,Source& out) {
    const uint8_t primary=uint8_t(tags), subtype=uint8_t(tags>>8), secondary=uint8_t(tags>>16);
    if ((primary==0 || primary==3) && owner!=Owner::Unknown) { out=Source::Attack; return true; }
    if (primary==2 && ((subtype==1 && secondary==0) || (subtype==4 && secondary==3)) &&
        owner!=Owner::Unknown) { out=Source::Attack; return true; }
    if (primary==2 && subtype==8 && secondary==3 && owner!=Owner::Unknown) {
        out=Source::Attack; return true; // Palico support bombs remain owned attacks.
    }
    if (subtype==2 && (primary==1 || (primary==2 && secondary==1))) {
        out=Source::Monster; return true;
    }
    if (primary==2 && subtype==8 && secondary==0 && owner!=Owner::Unknown) {
        out=Source::Object; return true;
    }
    return false;
}
}
