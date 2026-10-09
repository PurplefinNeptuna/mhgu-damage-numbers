// Forge entry points: install capture at boot, monitor patches on update,
// initialize ImGui resources, and route settings and overlay callbacks separately.

#include "capture.h"
#include "cosmetics.h"
#include <forge/hook.h>
#include <forge/log.h>
#include <stdarg.h>
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include <cstddef>
#include <cstdint>

// These structures mirror the loader's ARM32 plugin handshake. The assertions
// below catch incompatible pointer widths or field packing at compile time.
struct ForgeVersion {
    uint16_t major, minor, patch;
};

struct PluginInitParam {
    ForgeVersion required_ver;
    char name[64];
};

static_assert(sizeof(void *) == 4 && sizeof(Jit) == 28 && sizeof(Hook) == 76);
static_assert(sizeof(PluginInitParam) == 70 && offsetof(PluginInitParam, name) == 6);

extern "C" void init() {
}

extern "C" void fini() {
}

// Advertise the required Forge version and plugin name through the supplied
// fixed-layout buffer. Volatile byte stores also clear any padding.
extern "C" void forge_onLoad(PluginInitParam *out) {
    if (!out)
        return;
    auto *bytes = reinterpret_cast<volatile unsigned char *>(out);
    for (unsigned i = 0; i < sizeof(*out); ++i)
        bytes[i] = 0;
    out->required_ver = {1, 4, 1};
    const char name[] = "Damage numbers";
    for (unsigned i = 0; i < sizeof(name); ++i)
        bytes[offsetof(PluginInitParam, name) + i] = name[i];
}

// Boot is the verified quiet boundary for publishing detours.
extern "C" void forge_onInit() {
    damage_numbers::capture_install();
}

// Detect later changes to patched code before trusting further captures.
extern "C" void forge_onUpdate(float) {
    damage_numbers::capture_monitor();
}

// Use the loader's ImGui context and allocators before registering resources.
extern "C" void forge_onImGuiInit(void *ctx, void *alloc, void *free_fn) {
    igSetCurrentContext(static_cast<ImGuiContext *>(ctx));
    igSetAllocatorFunctions(reinterpret_cast<ImGuiMemAllocFunc>(alloc),
                            reinterpret_cast<ImGuiMemFreeFunc>(free_fn), nullptr);
    damage_numbers::cosmetics_init(ctx);
    damage_numbers::capture_reset_placement();
}

// Render the compact settings tree when Forge visits Plugin UI.
extern "C" void forge_onImGuiRender() {
    damage_numbers::capture_panel();
}

// Draw the overlay independently of whether the settings tree is open.
extern "C" void forge_onImGuiFreeRender() {
    damage_numbers::capture_render();
}
