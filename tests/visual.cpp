// Run the pinned C++ oracle and production Rust with the same real ImGui.
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include "oracle/cosmetics.h"
#include "oracle/preview.h"
#include <fstream>
#include <cassert>
#include <string>
#include <filesystem>
#include <vector>
#include <set>
extern "C" void forge_log_info(const char*,...) {}
extern "C" void rs_initialize();
extern "C" unsigned rs_warning_count();
extern "C" void rs_scene(unsigned, float, float, float, float);

// Linker wrappers inject a short read and count real file handles. Font loading
// stays on ImGui's normal path; only the plugin's artwork-file API is wrapped.
static unsigned opens, closes, reads;
static int fail_read = -1;
static std::set<ImFileHandle> live_files;
static std::set<ImTextureData*> live_user_textures;
static bool tracking_rust;
extern "C" void __real_igRegisterUserTexture(ImTextureData*);
extern "C" void __real_igUnregisterUserTexture(ImTextureData*);
extern "C" void __wrap_igRegisterUserTexture(ImTextureData* texture) {
    if (tracking_rust) assert(live_user_textures.insert(texture).second);
    __real_igRegisterUserTexture(texture);
}
extern "C" void __wrap_igUnregisterUserTexture(ImTextureData* texture) {
    live_user_textures.erase(texture);
    __real_igUnregisterUserTexture(texture);
}
extern "C" ImFileHandle __real_igImFileOpen(const char*, const char*);
extern "C" bool __real_igImFileClose(ImFileHandle);
extern "C" ImU64 __real_igImFileRead(void*, ImU64, ImU64, ImFileHandle);
extern "C" ImFileHandle __wrap_igImFileOpen(const char* path, const char* mode) {
    auto* file = __real_igImFileOpen(path, mode);
    if (file) { ++opens; assert(live_files.insert(file).second); }
    return file;
}
extern "C" bool __wrap_igImFileClose(ImFileHandle file) {
    assert(live_files.erase(file) == 1); ++closes;
    return __real_igImFileClose(file);
}
extern "C" ImU64 __wrap_igImFileRead(void* data, ImU64 size, ImU64 count, ImFileHandle file) {
    unsigned call = reads++;
    if (int(call) == fail_read) return count - 1;
    return __real_igImFileRead(data, size, count, file);
}

