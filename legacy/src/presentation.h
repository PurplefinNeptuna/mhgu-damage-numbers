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
static constexpr float base_pixels_720p=20.0f;
static constexpr float wilds_star_height=.31612110f;
inline Point glyph_origin(Point p,VisualStyle style) {
    return style==VisualStyle::Wilds?Point{float(int(p.x)),float(int(p.y))}:p;
}
inline uint32_t star_rgb(Critical c) { return c==Critical::Positive?0xf1689b:0x68b9ff; }
struct DrawNumber {
    Point position, offset, star;
    float pixels, opacity, advance, unit, classic_stroke;
    Rect ink, solid, reserve;
    uint32_t color;
    Critical critical;
    VisualStyle style;
    HitKey key;
    char text[12];
};
inline Rect marker_bounds(Rect ink,float pixels,float unit,Critical critical,VisualStyle style) {
    if (critical==Critical::None) return ink;
    const float cx=(ink.left+ink.right)*0.5f;
    const float halfh=style==VisualStyle::Wilds?pixels*wilds_star_height*.5f:2*unit;
    const float y=critical==Critical::Positive?ink.top-2*unit-halfh:ink.bottom+2*unit+halfh;
    const float halfw=style==VisualStyle::Wilds?pixels*.38078224f:(ink.right-ink.left)*.5f;
    return united(ink,{cx-halfw,y-halfh,cx+halfw,y+halfh});
}
inline Rect solid_bounds(Rect ink,Point pos,float pixels,float advance,float unit,Critical crit,VisualStyle style) {
    if(style==VisualStyle::Wilds)return marker_bounds(ink,pixels,unit,crit,style);
    if(crit==Critical::None)return ink;
    const float stroke=1.25f*unit;
    const float y=crit==Critical::Positive?pos.y-2*stroke:pos.y+pixels+2*stroke;
    return united(ink,{pos.x-2*stroke,y-2*stroke,pos.x+advance+2*stroke,y+2*stroke});
}
// Wilds appearance uses the accepted open-font stamp/hold/exit timing.
// Classic retains its original continuous drift.
inline float peak_scale(VisualStyle style) {
    if(style==VisualStyle::Wilds)return 1.2f;
    return 1.25f;
}
struct NumberMotion { float scale, opacity, rise; };
inline NumberMotion number_motion(float age,float lifetime,VisualStyle style) {
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
    // Keep the selected 20px preview's marker gap and collision margin.
    const float base=base_pixels_720p*view.height/720.0f*s.scale,unit=base/24.0f;
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
        const float peak_pixels=base*peak_scale(s.style);
        d.pixels=base*motion.scale;d.unit=unit;d.classic_stroke=1.25f*view.height/720.0f;
        d.opacity=motion.opacity;
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
        const float shadow=s.style==VisualStyle::Wilds?unit:1.25f*view.height/720.0f;
        const float snap_pad=s.style==VisualStyle::Wilds?1.0f:0.0f;
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
