#pragma once
#include "model.h"

namespace damage_numbers {
enum class DepthRange { ZeroToOne, MinusOneToOne };
struct View {
    float world_to_clip[16]; // Row-major; supplied by a verified camera binding.
    float x, y, width, height;
    DepthRange depth;
};
struct Point { float x, y; };
inline bool project(Vec3 p, const View& v, Point& out) {
    if (!valid(p) || !finite(v.x) || !finite(v.y) || !finite(v.width) || !finite(v.height) ||
        v.width<=0 || v.height<=0) return false;
    for (float n:v.world_to_clip) if (!finite(n)) return false;
    const float* m=v.world_to_clip;
    const float x=m[0]*p.x+m[1]*p.y+m[2]*p.z+m[3];
    const float y=m[4]*p.x+m[5]*p.y+m[6]*p.z+m[7];
    const float z=m[8]*p.x+m[9]*p.y+m[10]*p.z+m[11];
    const float w=m[12]*p.x+m[13]*p.y+m[14]*p.z+m[15];
    if (!finite(x)||!finite(y)||!finite(z)||!finite(w)||w<=0.00001f ||
        x < -w || x > w || y < -w || y > w || z>w ||
        z < (v.depth==DepthRange::ZeroToOne ? 0 : -w)) return false;
    out={v.x+(x/w+1)*v.width*0.5f,v.y+(1-y/w)*v.height*0.5f};
    return finite(out.x)&&finite(out.y);
}
inline unsigned digits(int32_t value, char (&out)[12]) {
    if (value<=0) { out[0]=0; return 0; }
    char reverse[10]; unsigned n=0;
    do { reverse[n++]=char('0'+value%10); value/=10; } while(value);
    for(unsigned i=0;i<n;++i) out[i]=reverse[n-1-i];
    out[n]=0; return n;
}
struct DrawNumber {
    Point position;
    float pixels, opacity;
    uint32_t color;
    Critical critical;
    HitKey key;
    char text[12];
};

// This produces layout commands only. Native rendering uses outlined text and
// critical lines; host tests can verify the exact same projection/animation.
inline unsigned layout(const Model& model, const View& view, float now,
                       DrawNumber (&out)[128]) {
    if (!finite(now)) return 0;
    unsigned count=0;
    const Settings& s=model.config();
    const float viewport_scale=view.height/720.0f;
    if (!s.enabled || !valid(s) || !finite(viewport_scale) || viewport_scale<=0) return 0;
    for(unsigned i=0;i<model.size();++i) {
        const Event& e=model.at(i).event;
        const float age=now-e.captured_at;
        if (age<0 || age>=s.lifetime || !visible(e,s)) continue;
        Point anchor{};
        if (!project(e.position,view,anchor)) continue;
        DrawNumber d{};
        const float pop=age<0.1f ? 1.0f+0.25f*(1.0f-age/0.1f) : 1.0f;
        d.pixels=24.0f*viewport_scale*s.scale*pop;
        d.opacity=age>s.lifetime-0.25f ? (s.lifetime-age)/0.25f : 1.0f;
        d.color=rgb(tint(e,s)); d.critical=marker(e); d.key=e.key;
        const unsigned n=digits(e.total_damage,d.text);
        const float width=float(n)*d.pixels*0.65f;
        d.position={anchor.x-width*0.5f,anchor.y-8*viewport_scale-age/s.lifetime*30*viewport_scale};
        // Place younger hits above nearby older numbers. Deterministic input
        // order preserves every impact and gives a stable multi-hit pattern.
        for(unsigned attempt=0;attempt<count;++attempt) {
            bool moved=false;
            for(unsigned j=0;j<count;++j) {
                const DrawNumber& other=out[j]; unsigned len=0;
                while(other.text[len])++len;
                const float other_width=float(len)*other.pixels*0.65f;
                if (d.position.x < other.position.x+other_width+4*viewport_scale &&
                    other.position.x < d.position.x+width+4*viewport_scale &&
                    d.position.y < other.position.y+other.pixels+4*viewport_scale &&
                    other.position.y < d.position.y+d.pixels+4*viewport_scale) {
                    d.position.y=other.position.y-d.pixels-4*viewport_scale;
                    moved=true; break;
                }
            }
            if(!moved)break;
        }
        if (d.position.x<view.x || d.position.y<view.y ||
            d.position.x+width>view.x+view.width || d.position.y+d.pixels>view.y+view.height) continue;
        out[count++]=d;
    }
    return count;
}
}
