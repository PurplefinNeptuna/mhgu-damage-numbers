#pragma once
#include "camera.h"
#include "outcomes.h"
#include "sources.h"

namespace damage_numbers {
// Native rectangles are framebuffer pixels. The preview accepts a full-screen
// viewport with matching aspect only, then projects into Forge's swapchain size.
// Offset/split viewports need an independently verified framebuffer binding.
inline bool preview_camera(const float (&view)[16], const float (&projection)[16],
                           const int32_t (&rect)[4], float width, float height, View& out) {
    if (rect[0]!=0 || rect[1]!=0 || rect[2]<=0 || rect[3]<=0 ||
        !finite(width) || !finite(height) || width<=0 || height<=0) return false;
    const float native_aspect=float(rect[2])/float(rect[3]);
    const float display_aspect=width/height;
    if (!finite(native_aspect) || !finite(display_aspect) ||
        fabsf(native_aspect/display_aspect-1)>0.02f) return false;
    return from_native_camera(view,projection,0,0,width,height,out);
}
inline bool preview_seconds(uint64_t tick, uint64_t origin, uint32_t frequency, float& out) {
    if (!frequency || tick<origin) return false;
    out=float(tick-origin)/float(frequency);
    return finite(out);
}
inline bool preview_fresh(uint64_t tick, uint64_t now, uint32_t frequency, float limit) {
    float age=0;
    return tick && preview_seconds(now,tick,frequency,age) && age<limit;
}
// Ordinary owned attacks; optional sources are classified independently.
inline bool preview_source(uint32_t tags, Owner owner) {
    if (owner!=Owner::LocalPlayer && owner!=Owner::LocalPalico) return false;
    Source source=Source::Attack;
    return source_outcome(tags,owner,source) && source==Source::Attack;
}
inline uint32_t imgui_color(uint32_t rgb, float opacity) {
    if (!finite(opacity)) return 0;
    opacity=opacity<0 ? 0 : opacity>1 ? 1 : opacity;
    return ((rgb>>16)&255) | (rgb&0xff00) | ((rgb&255)<<16) |
           (uint32_t(opacity*255+0.5f)<<24);
}
}
