//! Forge/ImGui rendering of copied draw commands. Assets are loaded once per context, never per hit.
use core::{
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
static CONTEXT_EPOCH: AtomicU32 = AtomicU32::new(0);
use damage_numbers_core::{presentation::*, *};
use forge::{
    imgui::{DrawList, ImFont as Font},
    sys::imgui::*,
};
include!(concat!(env!("OUT_DIR"), "/assets.rs"));
#[derive(Default)]
pub struct Resources {
    font: Option<Font>,
    context: *mut ImGuiContext,
    context_epoch: u32,
    wilds: *mut ImTextureData,
    rise: *mut ImTextureData,
    rise_attempted: bool,
}
fn point(p: Point) -> ImVec2 {
    ImVec2::new(p.x, p.y)
}
impl Resources {
    /// The atlas owns file-loaded font data. Reinitialization discards pointers belonging to the preceding context.
    pub fn initialize(&mut self) {
        let context = forge::imgui::get_current_context() as *mut _;
        let epoch = CONTEXT_EPOCH.load(Ordering::Relaxed);
        if self.context == context && self.context_epoch == epoch {
            return;
        }
        self.context = context;
        self.context_epoch = epoch;
        self.font = None;
        self.wilds = ptr::null_mut();
        self.rise = ptr::null_mut();
        self.rise_attempted = false;
        let path = alloc::ffi::CString::new(alloc::format!(
            "{}:/nativeNX/plugins/damage-numbers-rs/wilds-digits.ttf",
            forge::fs::romfs::mount_point()
        ))
        .unwrap();
        // SAFETY: the configuration constructor and destructor use Forge's active ImGui allocator. Glyph ranges are static and NUL-terminated.
        unsafe {
            let cfg = ImFontConfig_ImFontConfig();
            if !cfg.is_null() {
                (*cfg).FontDataOwnedByAtlas = true;
                static RANGES: [ImWchar; 7] = [32, 32, 48, 57, 63, 63, 0];
                self.font = Font::load_with(&path, 24.0 * FONT_SCALE, &*cfg, Some(&RANGES));
                ImFontConfig_destroy(cfg);
            }
            self.wilds = self.texture(WILDS, 512, 256);
        }
        if self.font.is_none() {
            log::warn!("[damage-numbers-rs] digit font unavailable; using Forge font");
        }
        if self.wilds.is_null() {
            log::warn!("[damage-numbers-rs] effect atlas unavailable");
        }
    }
    /// Register a decoded texture and arrange matching context-shutdown destruction.
    unsafe fn texture(&self, packed: &[u8], width: i32, height: i32) -> *mut ImTextureData {
        // SAFETY: only called with an active Forge context. Constructor-allocated texture owns its pixel buffer and shutdown callback.
        unsafe {
            let io = forge::imgui::get_io();
            if (io.BackendFlags.0 & ImGuiBackendFlags::RendererHasTextures.0) == 0 {
                return ptr::null_mut();
            }
            let texture = ImTextureData_ImTextureData();
            if texture.is_null() {
                return texture;
            }
            ImTextureData_Create(texture, ImTextureFormat::RGBA32, width, height);
            let pixels = ImTextureData_GetPixels(texture).cast::<u8>();
            if pixels.is_null()
                || !decode_sprite(
                    packed,
                    core::slice::from_raw_parts_mut(pixels, width as usize * height as usize * 4),
                )
            {
                ImTextureData_destroy(texture);
                return ptr::null_mut();
            }
            igRegisterUserTexture(texture);
            let mut hook = ImGuiContextHook {
                HookId: 0,
                Type: ImGuiContextHookType::Shutdown,
                Owner: 0,
                Callback: Some(shutdown),
                UserData: texture.cast(),
            };
            igAddContextHook(self.context, &mut hook);
            texture
        }
    }
    fn selected_font(&self, style: Style) -> Font {
        if style != Style::Classic {
            if let Some(font) = self.font {
                return font;
            }
        }
        Font::current()
    }
    fn font_pixels(&self, pixels: f32, style: Style) -> f32 {
        if style != Style::Classic && self.font.is_some() {
            pixels * FONT_SCALE
        } else {
            pixels
        }
    }
    pub fn measure(&self, text: &[u8; 12], pixels: f32, style: Style) -> Option<TextMetrics> {
        let font = self.selected_font(style);
        let pixels = self.font_pixels(pixels, style);
        // SAFETY: font belongs to the current atlas; glyphs are read only and every pointer is checked before access.
        unsafe {
            let baked = ImFont_GetFontBaked(font.raw(), pixels, -1.0);
            if baked.is_null() || (*baked).Size <= 0.0 {
                return None;
            }
            let ratio = pixels / (*baked).Size;
            let mut advance = 0.0;
            let mut ink: Option<Rect> = None;
            for &byte in text.iter().take_while(|&&b| b != 0) {
                let glyph = ImFontBaked_FindGlyph(baked, byte as ImWchar);
                if glyph.is_null() {
                    return None;
                }
                let g = &*glyph;
                let bounds = Rect {
                    left: advance + g.X0 * ratio,
                    top: g.Y0 * ratio,
                    right: advance + g.X1 * ratio,
                    bottom: g.Y1 * ratio,
                };
                if g.Visible() != 0 && bounds.valid() {
                    ink = Some(ink.map_or(bounds, |a| a.united(bounds)));
                }
                advance += g.AdvanceX * ratio;
            }
            let mut ink = ink?;
            if style == Style::Rise {
                advance *= RISE_GLYPH_WIDTH;
                ink.left *= RISE_GLYPH_WIDTH;
                ink.right *= RISE_GLYPH_WIDTH;
            }
            if !advance.is_finite() || advance <= 0.0 {
                return None;
            }
            Some(TextMetrics { advance, ink })
        }
    }
    unsafe fn sprite(&self, draw: *mut ImDrawList, index: usize, target: Rect, color: u32) {
        if self.wilds.is_null() || index >= 22 {
            return;
        }
        let r = RECTS[index];
        // SAFETY: registered atlas and current draw list remain live during the render callback; rectangles index the pinned atlas.
        unsafe {
            ImDrawList_AddImage(
                draw,
                ImTextureData_GetTexRef(self.wilds),
                ImVec2::new(target.left, target.top),
                ImVec2::new(target.right, target.bottom),
                ImVec2::new(r[0] as f32 / 512.0, r[1] as f32 / 256.0),
                ImVec2::new((r[0] + r[2]) as f32 / 512.0, (r[1] + r[3]) as f32 / 256.0),
                color,
            );
        }
    }
    unsafe fn glyph_effects(&self, draw: *mut ImDrawList, n: &DrawNumber, font: Font, pixels: f32) {
        if self.wilds.is_null() {
            return;
        }
        // SAFETY: atlas font, baked glyphs and draw list are current-context objects; checks precede each dereference.
        unsafe {
            let baked = ImFont_GetFontBaked(font.raw(), pixels, -1.0);
            if baked.is_null() || (*baked).Size <= 0.0 {
                return;
            }
            let ratio = pixels / (*baked).Size;
            let mut advance = 0.0;
            for &byte in n.text.iter().take_while(|&&b| b != 0) {
                if !byte.is_ascii_digit() {
                    return;
                }
                let glyph = ImFontBaked_FindGlyph(baked, byte as ImWchar);
                if glyph.is_null() {
                    return;
                }
                let g = &*glyph;
                let index = (byte - b'0') as usize;
                let r = RECTS[index];
                let width = if n.style == Style::Rise {
                    RISE_GLYPH_WIDTH
                } else {
                    1.0
                };
                let sx = (g.X1 - g.X0) * ratio * width / r[6] as f32;
                let sy = (g.Y1 - g.Y0) * ratio / r[7] as f32;
                let x = n.position.x + (advance + g.X0 * ratio) * width - r[4] as f32 * sx;
                let y = n.position.y + g.Y0 * ratio - r[5] as f32 * sy;
                self.sprite(
                    draw,
                    index,
                    Rect {
                        left: x,
                        top: y,
                        right: x + r[2] as f32 * sx,
                        bottom: y + r[3] as f32 * sy,
                    },
                    imgui_color(
                        0,
                        n.opacity * if n.style == Style::Rise { 0.55 } else { 0.22 },
                    ),
                );
                advance += g.AdvanceX * ratio;
            }
        }
    }
    unsafe fn effect(&mut self, draw: *mut ImDrawList, n: &DrawNumber) {
        if n.style != Style::Rise || n.critical == Critical::None || n.slash.opacity <= 0.0 {
            return;
        }
        // SAFETY: creation and draw submission run only in the active ImGui callback.
        unsafe {
            if !self.rise_attempted {
                self.rise_attempted = true;
                self.rise = self.texture(RISE, 64, 512);
                if self.rise.is_null() {
                    log::warn!("[damage-numbers-rs] Rise slash unavailable");
                }
            }
            if self.rise.is_null() {
                return;
            }
            let points = slash_quad(n.slash);
            let v = if n.critical == Critical::Positive {
                0.0
            } else {
                0.5
            };
            ImDrawList_PushClipRect(
                draw,
                ImVec2::new(n.slash.clip.left, n.slash.clip.top),
                ImVec2::new(n.slash.clip.right, n.slash.clip.bottom),
                true,
            );
            ImDrawList_AddImageQuad(
                draw,
                ImTextureData_GetTexRef(self.rise),
                point(points[0]),
                point(points[1]),
                point(points[2]),
                point(points[3]),
                ImVec2::new(0.0, v),
                ImVec2::new(1.0, v),
                ImVec2::new(1.0, v + 0.5),
                ImVec2::new(0.0, v + 0.5),
                imgui_color(0xffffff, n.slash.opacity),
            );
            ImDrawList_PopClipRect(draw);
        }
    }
    unsafe fn number(&self, draw: *mut ImDrawList, n: &DrawNumber) {
        let font = self.selected_font(n.style);
        let pixels = self.font_pixels(n.pixels, n.style);
        // SAFETY: draw list and font belong to this context. Only vertices emitted by this text call are edited, within VtxBuffer.Size.
        unsafe {
            let text = |x: f32, y: f32, color: u32| {
                let first = (*draw).VtxBuffer.Size;
                ImDrawList_AddText_FontPtr(
                    draw,
                    font.raw(),
                    pixels,
                    ImVec2::new(n.position.x + x, n.position.y + y),
                    color,
                    n.text.as_ptr().cast(),
                    ptr::null(),
                    0.0,
                    ptr::null(),
                );
                if n.style == Style::Rise {
                    for i in first..(*draw).VtxBuffer.Size {
                        let vertex = &mut *(*draw).VtxBuffer.Data.add(i as usize);
                        vertex.pos.x =
                            n.position.x + (vertex.pos.x - n.position.x) * RISE_GLYPH_WIDTH;
                    }
                }
            };
            let color = imgui_color(n.color, n.opacity);
            if n.style == Style::Classic {
                let stroke = n.classic_stroke;
                let edge = imgui_color(0, n.opacity);
                for x in -1..=1 {
                    for y in -1..=1 {
                        if x != 0 || y != 0 {
                            text(x as f32 * stroke, y as f32 * stroke, edge);
                        }
                    }
                }
                text(0.0, 0.0, color);
                if n.critical != Critical::None {
                    let y = if n.critical == Critical::Positive {
                        n.position.y - 2.0 * stroke
                    } else {
                        n.position.y + n.pixels + 2.0 * stroke
                    };
                    ImDrawList_AddLine(
                        draw,
                        ImVec2::new(n.position.x, y),
                        ImVec2::new(n.position.x + n.advance, y),
                        edge,
                        4.0 * stroke,
                    );
                    ImDrawList_AddLine(
                        draw,
                        ImVec2::new(n.position.x, y),
                        ImVec2::new(n.position.x + n.advance, y),
                        color,
                        2.0 * stroke,
                    );
                }
                return;
            }
            self.glyph_effects(draw, n, font, pixels);
            let stroke = if n.style == Style::Rise {
                0.10 * n.pixels
            } else {
                0.5 * n.pixels / BASE_PIXELS
            };
            let edge = imgui_color(
                if n.style == Style::Rise { 0 } else { 0x17130f },
                n.opacity * if n.style == Style::Rise { 1.0 } else { 0.9 },
            );
            for x in -1..=1 {
                for y in -1..=1 {
                    if x != 0 || y != 0 {
                        let first = (*draw).VtxBuffer.Size;
                        text(0.0, 0.0, edge);
                        for i in first..(*draw).VtxBuffer.Size {
                            let vertex = &mut *(*draw).VtxBuffer.Data.add(i as usize);
                            vertex.pos.x += x as f32 * stroke;
                            vertex.pos.y += y as f32 * stroke;
                        }
                    }
                }
            }
            text(0.0, 0.0, color);
            if n.style != Style::Rise && n.critical != Critical::None {
                let scale = n.pixels / 60.0 * WILDS_CRIT_SCALE;
                let height = 48.0 * scale;
                self.sprite(
                    draw,
                    if n.critical == Critical::Positive {
                        20
                    } else {
                        21
                    },
                    Rect {
                        left: n.star.x - 44.5 * scale,
                        top: n.star.y - height * 0.5,
                        right: n.star.x + 43.5 * scale,
                        bottom: n.star.y + height * 0.5,
                    },
                    imgui_color(0xffffff, n.opacity),
                );
            }
        }
    }
    pub fn draw(&mut self, view: &View, numbers: &[DrawNumber]) {
        if numbers.is_empty() {
            return;
        }
        let draw = DrawList::background().raw();
        if draw.is_null() {
            return;
        }
        // SAFETY: each push/pop is balanced within this callback; effects are submitted before all digit fills.
        unsafe {
            ImDrawList_PushClipRect(
                draw,
                ImVec2::new(view.x, view.y),
                ImVec2::new(view.x + view.width, view.y + view.height),
                true,
            );
            for number in numbers {
                self.effect(draw, number);
            }
            for number in numbers {
                self.number(draw, number);
            }
            ImDrawList_PopClipRect(draw);
        }
    }
}
unsafe extern "C" fn shutdown(_context: *mut ImGuiContext, hook: *mut ImGuiContextHook) {
    // SAFETY: registration supplies this owned texture exactly once at context shutdown.
    CONTEXT_EPOCH.fetch_add(1, Ordering::Relaxed);
    unsafe {
        let texture = (*hook).UserData.cast::<ImTextureData>();
        igUnregisterUserTexture(texture);
        ImTextureData_destroy(texture);
    }
}
