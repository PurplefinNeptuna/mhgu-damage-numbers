//! Temporary r29 diagnostic output. Only the render consumer calls this code.
use damage_numbers_core::{capture::Record, presentation::*, Event, HitKey};

// Identity and viewport metadata are copied values, never deferred native reads.
type CameraKey = (u32, u32, [i32; 4], [u32; 5]);
#[derive(Default)]
pub struct Diagnostics {
    cameras: [Option<CameraKey>; 16],
    hits: [Option<HitKey>; 32],
    displays: usize,
    last_display: Option<[u32; 8]>,
}
impl Diagnostics {
    // Bitwise comparison also deduplicates unavailable/non-finite display inputs.
    // Budgets survive player/camera/context changes and stop at process lifetime limits.
    pub fn display(&mut self, values: [f32; 8]) {
        let bits = values.map(f32::to_bits);
        if self.last_display == Some(bits) || self.displays == 4 {
            return;
        }
        self.last_display = Some(bits);
        self.displays += 1;
        log::error!("[DN-DIAG] display#{} size={:?} framebuffer_scale={:?} main_viewport_pos_size={:?}",
            self.displays, &values[..2], &values[2..4], &values[4..]);
    }
    pub fn camera(&mut self, record: &Record) {
        if record.kind != 3 {
            return;
        }
        let key = (record.actor, record.monster, record.rect,
            [record.attack, record.attack_counter, record.flags, record.source_tags, record.damage_type]);
        if self.cameras.contains(&Some(key)) {
            return;
        }
        let Some(index) = self.cameras.iter().position(Option::is_none) else { return };
        self.cameras[index] = Some(key);
        log::error!("[DN-DIAG] camera#{}", index + 1);
        camera_record(record);
        if index == 15 {
            log::error!("[DN-DIAG] camera limit reached (16); hit snapshots continue independently");
        }
    }
    // Log the selected camera again with each first-visible hit: its matrices can
    // move without changing identity. No native hook formats or writes a log line.
    pub fn hit(&mut self, event: Event, camera: &Record, view: &View,
        native_camera_used: bool, now: u64, number: Option<&DrawNumber>) {
        if self.hits.contains(&Some(event.key)) {
            return;
        }
        let Some(index) = self.hits.iter().position(Option::is_none) else { return };
        self.hits[index] = Some(event.key);
        let projected = event.position.and_then(|p| project(p, view));
        log::error!("[DN-DIAG] hit#{} key={:?} damage={} xyz={:?} captured_seconds={:.6} now_tick={} native_camera_used={}",
            index + 1, event.key, event.total_damage, event.position, event.captured_at, now, native_camera_used);
        log::error!("[DN-DIAG] projection={:?} fallback={} display={:?}", projected, projected.is_none(),
            [view.x, view.y, view.width, view.height]);
        if let Some(number) = number {
            log::error!("[DN-DIAG] text_xy={:?} placement_offset={:?} pixels={} style={:?}",
                number.position, number.offset, number.pixels, number.style);
        } else {
            log::error!("[DN-DIAG] text omitted by layout");
        }
        camera_record(camera);
        if index == 31 {
            log::error!("[DN-DIAG] hit limit reached (32); restart the game for another sample");
        }
    }
}
// ERROR severity is deliberate for this temporary candidate: stock Forge's
// default filter is ERROR. Each short line fits both Forge's 512-byte writers.
fn camera_record(record: &Record) {
    log::error!("[DN-DIAG] selected camera=0x{:08x} viewport=0x{:08x} scene_texture=0x{:08x} test_camera=0x{:08x} tick={} player=0x{:08x}",
        record.actor, record.monster, record.attack, record.attack_counter, record.tick, record.player);
    log::error!("[DN-DIAG] rect={:?} metadata_valid={} visible={} subscene_number_mode_display={:?}",
        record.rect, record.damage_type == 1, record.flags, record.source_tags.to_le_bytes());
    for (name, matrix) in [("view", &record.view), ("projection", &record.projection)] {
        for (row, values) in matrix.chunks_exact(4).enumerate() {
            log::error!("[DN-DIAG] {}[{}]={:.9e},{:.9e},{:.9e},{:.9e}",
                name, row, values[0], values[1], values[2], values[3]);
        }
    }
}