// Serialize actual atlas pixels and draw commands for the unchanged compositor.
void dump(ImDrawList* draw, const std::string& path) {
    auto* dd = igGetDrawData();
    std::ofstream out(path + "/draw.json");
    out.precision(9);
    out << "{\"display\":[" << dd->DisplaySize.x << ',' << dd->DisplaySize.y
        << "],\"background\":[0.09,0.12,0.14],\"textures\":[";
    for (int i = 0; i < dd->Textures->Size; ++i) {
        auto* texture = dd->Textures->Data[i];
        ImTextureData_SetTexID(texture, i + 1);
        std::ofstream file(path + "/texture-" + std::to_string(i + 1) + ".rgba", std::ios::binary);
        assert(texture->Format == ImTextureFormat_RGBA32);
        file.write((char*)texture->Pixels, texture->Width * texture->Height * 4);
        if (i) out << ',';
        out << '[' << i + 1 << ',' << texture->Width << ',' << texture->Height << ']';
        ImTextureData_SetStatus(texture, ImTextureStatus_OK);
    }
    out << "],\"vertices\":[";
    for (int i = 0; i < draw->VtxBuffer.Size; ++i) {
        auto v = draw->VtxBuffer.Data[i];
        if (i) out << ',';
        out << '[' << v.pos.x << ',' << v.pos.y << ',' << v.uv.x << ',' << v.uv.y << ',' << v.col << ']';
    }
    out << "],\"indices\":[";
    for (int i = 0; i < draw->IdxBuffer.Size; ++i) {
        if (i) out << ',';
        out << draw->IdxBuffer.Data[i];
    }
    out << "],\"commands\":[";
    for (int i = 0; i < draw->CmdBuffer.Size; ++i) {
        auto c = draw->CmdBuffer.Data[i];
        if (i) out << ',';
        out << '[' << ImDrawCmd_GetTexID(&c) << ',' << c.IdxOffset << ',' << c.ElemCount
            << ',' << c.VtxOffset << ',' << c.ClipRect.x << ',' << c.ClipRect.y
            << ',' << c.ClipRect.z << ',' << c.ClipRect.w << ']';
    }
    out << "]}";
}
struct Scene { unsigned style; float age, width, height, scale; };
std::vector<Scene> scenes() {
    std::vector<Scene> result;
    // UI order changes, but the original enum IDs remain Wilds=0, Classic=1, Rise=2.
    for (unsigned style : {0u, 2u, 1u}) {
        for (float height : {720.f, 1080.f})
            for (float age : {.05f, .2f, .75f})
                result.push_back({style, age, height * 16 / 9, height, 1});
        for (float scale : {.5f, 3.f})
            result.push_back({style, .2f, 1920, 1080, scale});
    }
    return result;
}
// Feed identical copied hits into the frozen oracle at each resolution/scale.
void cpp_scene(const Scene& s, ImDrawList* draw) {
    using namespace damage_numbers;
    Model model; model.reset(7);
    Settings settings; settings.style = VisualStyle(s.style); settings.scale = s.scale;
    model.configure(settings);
    int amounts[] = {40,47,12,6,155,111,888,1,9999};
    for (unsigned i = 0; i < 9; ++i) {
        Event e{{7,i+1,42,0}, {float(int(i%3)-1)*.3f,float(int(i/3)-1)*.3f,.5f},
            amounts[i], int16_t(i%3==0?44:45), Source::Attack, Owner::LocalPlayer,
            Critical(i%3), i==3, 0};
        model.capture(e);
    }
    model.update(s.age, true);
    View view{{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1},0,0,s.width,s.height,DepthRange::ZeroToOne};
    DrawNumber numbers[128]; PlacementState cache;
    unsigned count = layout(model,view,s.age,numbers,cache,cosmetics_measure);
    ImDrawList_PushClipRect(draw,{0,0},{s.width,s.height},true);
    for (unsigned i=0;i<count;++i) cosmetics_draw_effect(draw,numbers[i]);
    for (unsigned i=0;i<count;++i) cosmetics_draw(draw,numbers[i]);
    ImDrawList_PopClipRect(draw);
}
// Missing and malformed artwork must preserve numbers and leave no user
// textures or open handles after shutdown. Each new context retries once.
void faults(const char* font_path, const std::string& output) {
    const auto assets = std::filesystem::path("app:/nativeNX/plugins/damage-numbers");
    tracking_rust = true;
    unsigned cases = 0, reused = 0;
    ImGuiContext* previous = nullptr;
    for (int mode = 0; mode < 8; ++mode) for (int repeat = 0; repeat < 2; ++repeat) {
        auto* ctx = igCreateContext(nullptr);
        if (ctx == previous) ++reused;
        igSetCurrentContext(ctx);
        auto* io = igGetIO_Nil(); io->BackendFlags |= ImGuiBackendFlags_RendererHasTextures;
        io->DisplaySize = {1920,1080}; io->DeltaTime = 1.f/60; io->IniFilename = nullptr;
        ImFontAtlas_AddFontFromFileTTF(io->Fonts, font_path, 24, nullptr, nullptr);
        std::vector<std::filesystem::path> modified;
        auto remove = [&](const char* name, bool wrong_size) {
            auto path = assets / name;
            std::filesystem::rename(path, path.string() + ".saved");
            modified.push_back(path);
            if (wrong_size) { std::ofstream file(path, std::ios::binary); file << "wrong"; }
        };
        if (mode == 0 || mode == 4 || mode == 7) remove("wilds-effects.rgba", false);
        if (mode == 1) remove("wilds-effects.rgba", true);
        if (mode == 2 || mode == 4 || mode == 7) remove("rise-slash.rgba", false);
        if (mode == 3) remove("rise-slash.rgba", true);
        if (mode == 4) remove("OpenSans.ttf", false);
        opens = closes = reads = 0; live_files.clear();
        fail_read = mode == 5 ? 0 : mode == 6 ? 1 : -1;
        rs_initialize();
        unsigned initialized_opens = opens;
        rs_initialize(); assert(opens == initialized_opens);
        igNewFrame(); rs_scene(2, .05f, 1920, 1080, 1); igRender();
        auto* dd = igGetDrawData(); assert(dd->TotalVtxCount > 0);
        assert(live_user_textures.size() == (mode == 4 || mode == 7 ? 0u : 1u));
        assert(live_files.empty() && opens == closes);
        unsigned attempted_opens = opens;
        igNewFrame(); rs_scene(2, .05f, 1920, 1080, 1); igRender();
        assert(opens == attempted_opens && live_files.empty());
        igDestroyContext(ctx); previous = ctx;
        assert(live_user_textures.empty());
        for (const auto& path : modified) {
            std::filesystem::remove(path);
            std::filesystem::rename(path.string() + ".saved", path);
        }
        ++cases;
    }
    assert(rs_warning_count() == 3);
    std::ofstream out(output + "/faults.json");
    out << "{\"cases\":" << cases << ",\"reused_context_addresses\":" << reused
        << ",\"warnings\":3,\"open_handles\":0,\"readable_numbers\":true,\"shutdown_cleanup\":true}";
}
int main(int argc, char** argv) {
    assert(argc == 3 || argc == 4);
    if (argc == 4) { faults(argv[1], argv[2]); return 0; }
    std::vector<ImGuiContext*> contexts;
    for (const auto& s : scenes()) for (bool rust : {false,true}) {
        auto* ctx = igCreateContext(nullptr); contexts.push_back(ctx); igSetCurrentContext(ctx);
        auto* io = igGetIO_Nil(); io->BackendFlags |= ImGuiBackendFlags_RendererHasTextures;
        io->DisplaySize = {s.width,s.height}; io->DeltaTime=1.f/60; io->IniFilename=nullptr;
        ImFontAtlas_AddFontFromFileTTF(io->Fonts,argv[1],24,nullptr,nullptr);
        tracking_rust = rust;
        if (rust) {
            unsigned before = opens;
            rs_initialize(); assert(opens == before + 1 && live_files.empty());
        } else damage_numbers::cosmetics_init(ctx);
        igNewFrame(); auto* draw=igGetBackgroundDrawList_Nil();
        if (rust) rs_scene(s.style,s.age,s.width,s.height,s.scale); else cpp_scene(s,draw);
        igRender();
        std::string name = std::to_string(s.style) + "-" + std::to_string(int(s.height))
            + "-" + std::to_string(s.scale) + "-" + std::to_string(s.age);
        std::string path=std::string(argv[2])+"/"+(rust?"rust-":"cpp-")+name;
        std::filesystem::create_directories(path); dump(draw,path);
    }
    for(auto* ctx:contexts) { igSetCurrentContext(ctx); igDestroyContext(ctx); }
    assert(live_user_textures.empty() && live_files.empty());
}
