#include "capture.h"
#include <forge/hook.h>
#include <stdarg.h>
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include <cstddef>
#include <cstdint>
struct ForgeVersion { uint16_t major,minor,patch; };
struct PluginInitParam { ForgeVersion required_ver; char name[64]; };
static_assert(sizeof(void*)==4 && sizeof(Jit)==28 && sizeof(Hook)==76);
static_assert(sizeof(PluginInitParam)==70 && offsetof(PluginInitParam,name)==6);
extern "C" void init() {}
extern "C" void fini() {}
extern "C" void forge_onLoad(PluginInitParam* out) {
    if (!out) return;
    auto* bytes=reinterpret_cast<volatile unsigned char*>(out);
    for (unsigned i=0;i<sizeof(*out);++i) bytes[i]=0;
    out->required_ver={1,4,1};
    const char name[]="Damage numbers - capture research";
    for (unsigned i=0;i<sizeof(name);++i) bytes[offsetof(PluginInitParam,name)+i]=name[i];
}
extern "C" void forge_onInit() { damage_numbers::capture_install(); }
extern "C" void forge_onUpdate(float) { damage_numbers::capture_monitor(); }
extern "C" void forge_onImGuiInit(void* ctx,void* alloc,void* free_fn) {
    igSetCurrentContext(static_cast<ImGuiContext*>(ctx));
    igSetAllocatorFunctions(reinterpret_cast<ImGuiMemAllocFunc>(alloc),reinterpret_cast<ImGuiMemFreeFunc>(free_fn),nullptr);
}
extern "C" void forge_onImGuiRender() { damage_numbers::capture_panel(); }
extern "C" void forge_onImGuiFreeRender() { damage_numbers::capture_consume(); }
