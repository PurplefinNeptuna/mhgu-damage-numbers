#include "log_policy.h"
#include "cosmetics.h"
#include "preview.h"
#include <forge/log.h>
#include <stdarg.h>
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include <cstddef>
#include "wilds_font_data.h"
#include "wilds_sprite_data.h"
#include "sprite_codec.h"
namespace damage_numbers {
static void* font_context;
static ImFont* wilds_font;
static bool font_warning;
static ImTextureData* sprite_texture;
extern "C" __attribute__((used)) const uint32_t dn_sprite_abi[]={
    sizeof(ImTextureData),offsetof(ImTextureData,Pixels),offsetof(ImTextureData,Status),offsetof(ImTextureData,RefCount)};
// The fixture reads compiler-derived ABI offsets, then checks actual ARM calls.
extern "C" __attribute__((used)) const uint32_t dn_font_abi[]={
    offsetof(ImGuiIO,Fonts),offsetof(ImFontConfig,FontDataOwnedByAtlas),
    offsetof(ImFontBaked,Size),offsetof(ImFontGlyph,AdvanceX),offsetof(ImFontGlyph,X0),offsetof(ImGuiIO,BackendFlags)};
extern "C" __attribute__((used)) const uint32_t dn_draw_abi[]={
    offsetof(ImDrawList,VtxBuffer),offsetof(ImVector_ImDrawVert,Data),sizeof(ImDrawVert),offsetof(ImDrawVert,pos)};
static_assert(sizeof(ImWchar)==2 && sizeof(ImFontGlyph)==44);
#ifndef DAMAGE_NUMBERS_HOST_VISUAL
static_assert(offsetof(ImGuiIO,Fonts)==44 && offsetof(ImFontConfig,FontDataOwnedByAtlas)==48);
static_assert(offsetof(ImFontBaked,Size)==16 && offsetof(ImFontGlyph,AdvanceX)==4 && offsetof(ImFontGlyph,X0)==8);
#endif
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
    ImGuiIO* texture_io=igGetIO_Nil();
    sprite_texture=nullptr;
    if(texture_io&&(texture_io->BackendFlags&ImGuiBackendFlags_RendererHasTextures)) {
        sprite_texture=ImTextureData_ImTextureData();
        if(sprite_texture) {
            ImTextureData_Create(sprite_texture,ImTextureFormat_RGBA32,wilds_sprite_width,wilds_sprite_height);
            auto* pixels=static_cast<uint8_t*>(ImTextureData_GetPixels(sprite_texture));
            if(pixels&&decode_sprite(wilds_sprite_lz,sizeof(wilds_sprite_lz),pixels,
                    wilds_sprite_width*wilds_sprite_height*4)) {
                igRegisterUserTexture(sprite_texture);
            } else {ImTextureData_destroy(sprite_texture);sprite_texture=nullptr;}
        }
    }
    static mod_log::Once failures;
    if(!sprite_texture && failures.claim(0))forge_log_warn("[damage-numbers] Wilds effect texture unavailable; using crisp digits without sprite effects");
    if(!wilds_font&&!font_warning) {
        font_warning=true;
        forge_log_warn("[damage-numbers] Wilds digit font unavailable; using Forge font with Wilds colors/stars");
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
static void sprite(ImDrawList* draw,unsigned index,Rect target,uint32_t color) {
    if(!sprite_texture||index>=22)return;
    const auto* r=wilds_sprite_rects[index];
    ImDrawList_AddImage(draw,ImTextureData_GetTexRef(sprite_texture),
        {target.left,target.top},{target.right,target.bottom},
        {float(r[0])/wilds_sprite_width,float(r[1])/wilds_sprite_height},
        {float(r[0]+r[2])/wilds_sprite_width,float(r[1]+r[3])/wilds_sprite_height},color);
}
static void glyph_effects(ImDrawList* draw,const DrawNumber& n,ImFont* font,float pixels) {
    if(!sprite_texture)return;
    ImFontBaked* baked=ImFont_GetFontBaked(font,pixels,-1);if(!baked||baked->Size<=0)return;
    const float ratio=pixels/baked->Size;float advance=0;
    for(const char* p=n.text;*p;++p) {
        if(*p<'0'||*p>'9')return;
        ImFontGlyph* g=ImFontBaked_FindGlyph(baked,ImWchar(*p));if(!g)return;
        const unsigned index=unsigned(*p-'0');const auto* r=wilds_sprite_rects[index];
        const float sx=(g->X1-g->X0)*ratio/r[6],sy=(g->Y1-g->Y0)*ratio/r[7];
        const float x=n.position.x+advance+g->X0*ratio-r[4]*sx;
        const float y=n.position.y+g->Y0*ratio-r[5]*sy;
        const Rect target={x,y,x+r[2]*sx,y+r[3]*sy};
        // Keep the centered shadow; the crisp edge replaces the warm gold halo.
        sprite(draw,index,target,imgui_color(0,n.opacity*.22f));
        advance+=g->AdvanceX*ratio;
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
    glyph_effects(draw,n,font,pixels);
    const float stroke=.5f*n.pixels/base_pixels_720p;
    const uint32_t edge=imgui_color(0x17130f,n.opacity*.9f);
    for(int x=-1;x<=1;++x)for(int y=-1;y<=1;++y)if(x||y) {
        const int first=draw->VtxBuffer.Size;
        text(0,0,edge);
        // ImGui truncates text origins. Translate emitted vertices so the
        // half-pixel edge remains symmetric at every size and resolution.
        for(int i=first;i<draw->VtxBuffer.Size;++i) {
            draw->VtxBuffer.Data[i].pos.x+=x*stroke;
            draw->VtxBuffer.Data[i].pos.y+=y*stroke;
        }
    }
    text(0,0,color);
    if(n.critical==Critical::None)return;
    // Procedural sprites include the complete colored bloom and pale core.
    const float scale=n.pixels*wilds_sprite_font_scale/60.0f*wilds_crit_scale;
    const float height=48*scale;
    sprite(draw,n.critical==Critical::Positive?20:21,
        {n.star.x-44.5f*scale,n.star.y-height*.5f,n.star.x+43.5f*scale,n.star.y+height*.5f},
        imgui_color(0xffffff,n.opacity));
}
}
