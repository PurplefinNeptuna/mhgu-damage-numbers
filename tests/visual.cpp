// Run C++ r25 and production Rust against the same real ImGui and serialized textures/vertices.
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include "oracle/cosmetics.h"
#include "oracle/preview.h"
#include <fstream>
#include <cassert>
#include <string>
#include <filesystem>
#include <vector>
extern "C" void forge_log_info(const char*,...){}
extern "C" void rs_initialize();
extern "C" void rs_scene(unsigned,float);
void dump(ImDrawList* draw,const std::string &path){auto* dd=igGetDrawData();std::ofstream out(path+"/draw.json");out.precision(9);out<<"{\"display\":[1920,1080],\"background\":[0.09,0.12,0.14],\"textures\":[";for(int i=0;i<dd->Textures->Size;++i){auto*t=dd->Textures->Data[i];ImTextureData_SetTexID(t,i+1);std::ofstream file(path+"/texture-"+std::to_string(i+1)+".rgba",std::ios::binary);assert(t->Format==ImTextureFormat_RGBA32);file.write((char*)t->Pixels,t->Width*t->Height*4);if(i)out<<',';out<<'['<<i+1<<','<<t->Width<<','<<t->Height<<']';ImTextureData_SetStatus(t,ImTextureStatus_OK);}out<<"],\"vertices\":[";for(int i=0;i<draw->VtxBuffer.Size;++i){auto v=draw->VtxBuffer.Data[i];if(i)out<<',';out<<'['<<v.pos.x<<','<<v.pos.y<<','<<v.uv.x<<','<<v.uv.y<<','<<v.col<<']';}out<<"],\"indices\":[";for(int i=0;i<draw->IdxBuffer.Size;++i){if(i)out<<',';out<<draw->IdxBuffer.Data[i];}out<<"],\"commands\":[";for(int i=0;i<draw->CmdBuffer.Size;++i){auto c=draw->CmdBuffer.Data[i];if(i)out<<',';out<<'['<<ImDrawCmd_GetTexID(&c)<<','<<c.IdxOffset<<','<<c.ElemCount<<','<<c.VtxOffset<<','<<c.ClipRect.x<<','<<c.ClipRect.y<<','<<c.ClipRect.z<<','<<c.ClipRect.w<<']';}out<<"]}";}
int main(int argc,char**argv){assert(argc==3);using namespace damage_numbers;std::vector<ImGuiContext*> contexts;for(unsigned style=0;style<3;++style)for(float age:{.05f,.2f,.75f})for(bool rust:{false,true}){
 auto*ctx=igCreateContext(nullptr);contexts.push_back(ctx);igSetCurrentContext(ctx);auto*io=igGetIO_Nil();io->BackendFlags|=ImGuiBackendFlags_RendererHasTextures;io->DisplaySize={1920,1080};io->DeltaTime=1.f/60;io->IniFilename=nullptr;ImFontAtlas_AddFontFromFileTTF(io->Fonts,argv[1],24,nullptr,nullptr);if(rust)rs_initialize();else cosmetics_init(ctx);igNewFrame();auto*draw=igGetBackgroundDrawList_Nil();if(rust)rs_scene(style,age);else{
 Model model;model.reset(7);Settings settings;settings.style=VisualStyle(style);model.configure(settings);int amounts[]={40,47,12,6,155,111,888,1,9999};for(unsigned i=0;i<9;++i){Event e{{7,i+1,42,0},{float(int(i%3)-1)*.3f,float(int(i/3)-1)*.3f,.5f},amounts[i],int16_t(i%3==0?44:45),Source::Attack,Owner::LocalPlayer,Critical(i%3),i==3,0};model.capture(e);}model.update(age,true);View view{{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1},0,0,1920,1080,DepthRange::ZeroToOne};DrawNumber numbers[128];PlacementState cache;unsigned count=layout(model,view,age,numbers,cache,cosmetics_measure);ImDrawList_PushClipRect(draw,{0,0},{1920,1080},true);for(unsigned i=0;i<count;++i)cosmetics_draw_effect(draw,numbers[i]);for(unsigned i=0;i<count;++i)cosmetics_draw(draw,numbers[i]);ImDrawList_PopClipRect(draw);}
 igRender();std::string path=std::string(argv[2])+"/"+(rust?"rust-":"cpp-")+std::to_string(style)+"-"+std::to_string(age);std::filesystem::create_directories(path);dump(draw,path);
}for(auto*ctx:contexts){igSetCurrentContext(ctx);igDestroyContext(ctx);}}
