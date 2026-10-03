#include "feedback.h"
extern "C" uint32_t dn_feedback(const uint32_t* v) {
    damage_numbers::FeedbackInput in{};
    in.first = {v[0] != 0, uint8_t(v[1]), v[2]};
    in.second = {v[3] != 0, uint8_t(v[4]), v[5]};
    in.minds_eye = (v[6] & 1) != 0;
    in.combined_minds_eye = (v[6] & 14) != 0;
    in.blind_eye = (v[6] & 16) != 0;
    in.attack_counter = uint16_t(v[7]);
    in.status_result = int(v[8]);
    in.status_prevents_bounce = int(v[9]);
    in.stance_result = int(v[10]);
    in.weapon_level = uint8_t(v[11]);
    in.move_prevents_bounce_8 = v[12] != 0;
    in.move_prevents_bounce_14 = v[13] != 0;
    return damage_numbers::resolved_feedback(in);
}
