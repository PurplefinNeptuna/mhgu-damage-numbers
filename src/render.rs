//! Forge/ImGui rendering of copied draw commands. Assets are loaded once per context, never per hit.
use core::{
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
// Context shutdown advances this generation counter. Comparing both
// pointer and generation prevents reuse of a dead context's resource pointers
// if a new context is later allocated at the same address.
static CONTEXT_EPOCH: AtomicU32 = AtomicU32::new(0);
use damage_numbers_core::{presentation::*, *};
use forge::{
    imgui::{DrawList, ImFont as Font},
    sys::imgui::*,
};
// Only numeric metrics live in code; font and artwork bytes come from RomFS.
#[path = "assets.rs"]
mod assets;
use assets::{FONT_SCALE, RECTS};
// font is an atlas-owned font handle, and texture pointers are registered
// ImGui resources owned until context shutdown. rise_attempted records the
// single lazy slash-load attempt; repeated frames do not allocate or retry.
#[derive(Default)]
pub struct Resources {
    font: Option<Font>,
    context: *mut ImGuiContext,
    context_epoch: u32,
    wilds: *mut ImTextureData,
    rise: *mut ImTextureData,
    rise_attempted: bool,
    font_warned: bool,
    wilds_warned: bool,
    rise_warned: bool,
}
fn point(p: Point) -> ImVec2 {
    ImVec2::new(p.x, p.y)
}
impl Resources {
    /// The atlas owns file-loaded font data. Reinitialization discards pointers belonging to the preceding context.
    pub fn initialize(&mut self) {
        let context = forge::imgui::get_current_context() as *mut _;
        let epoch = CONTEXT_EPOCH.load(Ordering::Relaxed);
        // Repeated initialization of this live context keeps its existing assets.
        // A different context/generation clears handles before loading fresh ones.
        if self.context == context && self.context_epoch == epoch {
            return;
        }
        self.context = context;
        self.context_epoch = epoch;
        self.font = None;
        self.wilds = ptr::null_mut();
        self.rise = ptr::null_mut();
        self.rise_attempted = false;
        // Context invalidation must also run when every artwork file is missing.
        // Register it independently of textures so an address reused by a new
        // context cannot retain a stale atlas-owned font handle.
        unsafe {
            let mut hook = ImGuiContextHook {
                HookId: 0,
                Type: ImGuiContextHookType::Shutdown,
                Owner: 0,
                Callback: Some(invalidate_context),
                UserData: ptr::null_mut(),
            };
            igAddContextHook(context, &mut hook);
        }
        // Use Forge's actual RomFS mount prefix rather than assuming app:.
        // CString supplies the NUL-terminated path required by the native font loader.
        let path = alloc::ffi::CString::new(alloc::format!(
            "{}:/nativeNX/plugins/damage-numbers/OpenSans.ttf",
            forge::fs::romfs::mount_point()
        ))
        .unwrap();
        // SAFETY: the configuration constructor and destructor use Forge's active ImGui allocator. Glyph ranges are static and NUL-terminated.
        unsafe {
            let cfg = ImFontConfig_ImFontConfig();
            if !cfg.is_null() {
                (*cfg).FontDataOwnedByAtlas = true;
                // Missing optional fonts must return None, not trigger ImGui's
                // default load-error assertion before our readable-font fallback.
                (*cfg).Flags |= ImFontFlags::NoLoadError;
                // Glyph-range pairs cover space, digits 0..9 and fallback '?'; zero ends
                // this static list. FontDataOwnedByAtlas transfers loaded file data to ImGui,
                // so only the temporary configuration object is destroyed after loading.
                static RANGES: [ImWchar; 7] = [32, 32, 48, 57, 63, 63, 0];
                self.font = Font::load_with(&path, 24.0 * FONT_SCALE, &*cfg, Some(&RANGES));
                ImFontConfig_destroy(cfg);
            }
            self.wilds = self.texture(c"wilds-effects.rgba", 512, 256);
        }
        if self.font.is_none() && !self.font_warned {
            self.font_warned = true;
            log::warn!("[damage-numbers] digit font unavailable; using Forge font");
        }
        if self.wilds.is_null() && !self.wilds_warned {
            self.wilds_warned = true;
            log::warn!("[damage-numbers] effect atlas unavailable");
        }
    }
    /// Read one fixed-size RGBA asset directly into ImGui's texture buffer.
    /// Caller supplies the active context and the pinned asset dimensions.
    unsafe fn texture(
        &self,
        filename: &core::ffi::CStr,
        width: i32,
        height: i32,
    ) -> *mut ImTextureData {
        // The file guard closes on every return, including wrong size, allocation
        // failure or a short read. Texture ownership transfers only after success.
        struct File(ImFileHandle);
        impl Drop for File {
            fn drop(&mut self) {
                // SAFETY: File is created only for a non-null open handle, once.
                unsafe {
                    igImFileClose(self.0);
                }
            }
        }
        // SAFETY: Forge provides the current context/allocator; the filenames and
        // dimensions are internal constants, and the file length is checked first.
        unsafe {
            let io = forge::imgui::get_io();
            if (io.BackendFlags.0 & ImGuiBackendFlags::RendererHasTextures.0) == 0 {
                return ptr::null_mut();
            }
            let path = alloc::ffi::CString::new(alloc::format!(
                "{}:/nativeNX/plugins/damage-numbers/{}",
                forge::fs::romfs::mount_point(),
                filename.to_str().unwrap()
            ))
            .unwrap();
            let handle = igImFileOpen(path.as_ptr(), c"rb".as_ptr());
            if handle.is_null() {
                return ptr::null_mut();
            }
            let file = File(handle);
            // RGBA8 has four bytes per pixel. These assets have no encoded header
            // or decoder; reject both truncated files and unexpected trailing data.
            let bytes = width as u64 * height as u64 * 4;
            if igImFileGetSize(file.0) != bytes {
                return ptr::null_mut();
            }
            let texture = ImTextureData_ImTextureData();
            if texture.is_null() {
                return texture;
            }
            ImTextureData_Create(texture, ImTextureFormat::RGBA32, width, height);
            let pixels = ImTextureData_GetPixels(texture);
            if pixels.is_null() || igImFileRead(pixels, 1, bytes, file.0) != bytes {
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
    // Classic uses Forge's current font. Wilds/Rise use the namespaced digit
    // font when available, falling back to the current font if loading failed.
    fn selected_font(&self, style: Style) -> Font {
        if style != Style::Classic {
            if let Some(font) = self.font {
                return font;
            }
        }
        Font::current()
    }
    // The generated digit font's em size differs from its visible digit height.
    // FONT_SCALE makes the requested presentation height match the actual glyphs.
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
            // ImGui may supply metrics baked at another size. ratio converts those
            // metrics to requested pixels. Accumulate spacing separately from visible ink,
            // so layout and text drawing agree on both origin and bounding rectangle.
            let ratio = pixels / (*baked).Size;
            let mut advance = 0.0;
            let mut ink: Option<Rect> = None;
            for &byte in text.iter().take_while(|&&b| b != 0) {
                let glyph = ImFontBaked_FindGlyph(baked, byte as ImWchar);
                if glyph.is_null() {
                    return None;
                }
                let g = &*glyph;
                // X0/Y0/X1/Y1 are the glyph ink relative to its pen origin. Add the
                // current advance horizontally, then union this glyph with earlier ink.
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
            // Rise stretches digits horizontally by 1.08. Measurement must apply the
            // same stretch as the vertex edit in number, or placement would be too narrow.
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
    // Requirements: draw is the current context's live draw list, and the atlas
    // is registered with that context. Each RECTS row begins [x,y,width,height];
    // dividing by atlas dimensions converts its pixel rectangle to normalized UVs.
    unsafe fn sprite(&self, draw: DrawList, index: usize, target: Rect, color: u32) {
        if self.wilds.is_null() || index >= 22 {
            return;
        }
        let r = RECTS[index];
        // SAFETY: registered atlas and current draw list remain live during the render callback; rectangles index the pinned atlas.
        unsafe {
            draw.add_image(
                ImTextureData_GetTexRef(self.wilds),
                ImVec2::new(target.left, target.top),
                ImVec2::new(target.right, target.bottom),
                ImVec2::new(r[0] as f32 / 512.0, r[1] as f32 / 256.0),
                ImVec2::new((r[0] + r[2]) as f32 / 512.0, (r[1] + r[3]) as f32 / 256.0),
                color,
            );
        }
    }
    // Draw atlas shadows behind the font's real glyphs. Requirements: draw/font
    // belong to the active context and n's text is terminated within its buffer.
    // Metadata stores the shadow rectangle plus its embedded glyph ink rectangle.
    unsafe fn glyph_effects(&self, draw: DrawList, n: &DrawNumber, font: Font, pixels: f32) {
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
                // r[4..8] describes the glyph's inset x/y and ink width/height inside the
                // shadow sprite. Scale by real glyph ink, then subtract that inset so the
                // sprite's internal glyph matches the font glyph at exactly the same origin.
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
    // Submit Rise's critical slash only; ordinary hits have no slash texture work.
    // Requirements: draw and resources belong to the active context. The first
    // visible critical lazily creates the shared atlas for all later Rise hits.
    unsafe fn effect(&mut self, draw: DrawList, n: &DrawNumber) {
        if n.style != Style::Rise || n.critical == Critical::None || n.slash.opacity <= 0.0 {
            return;
        }
        // SAFETY: creation and draw submission run only in the active ImGui callback.
        unsafe {
            if !self.rise_attempted {
                self.rise_attempted = true;
                self.rise = self.texture(c"rise-slash.rgba", 64, 512);
                if self.rise.is_null() && !self.rise_warned {
                    self.rise_warned = true;
                    log::warn!("[damage-numbers] Rise slash unavailable");
                }
            }
            if self.rise.is_null() {
                return;
            }
            let points = slash_quad(n.slash);
            // Positive and negative critical artwork occupies the top/bottom half of
            // the same atlas. v selects that half; the quad keeps the projected anchor.
            let v = if n.critical == Critical::Positive {
                0.0
            } else {
                0.5
            };
            draw.push_clip_rect(
                ImVec2::new(n.slash.clip.left, n.slash.clip.top),
                ImVec2::new(n.slash.clip.right, n.slash.clip.bottom),
                true,
            );
            ImDrawList_AddImageQuad(
                draw.raw(),
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
            draw.pop_clip_rect();
        }
    }
    // Requirements: current draw/font/texture context and valid digit buffer.
    // Draw outlines before fills, then the style's critical marker. All alpha
    // comes from the copied command, so rendering never reads native hit objects.
    unsafe fn number(&self, draw: DrawList, n: &DrawNumber) {
        let font = self.selected_font(n.style);
        let pixels = self.font_pixels(n.pixels, n.style);
        // Model output is bounded ASCII. Borrow its terminated text once for the
        // stock text wrapper; malformed internal records produce no draw commands.
        let Some(value) = core::ffi::CStr::from_bytes_until_nul(&n.text)
            .ok()
            .and_then(|text| text.to_str().ok())
        else {
            return;
        };
        // SAFETY: draw list and font belong to this context. Only vertices emitted by this text call are edited, within VtxBuffer.Size.
        unsafe {
            let text = |x: f32, y: f32, color: u32| {
                // Remember the first new vertex so Rise's width stretch touches only this
                // text call. Earlier outlines, effects and unrelated UI vertices stay intact.
                let first = (*draw.raw()).VtxBuffer.Size;
                draw.add_text_with_font(
                    &font,
                    pixels,
                    ImVec2::new(n.position.x + x, n.position.y + y),
                    color,
                    value,
                );
                if n.style == Style::Rise {
                    for i in first..(*draw.raw()).VtxBuffer.Size {
                        let vertex = &mut *(*draw.raw()).VtxBuffer.Data.add(i as usize);
                        vertex.pos.x =
                            n.position.x + (vertex.pos.x - n.position.x) * RISE_GLYPH_WIDTH;
                    }
                }
            };
            let color = imgui_color(n.color, n.opacity);
            // Classic outlines are eight neighboring black text copies plus one fill.
            // Its critical marker is a two-stroke line above/below the text, not a star.
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
                    draw.add_line(
                        ImVec2::new(n.position.x, y),
                        ImVec2::new(n.position.x + n.advance, y),
                        edge,
                        4.0 * stroke,
                    );
                    draw.add_line(
                        ImVec2::new(n.position.x, y),
                        ImVec2::new(n.position.x + n.advance, y),
                        color,
                        2.0 * stroke,
                    );
                }
                return;
            }
            // Wilds/Rise start with per-digit atlas shadows, then add a text outline.
            // Rise's thicker black outline and width stretch share the same fill metrics.
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
                        let first = (*draw.raw()).VtxBuffer.Size;
                        // Emit at the normal origin, apply Rise stretch there, then shift these
                        // new vertices by the outline offset. Shifting first would stretch stroke
                        // spacing too and break the intended outline thickness.
                        text(0.0, 0.0, edge);
                        for i in first..(*draw.raw()).VtxBuffer.Size {
                            let vertex = &mut *(*draw.raw()).VtxBuffer.Data.add(i as usize);
                            vertex.pos.x += x as f32 * stroke;
                            vertex.pos.y += y as f32 * stroke;
                        }
                    }
                }
            }
            text(0.0, 0.0, color);
            if n.style != Style::Rise && n.critical != Critical::None {
                // Atlas entries 20/21 are positive/negative Wilds stars. The 60-pixel
                // reference and 1.5 multiplier reproduce r25's enlarged critical marker.
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
    /// The Mod Options preview's frame: a thin border with no fill, a filled label tab at its top
    /// left, and a dot at each example spot that is on screen. Every size follows the view's
    /// height (reference 720), so no measure is a fixed pixel count. Drawn on the background list
    /// before the preview's numbers, so it stays under them.
    pub fn preview_frame(&self, view: &View, frame: Rect, dots: [Option<Point>; 3]) {
        const LABEL: &str = "Damage number preview";
        let draw = DrawList::background();
        if draw.raw().is_null() {
            return;
        }
        let k = view.height / 720.0;
        let (min, max) = (ImVec2::new(frame.left, frame.top), ImVec2::new(frame.right, frame.bottom));
        // A dark stroke under the light one keeps the border visible on bright ground.
        draw.add_rect(min, max, imgui_color(0x0c0906, 0.55), 0.0, 2.5 * k, ImDrawFlags(0));
        draw.add_rect(min, max, imgui_color(0xf2e8d0, 0.9), 0.0, 1.0 * k, ImDrawFlags(0));
        // Forge's default font, as Classic numbers use; calc_text_size measures at its own size.
        let font = Font::current();
        let size = 15.0 * k;
        let base = Font::current_size();
        let width = if base > 0.0 { forge::imgui::calc_text_size(LABEL, false, -1.0).x * size / base } else { 0.0 };
        let (pad_x, pad_y) = (8.0 * k, 4.0 * k);
        draw.add_rect_filled(
            min,
            ImVec2::new(frame.left + width + 2.0 * pad_x, frame.top + size + 2.0 * pad_y),
            imgui_color(0x702c24, 0.94),
            0.0,
        );
        draw.add_text_with_font(&font, size, ImVec2::new(frame.left + pad_x, frame.top + pad_y), imgui_color(0xf3d36b, 1.0), LABEL);
        for dot in dots.into_iter().flatten() {
            let at = ImVec2::new(dot.x, dot.y);
            draw.add_circle_filled(at, 6.0 * k, imgui_color(0x0c0906, 0.6), 0);
            draw.add_circle_filled(at, 4.0 * k, imgui_color(0xffffff, 0.95), 0);
        }
    }
    pub fn draw(&mut self, view: &View, numbers: &[DrawNumber]) {
        if numbers.is_empty() {
            return;
        }
        // Use the background list so numbers are independent of the settings window.
        // Render all slash effects first, then all digits, retaining r25 layering.
        let draw = DrawList::background();
        if draw.raw().is_null() {
            return;
        }
        // SAFETY: each push/pop is balanced within this callback; effects are submitted before all digit fills.
        unsafe {
            draw.push_clip_rect(
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
            draw.pop_clip_rect();
        }
    }
}
// Called by ImGui while the context and registered texture still exist.
// The resource owner discards these raw pointers on the next initialization;
// this callback performs the actual unregister/free exactly once per texture.
unsafe extern "C" fn shutdown(_context: *mut ImGuiContext, hook: *mut ImGuiContextHook) {
    // SAFETY: registration supplies this owned texture exactly once at context shutdown.
    unsafe {
        let texture = (*hook).UserData.cast::<ImTextureData>();
        igUnregisterUserTexture(texture);
        ImTextureData_destroy(texture);
    }
}

// No resources are owned by this callback; it invalidates handles even when
// failed asset loads registered no texture cleanup callbacks at all.
unsafe extern "C" fn invalidate_context(_context: *mut ImGuiContext, _hook: *mut ImGuiContextHook) {
    CONTEXT_EPOCH.fetch_add(1, Ordering::Relaxed);
}
