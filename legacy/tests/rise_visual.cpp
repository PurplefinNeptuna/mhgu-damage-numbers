// Render Rise critical-slash geometry with actual ImGui draw data for the
// visual compositor, including a bright diagnostic background.

// Real pinned ImGui Rise preview with actual model, layout and cosmetics.
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
#include "cosmetics.h"
#include <fstream>
#include <cstdio>
#include <cassert>
#include <cmath>
#include <string>
using namespace damage_numbers;

static void dump(ImDrawList *draw, const std::string &path, bool bright) {
    auto *dd = igGetDrawData();
    assert(dd->CmdListsCount <= 1);
    std::ofstream out(path + "/draw.json");
    out << "{\"display\":[1280,720],\"background\":"
        << (bright ? "[0.87,0.9,0.86]" : "[0.09,0.12,0.14]") << ",\"textures\":[";
    for (int i = 0; i < dd->Textures->Size; ++i) {
        auto *t = dd->Textures->Data[i];
        ImTextureData_SetTexID(t, i + 1);
        std::ofstream file(path + "/texture-" + std::to_string(i + 1) + ".rgba", std::ios::binary);
        assert(t->Format == ImTextureFormat_RGBA32);
        file.write((char *)t->Pixels, t->Width * t->Height * 4);
        if (i)
            out << ',';
        out << '[' << i + 1 << ',' << t->Width << ',' << t->Height << ']';
        ImTextureData_SetStatus(t, ImTextureStatus_OK);
    }
    out << "],\"vertices\":[";
    for (int i = 0; i < draw->VtxBuffer.Size; ++i) {
        auto v = draw->VtxBuffer.Data[i];
        if (i)
            out << ',';
        out << '[' << v.pos.x << ',' << v.pos.y << ',' << v.uv.x << ',' << v.uv.y << ',' << v.col
            << ']';
    }
    out << "],\"indices\":[";
    for (int i = 0; i < draw->IdxBuffer.Size; ++i) {
        if (i)
            out << ',';
        out << draw->IdxBuffer.Data[i];
    }
    out << "],\"commands\":[";
    for (int i = 0; i < draw->CmdBuffer.Size; ++i) {
        auto c = draw->CmdBuffer.Data[i];
        if (i)
            out << ',';
        out << '[' << ImDrawCmd_GetTexID(&c) << ',' << c.IdxOffset << ',' << c.ElemCount << ','
            << c.VtxOffset << ',' << c.ClipRect.x << ',' << c.ClipRect.y << ',' << c.ClipRect.z
            << ',' << c.ClipRect.w << ']';
    }
    out << "]}";
}

void rise_visual(const std::string &root) {
    auto *io = igGetIO_Nil();
    io->DisplaySize = {1280, 720};
    View v = {
        {1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1}, 0, 0, 1280, 720, DepthRange::ZeroToOne};
    for (bool bright : {false, true})
        for (float fraction : {.5f, .75f}) {
            Model model;
            model.reset(7);
            Settings s;
            s.style = VisualStyle::Rise;
            assert(model.configure(s));
            for (unsigned i = 0; i < 9; ++i) {
                float x = 135 + (i % 3) * 425, y = i < 3 ? 200 : i < 6 ? 460 : 570;
                Event e = {{7, i + 1, 20, 0},
                           {x / 640 - 1, 1 - y / 360, .5f},
                           i % 3 == 0   ? 111
                           : i % 3 == 1 ? 888
                                        : 155,
                           int16_t(i % 3 == 0 ? 44 : 60),
                           Source::Attack,
                           Owner::LocalPlayer,
                           i < 3   ? Critical::None
                           : i < 6 ? Critical::Positive
                                   : Critical::Negative,
                           i % 3 == 2,
                           1};
                assert(model.capture(e));
            }
            for (unsigned i = 10; i < 14; ++i) {
                Event e = {{7, i, 20, 0},
                           {0, 1 - 650.0f / 360, .5f},
                           int32_t(47 + i),
                           45,
                           Source::Attack,
                           Owner::LocalPalico,
                           i % 2 ? Critical::Positive : Critical::Negative,
                           false,
                           1};
                assert(model.capture(e));
            }
            PlacementState cache;
            for (unsigned frame = 0; frame <= 27; ++frame) {
                float age = frame / 30.0f;
                model.update(1 + age, true);
                if (frame != 0 && frame != 1 && frame != 2 && frame != 4 && frame != 6 &&
                    frame != 9 && frame != 21 && frame != 23 && frame != 26 && frame != 27)
                    continue;
                igNewFrame();
                auto *draw = igGetBackgroundDrawList_Nil();
                DrawNumber n[128]{};
                unsigned count = layout(model, v, 1 + age, n, cache, cosmetics_measure);
                for (unsigned i = 0; i < count; ++i) {
                    n[i].slash = rise_slash(n[i].slash.origin, n[i].key, age, v, fraction);
                    cosmetics_draw_effect(draw, n[i]);
                }
                for (unsigned i = 0; i < count; ++i)
                    cosmetics_draw(draw, n[i]);
                igRender();
                dump(draw,
                     root + "/rise-" + (bright ? "bright" : "dark") + "-" +
                         (fraction == .5f ? "half" : "three-quarter") + "-" + std::to_string(frame),
                     bright);
            }
        }
    // Worst-case length/count bound, including thick outline and separate beams.
    igNewFrame();
    auto *draw = igGetBackgroundDrawList_Nil();
    for (unsigned i = 0; i < 128; ++i) {
        DrawNumber n{};
        std::snprintf(n.text, sizeof(n.text), "2147483647");
        n.style = VisualStyle::Rise;
        n.pixels = 35;
        n.opacity = 1;
        n.position = {30, 40};
        n.color = rgb(Tint::Weak, n.style);
        n.critical = Critical::Positive;
        n.slash = rise_slash({80, 300}, {7, i + 1, 20, 0}, .05f, v);
        cosmetics_draw_effect(draw, n);
        cosmetics_draw(draw, n);
    }
    assert(draw->VtxBuffer.Size == 51712 && draw->VtxBuffer.Size < 65536);
    igEndFrame();
    std::puts("Rise real-ImGui frames, clipped tilted quads and 128 ten-digit labels passed");
}
