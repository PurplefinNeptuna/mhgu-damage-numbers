// ImGui font/texture lifecycle and final drawing. Embedded open assets provide
// digit metrics, shadows, critical stars, and Rise slashes; failures use logged
// fallbacks. Geometry follows the bounds produced by presentation.h.

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
#include "rise_sprite_data.h"
#include "sprite_codec.h"

namespace damage_numbers {
static void *font_context;
static ImFont *wilds_font;
static bool font_warning;
static ImTextureData *sprite_texture;
static ImTextureData *rise_texture;
static bool rise_texture_attempted;
extern "C" __attribute__((used)) const uint32_t dn_context_hook_abi[] = {
    sizeof(ImGuiContextHook), offsetof(ImGuiContextHook, Type),
    offsetof(ImGuiContextHook, Callback), offsetof(ImGuiContextHook, UserData)};

// Release the registered Rise texture during ImGui context shutdown and
// clear resource pointers belonging to that context.
static void rise_shutdown(ImGuiContext *context, ImGuiContextHook *hook) {
    auto *texture = static_cast<ImTextureData *>(hook->UserData);
    igUnregisterUserTexture(texture);
    ImTextureData_destroy(texture);
    if (font_context == context) {
        font_context = nullptr;
        wilds_font = nullptr;
        sprite_texture = nullptr;
        rise_texture = nullptr;
        rise_texture_attempted = false;
    }
}

extern "C" __attribute__((used)) const uint32_t dn_sprite_abi[] = {
    sizeof(ImTextureData), offsetof(ImTextureData, Pixels), offsetof(ImTextureData, Status),
    offsetof(ImTextureData, RefCount)};
// The fixture reads compiler-derived ABI offsets, then checks actual ARM calls.
extern "C" __attribute__((used)) const uint32_t dn_font_abi[] = {
    offsetof(ImGuiIO, Fonts),    offsetof(ImFontConfig, FontDataOwnedByAtlas),
    offsetof(ImFontBaked, Size), offsetof(ImFontGlyph, AdvanceX),
    offsetof(ImFontGlyph, X0),   offsetof(ImGuiIO, BackendFlags)};
extern "C" __attribute__((used)) const uint32_t dn_draw_abi[] = {
    offsetof(ImDrawList, VtxBuffer), offsetof(ImVector_ImDrawVert, Data), sizeof(ImDrawVert),
    offsetof(ImDrawVert, pos)};
static_assert(sizeof(ImWchar) == 2 && sizeof(ImFontGlyph) == 44);
#ifndef DAMAGE_NUMBERS_HOST_VISUAL
static_assert(offsetof(ImGuiIO, Fonts) == 44 && offsetof(ImFontConfig, FontDataOwnedByAtlas) == 48);
static_assert(offsetof(ImFontBaked, Size) == 16 && offsetof(ImFontGlyph, AdvanceX) == 4 &&
              offsetof(ImFontGlyph, X0) == 8);
#endif
// Initialize once per context. The embedded font bytes remain caller-owned;
// the atlas borrows them for the lifetime of the plugin.
void cosmetics_init(void *context) {
    if (font_context == context)
        return;
    font_context = context;
    wilds_font = nullptr;
    rise_texture = nullptr;
    rise_texture_attempted = false;
    ImGuiIO *io = igGetIO_Nil();
    if (io && io->Fonts) {
        ImFontConfig *config = ImFontConfig_ImFontConfig();
        if (config) {
            config->FontDataOwnedByAtlas = false;
            static const ImWchar ranges[] = {32, 32, 48, 57, 63, 63, 0};
            wilds_font = ImFontAtlas_AddFontFromMemoryTTF(
                io->Fonts, const_cast<unsigned char *>(wilds_font_data), sizeof(wilds_font_data),
                24 * wilds_font_scale, config, ranges);
            ImFontConfig_destroy(config);
        }
    }
    ImGuiIO *texture_io = igGetIO_Nil();
    sprite_texture = nullptr;
    if (texture_io && (texture_io->BackendFlags & ImGuiBackendFlags_RendererHasTextures)) {
        sprite_texture = ImTextureData_ImTextureData();
        if (sprite_texture) {
            ImTextureData_Create(sprite_texture, ImTextureFormat_RGBA32, wilds_sprite_width,
                                 wilds_sprite_height);
            auto *pixels = static_cast<uint8_t *>(ImTextureData_GetPixels(sprite_texture));
            if (pixels && decode_sprite(wilds_sprite_lz, sizeof(wilds_sprite_lz), pixels,
                                        wilds_sprite_width * wilds_sprite_height * 4)) {
                igRegisterUserTexture(sprite_texture);
            } else {
                ImTextureData_destroy(sprite_texture);
                sprite_texture = nullptr;
            }
        }
    }
    static mod_log::Once failures;
    if (!sprite_texture && failures.claim(0))
        forge_log_warn("[damage-numbers] Wilds effect texture unavailable; using crisp digits without sprite effects");
    if (!wilds_font && !font_warning) {
        font_warning = true;
        forge_log_warn("[damage-numbers] Wilds digit font unavailable; using Forge font with Wilds colors/stars");
    }
}

static ImFont *selected_font(VisualStyle style) {
    return style != VisualStyle::Classic && wilds_font ? wilds_font : igGetFont();
}

// Use baked glyph advances and visible ink extents at the requested size.
// Rise width scaling must match the vertex adjustment used when drawing.
bool cosmetics_measure(const char *text, float pixels, VisualStyle style, TextMetrics &out) {
    ImFont *font = selected_font(style);
    if (!font)
        return false;
    if (style != VisualStyle::Classic && wilds_font)
        pixels *= wilds_font_scale;
    ImFontBaked *baked = ImFont_GetFontBaked(font, pixels, -1);
    if (!baked || baked->Size <= 0)
        return false;
    const float ratio = pixels / baked->Size;
    float advance = 0;
    Rect ink{};
    bool found = false;
    for (const char *p = text; *p; ++p) {
        ImFontGlyph *glyph = ImFontBaked_FindGlyph(baked, ImWchar(*p));
        if (!glyph)
            return false;
        Rect r = {advance + glyph->X0 * ratio, glyph->Y0 * ratio, advance + glyph->X1 * ratio,
                  glyph->Y1 * ratio};
        if (glyph->Visible && valid(r)) {
            ink = found ? united(ink, r) : r;
            found = true;
        }
        advance += glyph->AdvanceX * ratio;
    }
    if (style == VisualStyle::Rise) {
        advance *= rise_glyph_width;
        ink.left *= rise_glyph_width;
        ink.right *= rise_glyph_width;
    }
    out = {advance, ink};
    return found && finite(advance) && advance > 0;
}

// Map one atlas rectangle to a target rectangle using normalized UVs.
static void sprite(ImDrawList *draw, unsigned index, Rect target, uint32_t color) {
    if (!sprite_texture || index >= 22)
        return;
    const auto *r = wilds_sprite_rects[index];
    ImDrawList_AddImage(
        draw, ImTextureData_GetTexRef(sprite_texture), {target.left, target.top},
        {target.right, target.bottom},
        {float(r[0]) / wilds_sprite_width, float(r[1]) / wilds_sprite_height},
        {float(r[0] + r[2]) / wilds_sprite_width, float(r[1] + r[3]) / wilds_sprite_height}, color);
}

// Fit each digit-shadow sprite to the same baked glyph geometry as the fill.
static void glyph_effects(ImDrawList *draw, const DrawNumber &n, ImFont *font, float pixels) {
    if (!sprite_texture)
        return;
    ImFontBaked *baked = ImFont_GetFontBaked(font, pixels, -1);
    if (!baked || baked->Size <= 0)
        return;
    const float ratio = pixels / baked->Size;
    float advance = 0;
    for (const char *p = n.text; *p; ++p) {
        if (*p < '0' || *p > '9')
            return;
        ImFontGlyph *g = ImFontBaked_FindGlyph(baked, ImWchar(*p));
        if (!g)
            return;
        const unsigned index = unsigned(*p - '0');
        const auto *r = wilds_sprite_rects[index];
        const float width = n.style == VisualStyle::Rise ? rise_glyph_width : 1;
        const float sx = (g->X1 - g->X0) * ratio * width / r[6],
                    sy = (g->Y1 - g->Y0) * ratio / r[7];
        const float x = n.position.x + (advance + g->X0 * ratio) * width - r[4] * sx;
        const float y = n.position.y + g->Y0 * ratio - r[5] * sy;
        const Rect target = {x, y, x + r[2] * sx, y + r[3] * sy};
        // Keep the centered shadow; the crisp edge replaces the warm gold halo.
        sprite(draw, index, target,
               imgui_color(0, n.opacity * (n.style == VisualStyle::Rise ? .55f : .22f)));
        advance += g->AdvanceX * ratio;
    }
}

// Lazily register the Rise atlas on the first slash. Submit its clipped quad
// using separate positive/negative critical halves of the texture.
void cosmetics_draw_effect(void *list, const DrawNumber &n) {
    auto *draw = static_cast<ImDrawList *>(list);
    if (!draw || n.style != VisualStyle::Rise || n.critical == Critical::None ||
        n.slash.opacity <= 0)
        return;
    if (!rise_texture_attempted) {
        rise_texture_attempted = true;
        auto *io = igGetIO_Nil();
        if (io && (io->BackendFlags & ImGuiBackendFlags_RendererHasTextures)) {
            rise_texture = ImTextureData_ImTextureData();
            if (rise_texture) {
                ImTextureData_Create(rise_texture, ImTextureFormat_RGBA32, rise_sprite_width,
                                     rise_sprite_height);
                auto *pixels = static_cast<uint8_t *>(ImTextureData_GetPixels(rise_texture));
                if (pixels && decode_sprite(rise_sprite_lz, sizeof(rise_sprite_lz), pixels,
                                            rise_sprite_width * rise_sprite_height * 4)) {
                    igRegisterUserTexture(rise_texture);
                    ImGuiContextHook hook{};
                    hook.Type = ImGuiContextHookType_Shutdown;
                    hook.Callback = rise_shutdown;
                    hook.UserData = rise_texture;
                    igAddContextHook(static_cast<ImGuiContext *>(font_context), &hook);
                } else {
                    ImTextureData_destroy(rise_texture);
                    rise_texture = nullptr;
                }
            }
        }
        static mod_log::Once failures;
        if (!rise_texture && failures.claim(0))
            forge_log_warn("[damage-numbers] Rise slash texture unavailable; digits remain enabled");
    }
    if (!rise_texture)
        return;
    Point p[4];
    slash_quad(n.slash, p);
    const float v = n.critical == Critical::Positive ? 0 : .5f;
    ImDrawList_PushClipRect(draw, {n.slash.clip.left, n.slash.clip.top},
                            {n.slash.clip.right, n.slash.clip.bottom}, true);
    ImDrawList_AddImageQuad(draw, ImTextureData_GetTexRef(rise_texture), {p[0].x, p[0].y},
                            {p[1].x, p[1].y}, {p[2].x, p[2].y}, {p[3].x, p[3].y}, {0, v}, {1, v},
                            {1, v + .5f}, {0, v + .5f}, imgui_color(0xffffff, n.slash.opacity));
    ImDrawList_PopClipRect(draw);
}

// Draw the style's outline/shadow and crisp digit fill, then its critical
// marker. Rise widens only vertices created by this text call.
void cosmetics_draw(void *list, const DrawNumber &n) {
    auto *draw = static_cast<ImDrawList *>(list);
    ImFont *font = selected_font(n.style);
    if (!draw || !font)
        return;
    const float pixels =
        n.style != VisualStyle::Classic && wilds_font ? n.pixels * wilds_font_scale : n.pixels;
    auto text = [&](float x, float y, uint32_t color) {
        const int first = draw->VtxBuffer.Size;
        ImDrawList_AddText_FontPtr(draw, font, pixels, {n.position.x + x, n.position.y + y}, color,
                                   n.text, nullptr, 0, nullptr);
        if (n.style == VisualStyle::Rise)
            for (int i = first; i < draw->VtxBuffer.Size; ++i)
                draw->VtxBuffer.Data[i].pos.x =
                    n.position.x +
                    (draw->VtxBuffer.Data[i].pos.x - n.position.x) * rise_glyph_width;
    };
    const uint32_t color = imgui_color(n.color, n.opacity);
    if (n.style == VisualStyle::Classic) {
        const float stroke = n.classic_stroke;
        const uint32_t edge = imgui_color(0, n.opacity);
        for (int x = -1; x <= 1; ++x)
            for (int y = -1; y <= 1; ++y)
                if (x || y)
                    text(x * stroke, y * stroke, edge);
        text(0, 0, color);
        if (n.critical != Critical::None) {
            const float y = n.critical == Critical::Positive ? n.position.y - 2 * stroke
                                                             : n.position.y + n.pixels + 2 * stroke;
            ImDrawList_AddLine(draw, {n.position.x, y}, {n.position.x + n.advance, y}, edge,
                               4 * stroke);
            ImDrawList_AddLine(draw, {n.position.x, y}, {n.position.x + n.advance, y}, color,
                               2 * stroke);
        }
        return;
    }
    glyph_effects(draw, n, font, pixels);
    const float stroke =
        n.style == VisualStyle::Rise ? .10f * n.pixels : .5f * n.pixels / base_pixels_720p;
    const uint32_t edge = imgui_color(n.style == VisualStyle::Rise ? 0 : 0x17130f,
                                      n.opacity * (n.style == VisualStyle::Rise ? 1 : .9f));
    for (int x = -1; x <= 1; ++x)
        for (int y = -1; y <= 1; ++y)
            if (x || y) {
                const int first = draw->VtxBuffer.Size;
                text(0, 0, edge);
                // ImGui truncates text origins. Translate emitted vertices so the
                // half-pixel edge remains symmetric at every size and resolution.
                for (int i = first; i < draw->VtxBuffer.Size; ++i) {
                    draw->VtxBuffer.Data[i].pos.x += x * stroke;
                    draw->VtxBuffer.Data[i].pos.y += y * stroke;
                }
            }
    text(0, 0, color);
    if (n.style == VisualStyle::Rise || n.critical == Critical::None)
        return;
    // Procedural sprites include the complete colored bloom and pale core.
    const float scale = n.pixels * wilds_sprite_font_scale / 60.0f * wilds_crit_scale;
    const float height = 48 * scale;
    sprite(draw, n.critical == Critical::Positive ? 20 : 21,
           {n.star.x - 44.5f * scale, n.star.y - height * .5f, n.star.x + 43.5f * scale,
            n.star.y + height * .5f},
           imgui_color(0xffffff, n.opacity));
}
} // namespace damage_numbers
