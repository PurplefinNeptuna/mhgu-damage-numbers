// Real pinned ImGui font/texture/draw-list exercise; no game or GPU is launched.
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include "cosmetics.h"
#include <cstdio>
#include <fstream>
#include <string>
#include <cassert>
extern "C" void forge_log_info(const char*,...) {}
int main(int argc,char** argv) {
    assert(argc==3);auto* ctx=igCreateContext(nullptr);auto* io=igGetIO_Nil();
    io->BackendFlags|=ImGuiBackendFlags_RendererHasTextures;
    io->DisplaySize={620,220};io->DeltaTime=1.0f/60;io->IniFilename=nullptr;
    ImFontAtlas_AddFontFromFileTTF(io->Fonts,argv[1],24,nullptr,nullptr);
    damage_numbers::cosmetics_init(ctx);damage_numbers::cosmetics_init(ctx);
    igNewFrame();auto* draw=igGetBackgroundDrawList_Nil();
    using namespace damage_numbers;
    const char* texts[]={"40","47","12","6","155","111","888"};
    for(unsigned i=0;i<7;++i) {
        DrawNumber n{};std::snprintf(n.text,sizeof(n.text),"%s",texts[i]);n.style=VisualStyle::Wilds;
        n.pixels=30;n.opacity=1;n.unit=1.25f;n.position={30+float(i%4)*150,40+float(i/4)*110};
        n.color=rgb(i==1||i==2||i==4?Tint::Weak:Tint::White,n.style);
        n.critical=i==4?Critical::Negative:Critical::Positive;
        TextMetrics m{};assert(cosmetics_measure(n.text,n.pixels,n.style,m));n.advance=m.advance;n.ink=shifted(m.ink,n.position);
        n.star={(n.ink.left+n.ink.right)*.5f,n.critical==Critical::Positive?n.ink.top-2*n.unit-n.pixels*wilds_star_height*.5f:n.ink.bottom+2*n.unit+n.pixels*wilds_star_height*.5f};
        cosmetics_draw(draw,n);
    }
    igRender();auto* dd=igGetDrawData();assert(dd->CmdListsCount==1);
    std::ofstream out(std::string(argv[2])+"/draw.json");out<<"{\"textures\":[";
    for(int i=0;i<dd->Textures->Size;++i) {
        auto* t=dd->Textures->Data[i];ImTextureData_SetTexID(t,i+1);
        std::ofstream file(std::string(argv[2])+"/texture-"+std::to_string(i+1)+".rgba",std::ios::binary);
        assert(t->Format==ImTextureFormat_RGBA32);file.write((char*)t->Pixels,t->Width*t->Height*4);
        if(i)out<<',';out<<'['<<i+1<<','<<t->Width<<','<<t->Height<<']';
    }
    out<<"],\"vertices\":[";
    for(int i=0;i<draw->VtxBuffer.Size;++i){auto v=draw->VtxBuffer.Data[i];if(i)out<<',';out<<'['<<v.pos.x<<','<<v.pos.y<<','<<v.uv.x<<','<<v.uv.y<<','<<v.col<<']';}
    out<<"],\"indices\":[";
    for(int i=0;i<draw->IdxBuffer.Size;++i){if(i)out<<',';out<<draw->IdxBuffer.Data[i];}
    out<<"],\"commands\":[";
    for(int i=0;i<draw->CmdBuffer.Size;++i){auto c=draw->CmdBuffer.Data[i];if(i)out<<',';out<<'['<<ImDrawCmd_GetTexID(&c)<<','<<c.IdxOffset<<','<<c.ElemCount<<','<<c.VtxOffset<<']';}
    out<<"]}";assert(dd->Textures->Size==2);
    std::puts("Real ImGui font, original RGBA sprites and draw commands passed");
    // Texture and font data have process/context lifetime, as in Forge.
    igDestroyContext(ctx);
}
