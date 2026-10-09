// Expose the native-camera adapter through a C ABI for host/ARM probe callers.

#include "camera.h"
#include <cstring>

extern "C" int dn_camera_project(const float *native_view, const float *native_projection,
                                 const float *point, float *out) {
    float v[16], p[16];
    std::memcpy(v, native_view, sizeof(v));
    std::memcpy(p, native_projection, sizeof(p));
    damage_numbers::View camera{};
    damage_numbers::Point projected{};
    if (!damage_numbers::from_native_camera(v, p, 0, 0, 1280, 720, camera) ||
        !damage_numbers::project({point[0], point[1], point[2]}, camera, projected))
        return 0;
    out[0] = projected.x;
    out[1] = projected.y;
    return 1;
}
