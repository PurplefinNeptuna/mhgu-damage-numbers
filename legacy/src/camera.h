// Convert copied native camera matrices into the convention used by project().

#pragma once
#include "presentation.h"

namespace damage_numbers {
// Native matrices multiply row vectors: world * view * projection. Transpose
// that product for presentation's column-vector world_to_clip convention.
// This accepts copied engine matrices; it does not reconstruct a camera/FOV.
inline bool from_native_camera(const float (&view)[16], const float (&projection)[16], float left,
                               float top, float right, float bottom, View &out) {
    if (!finite(left) || !finite(top) || !finite(right) || !finite(bottom) || right <= left ||
        bottom <= top)
        return false;
    for (float n : view)
        if (!finite(n))
            return false;
    for (float n : projection)
        if (!finite(n))
            return false;
    View result{};
    for (unsigned row = 0; row < 4; ++row)
        for (unsigned col = 0; col < 4; ++col) {
            float n = 0;
            for (unsigned k = 0; k < 4; ++k)
                n += view[col * 4 + k] * projection[k * 4 + row];
            if (!finite(n))
                return false;
            result.world_to_clip[row * 4 + col] = n;
        }
    result.x = left;
    result.y = top;
    result.width = right - left;
    result.height = bottom - top;
    result.depth = DepthRange::ZeroToOne;
    out = result;
    return true;
}
} // namespace damage_numbers
