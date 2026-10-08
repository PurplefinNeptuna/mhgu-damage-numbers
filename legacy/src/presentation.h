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
struct Rect { float left, top, right, bottom; };
struct TextMetrics { float advance; Rect ink; };
using Measure = bool (*)(const char*,float,VisualStyle,TextMetrics&);
inline float low(float a,float b) { return a<b?a:b; }
inline float high(float a,float b) { return a>b?a:b; }
inline Rect shifted(Rect r,Point p) { return {r.left+p.x,r.top+p.y,r.right+p.x,r.bottom+p.y}; }
inline Rect united(Rect a,Rect b) { return {low(a.left,b.left),low(a.top,b.top),high(a.right,b.right),high(a.bottom,b.bottom)}; }
inline bool valid(Rect r) { return finite(r.left)&&finite(r.top)&&finite(r.right)&&finite(r.bottom)&&r.right>r.left&&r.bottom>r.top; }
inline float overlap(Rect a,Rect b,float margin) {
    return high(0,low(a.right,b.right)+margin-high(a.left,b.left))*
           high(0,low(a.bottom,b.bottom)+margin-high(a.top,b.top));
}
inline bool inside(Rect r,const View& v) { return r.left>=v.x&&r.top>=v.y&&r.right<=v.x+v.width&&r.bottom<=v.y+v.height; }
static constexpr float base_pixels_720p=24.0f;
static constexpr float rise_base_pixels_720p=20.0f;
static constexpr float rise_glyph_width=1.08f;
static constexpr float rise_slash_height=.75f; // User-selected total viewport extent.
static constexpr float wilds_crit_scale=1.5f;
static constexpr float wilds_star_height=.31612110f*wilds_crit_scale;
inline Point glyph_origin(Point p,VisualStyle style) {
    return style!=VisualStyle::Classic?Point{float(int(p.x)),float(int(p.y))}:p;
}
inline uint32_t star_rgb(Critical c) { return c==Critical::Positive?0xf1689b:0x68b9ff; }
struct Slash {
    Point origin;
    float slope, extent, width, opacity;
    Rect clip;
};
// Stable, affinity-independent direction; no frame seed or game pointer lookup.
inline float slash_slope(const HitKey& key) {
    uint32_t h=key.serial*0x9e3779b9u ^ key.epoch*0x85ebca6bu ^ key.target ^ key.application*0xc2b2ae35u;
    h^=h>>16;h*=0x7feb352du;h^=h>>15;
    return (float(h&65535u)/65535.0f-.5f)*.5f;
}
inline Slash rise_slash(Point origin,const HitKey& key,float age,const View& view,float fraction=rise_slash_height) {
    // A UI approximation: expand for 50ms, thin/fade over 150ms, then expire.
    const float grow=low(1,high(0,age/.05f));
    const float fade=low(1,high(0,(age-.05f)/.15f));
    return {origin,slash_slope(key),view.height*fraction*(.72f+.28f*grow),
        view.height/720.0f*(24+24*grow)*(1-.8f*fade),
        age<0?0:(.35f+.65f*grow)*(1-fade)*(1-fade),
        {view.x,view.y,view.x+view.width,view.y+view.height}};
}
// The registered hit is the center; equal upward/downward reach, no ground cutoff.
inline void slash_quad(const Slash& s,Point (&p)[4]) {
    const float top=s.extent*.5f,bottom=s.extent*.5f,half=s.width*.5f;
    p[0]={s.origin.x+s.slope*top-half,s.origin.y-top};
    p[1]={s.origin.x+s.slope*top+half,s.origin.y-top};
    p[2]={s.origin.x-s.slope*bottom+half,s.origin.y+bottom};
    p[3]={s.origin.x-s.slope*bottom-half,s.origin.y+bottom};
}
struct DrawNumber {
    Point position, offset, star;
    Slash slash;
    float pixels, opacity, advance, unit, classic_stroke;
    Rect ink, solid, reserve;
    uint32_t color;
    Critical critical;
    VisualStyle style;
    HitKey key;
    char text[12];
};
inline Rect marker_bounds(Rect ink,float pixels,float unit,Critical critical,VisualStyle style) {
    if (critical==Critical::None || style==VisualStyle::Rise) return ink;
    const float cx=(ink.left+ink.right)*0.5f;
    const float halfh=style==VisualStyle::Wilds?pixels*wilds_star_height*.5f:2*unit;
    const float y=critical==Critical::Positive?ink.top-2*unit-halfh:ink.bottom+2*unit+halfh;
    const float halfw=style==VisualStyle::Wilds?pixels*.38078224f*wilds_crit_scale:(ink.right-ink.left)*.5f;
    return united(ink,{cx-halfw,y-halfh,cx+halfw,y+halfh});
}
inline Rect solid_bounds(Rect ink,Point pos,float pixels,float advance,float unit,Critical crit,VisualStyle style) {
    if(style==VisualStyle::Wilds)return marker_bounds(ink,pixels,unit,crit,style);
    if(crit==Critical::None || style==VisualStyle::Rise)return ink;
    const float stroke=1.25f*unit;
    const float y=crit==Critical::Positive?pos.y-2*stroke:pos.y+pixels+2*stroke;
    return united(ink,{pos.x-2*stroke,y-2*stroke,pos.x+advance+2*stroke,y+2*stroke});
}
// Wilds appearance uses the accepted open-font stamp/hold/exit timing.
// Classic retains its original continuous drift.
inline float peak_scale(VisualStyle style) {
    if(style==VisualStyle::Rise)return 1.2f;
    if(style==VisualStyle::Wilds)return 1.2f;
    return 1.25f;
}
struct NumberMotion { float scale, opacity, rise; };
inline NumberMotion number_motion(float age,float lifetime,VisualStyle style) {
    if(style==VisualStyle::Rise) {
        // Scale the authored 9/10-frame entrance/exit to a bounded UI lifetime.
        const float entrance=low(.3f,lifetime*.3f),exit_time=low(10.0f/30,lifetime*.35f);
        const float t=low(1,high(0,age/entrance));
        const float scale=t<2.0f/9 ? .64f+(.56f*t*9/2) :
            t<7.0f/9 ? 1.2f-.32f*(t*9-2)/5 : .88f+.12f*(t*9-7)/2;
        const float exit=low(1,high(0,(age-(lifetime-exit_time))/exit_time));
        const float kick=low(1,exit*10);
        const float rise=exit<=.1f?-4*kick:-4+54*(exit-.1f)/.9f;
        return {scale,1-exit,rise};
    }
    if(style==VisualStyle::Wilds) {
        const float settle=low(1,high(0,age/.2f));
        const float stamp=1-settle;
        const float exit=low(1,high(0,(age-(lifetime-.2f))/.2f));
        return {1+.2f*stamp*stamp,1-exit,30*exit};
    }
    return {age<.1f?1+.25f*(1-age/.1f):1,
            age>lifetime-.25f?(lifetime-age)/.25f:1,age/lifetime*30};
}
struct PlacementState {
    struct Entry { HitKey key; Point offset; bool live; } entries[128]{};
    float x=0,y=0,width=0,height=0,scale=0;
    VisualStyle style=VisualStyle::Wilds;
    uint32_t epoch=0;
    void reset() { for(auto& e:entries)e.live=false; width=height=0; }
};
// Host fallback only. Native always supplies actual ImGui glyph bounds.
inline bool approximate_measure(const char* text,float pixels,VisualStyle,TextMetrics& m) {
    unsigned n=0;while(text[n])++n;
    m={float(n)*pixels*.55f,{0,pixels*.15f,float(n)*pixels*.55f,pixels*.85f}};
    return n!=0;
}
inline unsigned layout(const Model& model,const View& view,float now,DrawNumber (&out)[128],
                       PlacementState& cache,Measure measure=approximate_measure) {
    const Settings& s=model.config();
    // Scale marker gaps and collision margins with the selected style base.
    const float base=(s.style==VisualStyle::Rise?rise_base_pixels_720p:base_pixels_720p)*view.height/720.0f*s.scale,unit=base/24.0f;
    if (!finite(now)||!s.enabled||!valid(s)||!finite(unit)||unit<=0||!model.size()) { cache.reset();return 0; }
    const uint32_t epoch=model.at(0).event.key.epoch;
    if(cache.x!=view.x||cache.y!=view.y||cache.width!=view.width||cache.height!=view.height||
       cache.scale!=s.scale||cache.style!=s.style||cache.epoch!=epoch) cache.reset();
    cache.x=view.x;cache.y=view.y;cache.width=view.width;cache.height=view.height;
    cache.scale=s.scale;cache.style=s.style;cache.epoch=epoch;
    for(auto& entry:cache.entries) if(entry.live) {
        bool found=false;
        for(unsigned i=0;i<model.size();++i) {
            const Event& e=model.at(i).event;
            found=found || (entry.key==e.key && now>=e.captured_at && now-e.captured_at<s.lifetime);
        }
        if(!found)entry.live=false;
    }
    static constexpr Point candidates[17]={{0,0},{-.8f,0},{.8f,0},{0,-.5f},{0,.5f},
        {-.8f,-.5f},{.8f,-.5f},{-.8f,.5f},{.8f,.5f},{-1.6f,0},{1.6f,0},
        {-1.6f,-1},{1.6f,-1},{-1.6f,1},{1.6f,1},{0,-1},{0,1}};
    unsigned count=0;
    for(unsigned i=0;i<model.size();++i) {
        const Event& e=model.at(i).event;const float age=now-e.captured_at;
        if(age<0||age>=s.lifetime||!visible(e,s))continue;
        Point anchor{};if(!project(e.position,view,anchor))continue;
        DrawNumber d{};digits(e.total_damage,d.text);
        const NumberMotion motion=number_motion(age,s.lifetime,s.style);
        const float settled=base*(s.style==VisualStyle::Rise&&tint(e,s)==Tint::Weak?1.4f:1);
        const float peak_pixels=settled*peak_scale(s.style);
        d.pixels=settled*motion.scale;d.unit=unit;d.classic_stroke=1.25f*view.height/720.0f;
        d.opacity=motion.opacity;
        if(s.style==VisualStyle::Rise) d.slash=rise_slash(anchor,e.key,age,view);
        d.style=s.style;d.color=rgb(tint(e,s),s.style);d.critical=marker(e);d.key=e.key;
        TextMetrics current{},peak{};
        if(!measure(d.text,d.pixels,s.style,current)||!measure(d.text,peak_pixels,s.style,peak)||
           !valid(current.ink)||!valid(peak.ink)||!finite(current.advance)||!finite(peak.advance))continue;
        const float cy=anchor.y-8*view.height/720.0f-motion.rise*view.height/720.0f;
        d.position={anchor.x-current.advance*.5f,cy};d.advance=current.advance;
        d.ink=shifted(current.ink,d.position);
        Rect reserved=shifted(peak.ink,{anchor.x-peak.advance*.5f,cy});
        reserved=solid_bounds(reserved,{anchor.x-peak.advance*.5f,cy},peak_pixels,peak.advance,
            s.style==VisualStyle::Wilds?unit:view.height/720.0f,d.critical,s.style);
        const float shadow=s.style==VisualStyle::Rise?peak_pixels*.13f:s.style==VisualStyle::Wilds?unit:1.25f*view.height/720.0f;
        const float snap_pad=s.style!=VisualStyle::Classic?1.0f:0.0f;
        reserved={reserved.left-shadow-snap_pad,reserved.top-shadow-snap_pad,reserved.right+shadow+snap_pad,reserved.bottom+shadow+snap_pad};
        PlacementState::Entry* entry=nullptr;
        for(auto& item:cache.entries)if(item.live&&item.key==d.key){entry=&item;break;}
        if(!entry) {
            bool selected=false;float best_overlap=0,best_distance=0;Point best{};
            for(const Point candidate:candidates) {
                const Point offset={candidate.x*base,candidate.y*base};const Rect r=shifted(reserved,offset);
                if(!inside(r,view))continue;
                float area=0;for(unsigned j=0;j<count;++j)area+=overlap(r,out[j].reserve,unit);
                const float distance=offset.x*offset.x+offset.y*offset.y;
                if(!selected||area<best_overlap||(area==best_overlap&&distance<best_distance)) {
                    selected=true;best_overlap=area;best_distance=distance;best=offset;
                }
            }
            if(!selected)continue;
            for(auto& item:cache.entries)if(!item.live){entry=&item;break;}
            if(!entry)continue;
            *entry={d.key,best,true};
        }
        d.offset=entry->offset;d.position.x+=d.offset.x;d.position.y+=d.offset.y;
        // ImGui truncates text origins before drawing. Center stars on those
        // same visible glyph bounds rather than the pre-rounded text origin.
        d.position=glyph_origin(d.position,d.style);
        d.ink=shifted(current.ink,d.position);d.reserve=shifted(reserved,d.offset);
        d.solid=solid_bounds(d.ink,d.position,d.pixels,d.advance,
            s.style==VisualStyle::Wilds?unit:view.height/720.0f,d.critical,s.style);
        d.solid={d.solid.left-shadow,d.solid.top-shadow,d.solid.right+shadow,d.solid.bottom+shadow};
        if(!inside(d.solid,view))continue;
        const float halfh=s.style==VisualStyle::Wilds?d.pixels*wilds_star_height*.5f:2*unit;
        d.star={(d.ink.left+d.ink.right)*.5f,d.critical==Critical::Positive?
                d.ink.top-2*unit-halfh:d.ink.bottom+2*unit+halfh};
        out[count++]=d;
    }
    return count;
}
// Convenience for stateless host callers; native retains its cache across frames.
inline unsigned layout(const Model& m,const View& v,float now,DrawNumber (&out)[128]) {
    PlacementState state{};return layout(m,v,now,out,state);
}
}
