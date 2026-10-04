#include "cosmetics.h"
#include "preview.h"
#include <forge/log.h>
#include <stdarg.h>
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include <cstddef>
#include "wilds_font_data.h"
namespace damage_numbers {
static void* font_context;
static ImFont* wilds_font;
static bool font_warning;
// The fixture reads compiler-derived ABI offsets, then checks actual ARM calls.
extern "C" __attribute__((used)) const uint32_t dn_font_abi[]={
    offsetof(ImGuiIO,Fonts),offsetof(ImFontConfig,FontDataOwnedByAtlas),
    offsetof(ImFontBaked,Size),offsetof(ImFontGlyph,AdvanceX),offsetof(ImFontGlyph,X0)};
static_assert(sizeof(ImWchar)==2 && sizeof(ImFontGlyph)==44);
static_assert(offsetof(ImGuiIO,Fonts)==44 && offsetof(ImFontConfig,FontDataOwnedByAtlas)==48);
static_assert(offsetof(ImFontBaked,Size)==16 && offsetof(ImFontGlyph,AdvanceX)==4 && offsetof(ImFontGlyph,X0)==8);
void cosmetics_init(void* context) {
    if(font_context==context)return;
    font_context=context;wilds_font=nullptr;
    ImGuiIO* io=igGetIO_Nil();
    if(io&&io->Fonts) {
        ImFontConfig* config=ImFontConfig_ImFontConfig();
        if(config) {
            config->FontDataOwnedByAtlas=false;
            static const ImWchar ranges[]={32,32,48,57,63,63,0};
            wilds_font=ImFontAtlas_AddFontFromMemoryTTF(io->Fonts,const_cast<unsigned char*>(wilds_font_data),
                sizeof(wilds_font_data),24*wilds_font_scale,config,ranges);
            ImFontConfig_destroy(config);
        }
    }
    if(!wilds_font&&!font_warning) {
        font_warning=true;
        forge_log_info("[damage-numbers] Wilds digit font unavailable; using Forge font with Wilds colors/stars");
    }
}
static ImFont* selected_font(VisualStyle style) { return style==VisualStyle::Wilds&&wilds_font?wilds_font:igGetFont(); }
bool cosmetics_measure(const char* text,float pixels,VisualStyle style,TextMetrics& out) {
    ImFont* font=selected_font(style);if(!font)return false;
    if(style==VisualStyle::Wilds&&wilds_font)pixels*=wilds_font_scale;
    ImFontBaked* baked=ImFont_GetFontBaked(font,pixels,-1);if(!baked||baked->Size<=0)return false;
    const float ratio=pixels/baked->Size;float advance=0;Rect ink{};bool found=false;
    for(const char* p=text;*p;++p) {
        ImFontGlyph* glyph=ImFontBaked_FindGlyph(baked,ImWchar(*p));if(!glyph)return false;
        Rect r={advance+glyph->X0*ratio,glyph->Y0*ratio,advance+glyph->X1*ratio,glyph->Y1*ratio};
        if(glyph->Visible&&valid(r)){ink=found?united(ink,r):r;found=true;}
        advance+=glyph->AdvanceX*ratio;
    }
    out={advance,ink};return found&&finite(advance)&&advance>0;
}
static void star(ImDrawList* draw,Point center,float width,float height,uint32_t color) {
    const float x=width*.5f,y=height*.5f;
    const Point ring[8]={{-x,0},{-x*.10f,-y*.24f},{0,-y},{x*.10f,-y*.24f},
                        {x,0},{x*.10f,y*.24f},{0,y},{-x*.10f,y*.24f}};
    for(unsigned i=0;i<8;++i) {
        const Point a=ring[i],b=ring[(i+1)%8];
        ImDrawList_AddTriangleFilled(draw,{center.x,center.y},{center.x+a.x,center.y+a.y},
                                    {center.x+b.x,center.y+b.y},color);
    }
}
void cosmetics_draw(void* list,const DrawNumber& n) {
    auto* draw=static_cast<ImDrawList*>(list);ImFont* font=selected_font(n.style);if(!draw||!font)return;
    const float pixels=n.style==VisualStyle::Wilds&&wilds_font?n.pixels*wilds_font_scale:n.pixels;
    auto text=[&](float x,float y,uint32_t color) {
        ImDrawList_AddText_FontPtr(draw,font,pixels,{n.position.x+x,n.position.y+y},color,n.text,nullptr,0,nullptr);
    };
    const uint32_t color=imgui_color(n.color,n.opacity);
    if(n.style==VisualStyle::Classic) {
        const float stroke=n.classic_stroke;const uint32_t edge=imgui_color(0,n.opacity);
        for(int x=-1;x<=1;++x)for(int y=-1;y<=1;++y)if(x||y)text(x*stroke,y*stroke,edge);
        text(0,0,color);
        if(n.critical!=Critical::None) {
            const float y=n.critical==Critical::Positive?n.position.y-2*stroke:n.position.y+n.pixels+2*stroke;
            ImDrawList_AddLine(draw,{n.position.x,y},{n.position.x+n.advance,y},edge,4*stroke);
            ImDrawList_AddLine(draw,{n.position.x,y},{n.position.x+n.advance,y},color,2*stroke);
        }
        return;
    }
    text(n.unit,n.unit,imgui_color(0,n.opacity*.38f));
    const float edge=.45f*n.unit;const uint32_t outline=imgui_color(0,n.opacity*.18f);
    text(-edge,0,outline);text(edge,0,outline);text(0,-edge,outline);text(0,edge,outline);text(0,0,color);
    if(n.critical==Critical::None)return;
    const uint32_t rgb=star_rgb(n.critical);
    const float expansions[3]={1.8f,1.5f,1.25f},alphas[3]={.055f,.09f,.16f};
    for(unsigned i=0;i<3;++i)star(draw,n.star,n.pixels*1.1f*expansions[i],n.pixels*.24f*expansions[i],imgui_color(rgb,n.opacity*alphas[i]));
    star(draw,n.star,n.pixels*1.1f,n.pixels*.24f,imgui_color(rgb,n.opacity));
    star(draw,n.star,n.pixels*.42f,n.pixels*.065f,imgui_color(n.critical==Critical::Positive?0xffe7f0:0xe5f4ff,n.opacity*.9f));
}
}
