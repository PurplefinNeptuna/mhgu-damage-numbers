// Translate finalized native damage/critical outcomes into display policy.

#pragma once
#include "model.h"

namespace damage_numbers {
// Use after the verified monster-HP contribution path; status/stagger values
// never enter this adapter. Explicitly reject the native healing attack type.
inline bool damage_outcome(uint8_t native_damage_type, int32_t contribution) {
    return native_damage_type != 0x1d && contribution > 0;
}

inline bool critical_outcome(uint8_t native_flag, Critical &out) {
    switch (native_flag) {
        case 0:
            out = Critical::None;
            return true;
        case 1:
            out = Critical::Negative;
            return true;
        case 2:
            out = Critical::Positive;
            return true;
        default:
            return false;
    }
}
} // namespace damage_numbers
