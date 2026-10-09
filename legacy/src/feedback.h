#pragma once
#include <cstdint>

namespace damage_numbers {

// Mirror of main+0x2a87a8's resolved weapon feedback. This is a copied-value
// research helper, not a per-impact bounce binding. The native getter combines
// both active attacks; its result must not mark every impact as bounced.
struct FeedbackAttack {
    bool active = false;
    uint8_t effect = 0xff;
    uint32_t flags = 0;
};

struct FeedbackInput {
    FeedbackAttack first{}, second{};
    bool minds_eye = false, combined_minds_eye = false, blind_eye = false;
    uint16_t attack_counter = 0;
    int status_result = 0, status_prevents_bounce = 0, stance_result = 0;
    uint8_t weapon_level = 0;
    bool move_prevents_bounce_8 = false, move_prevents_bounce_14 = false;
};

inline uint8_t resolved_feedback(const FeedbackInput &in) {
    uint8_t effect = 0xff;
    uint32_t flags = 0;
    if (in.first.active && in.first.effect != 0xff) {
        effect = in.first.effect;
        flags = in.first.flags;
    }
    if (in.second.active && in.second.effect != 0xff &&
        (effect == 0xff || effect < in.second.effect)) {
        effect = in.second.effect;
        flags |= in.second.flags;
    }
    const bool force = (flags & (1u << 18)) != 0;
    if (!force && effect == 0 && (in.minds_eye || in.combined_minds_eye))
        effect = 2;
    if (!in.combined_minds_eye && in.blind_eye && effect == 2)
        effect = in.attack_counter % 3 == 0 ? 0 : 2;
    if (effect == 0 && in.status_result == 1 && in.status_prevents_bounce != 0)
        effect = 2;
    if (effect == 0 && !force && in.stance_result == 1 && in.weapon_level > 1)
        effect = 2;
    else if (effect == 0 &&
             (in.move_prevents_bounce_8 || in.move_prevents_bounce_14 || (flags & (1u << 27))))
        effect = 2;
    return effect;
}

} // namespace damage_numbers
