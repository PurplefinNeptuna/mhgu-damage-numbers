//! Eight gateway adapters. Scope borrows end before assembly resumes native execution.
use crate::mailbox::Mailbox;
use crate::*;

/// gateway.S saves all general and VFP registers before calling Rust. Keep these offsets exact.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RawFrame {
    pub target: u32,
    pub spare: u32,
    pub cpsr: u32,
    pub fpscr: u32,
    pub high_d: [u64; 16],
    pub low_d: [u64; 16],
    pub r: [u32; 14],
}
impl Default for RawFrame {
    fn default() -> Self {
        Self {
            target: 0,
            spare: 0,
            cpsr: 0,
            fpscr: 0,
            high_d: [0; 16],
            low_d: [0; 16],
            r: [0; 14],
        }
    }
}
const _: () = assert!(core::mem::size_of::<RawFrame>() == 328);
const _: () = assert!(core::mem::offset_of!(RawFrame, r) == 272);
const _: () = assert!(core::mem::offset_of!(RawFrame, low_d) == 144);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Feedback {
    pub attack: u32,
    pub flags: u32,
    pub active: u8,
    pub effect: u8,
    pub critical: u8,
    pub reserved: u8,
}
/// Integer identities are copied evidence only; the consumer never follows their addresses.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Record {
    pub kind: u32,
    pub serial: u32,
    pub actor: u32,
    pub attack: u32,
    pub monster: u32,
    pub flags: u32,
    pub damage_type: u32,
    pub part: u32,
    pub critical: u32,
    pub effect: u32,
    pub damage: i32,
    pub hzv: i32,
    pub position: Vec3,
    pub view: [f32; 16],
    pub projection: [f32; 16],
    pub rect: [i32; 4],
    pub feedback: [Feedback; 2],
    pub attack_counter: u32,
    pub source_tags: u32,
    pub player: u32,
    pub tick: u64,
}
const _: () = assert!(core::mem::size_of::<Record>() == 248);
const _: () = assert!(core::mem::offset_of!(Record, tick) == 240);
const _: () = assert!(core::mem::offset_of!(Record, position) == 48);
const _: () = assert!(core::mem::offset_of!(Record, feedback) == 204);
/// The integration must check mappings and copy values, never return a reference into game memory.
pub trait Memory {
    fn base(&self) -> u32;
    fn readable(&self, address: u32, bytes: usize) -> bool;
    fn word(&self, address: u32) -> u32;
    fn byte(&self, address: u32) -> u8;
    fn copy(&self, address: u32, out: &mut [u8]) -> bool;
    fn tick(&self) -> u64;
    fn cell(&self, offset: u32) -> u32 {
        let address = self.base().wrapping_add(offset);
        if self.readable(address, 4) {
            self.word(address)
        } else {
            0
        }
    }
    fn local_player(&self) -> u32 {
        let session = self.cell(0x188ca48);
        let table = self.cell(0x1885b58);
        if !self.readable(session, 0x34) || !self.readable(table, 0x2c) {
            return 0;
        }
        let index = self.word(session + 0x30);
        if index >= 4 {
            return 0;
        }
        let player = self.word(table + 0x1c + 4 * index);
        if self.readable(player, 4) { player } else { 0 }
    }
}
fn resolve(memory: &impl Memory, kind: u8, object: u32) -> Owner {
    if (kind != 0 && kind != 3) || !memory.readable(object, 4) {
        return Owner::Unknown;
    }
    let local = memory.local_player();
    let table = memory.cell(0x1885b58);
    if local == 0 || !memory.readable(table, 0x2c) {
        return Owner::Unknown;
    }
    if object == local {
        return Owner::LocalPlayer;
    }
    for i in 0..4 {
        if object == memory.word(table + 0x1c + 4 * i) {
            return Owner::RemotePlayer;
        }
    }
    if kind != 3 || !memory.readable(object, 0x3ac8) || memory.word(object + 0x13c8) != 0 {
        return Owner::Unknown;
    }
    let work = memory.word(object + 0x3ac4);
    if !memory.readable(work, 2) {
        return Owner::Unknown;
    }
    let master = memory.byte(work + 1);
    if master >= 4 {
        return Owner::Unknown;
    }
    let hunter = memory.word(table + 0x1c + 4 * master as u32);
    if !memory.readable(hunter, 4) {
        return Owner::Unknown;
    }
    if hunter == local {
        Owner::LocalPalico
    } else {
        Owner::RemotePalico
    }
}
pub fn attack_owner(memory: &impl Memory, hit: u32) -> Owner {
    if !memory.readable(hit, 0xb8) {
        return Owner::Unknown;
    }
    let primary = memory.byte(hit + 0x10);
    if primary == 0 || primary == 3 {
        resolve(memory, primary, memory.word(hit + 0xc))
    } else {
        resolve(memory, memory.byte(hit + 0x6e), memory.word(hit + 0x70))
    }
}
fn copy_value<T: Copy>(memory: &impl Memory, address: u32, out: &mut T) -> bool {
    // SAFETY: every caller supplies a fully initialized numeric-only POD value. The memory adapter checks the source span.
    let bytes = unsafe {
        core::slice::from_raw_parts_mut((out as *mut T).cast::<u8>(), core::mem::size_of::<T>())
    };
    memory.copy(address, bytes)
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Scope {
    pub kind: u32,
    pub caller: u32,
    pub sp: u32,
    pub subject: u32,
    pub monster: u32,
    pub auxiliary: u32,
    pub owner: Owner,
    pub hzv: i16,
    pub position: Vec3,
}
/// One lazily allocated state per native thread. Its mailbox belongs to the plugin, not this allocation.
pub struct CaptureState {
    pub slot: Option<usize>,
    pub depth: usize,
    pub serial: u32,
    pub quarantine: bool,
    pub stack: [Scope; 64],
    pub last_damage: Record,
}
impl CaptureState {
    pub fn new(slot: Option<usize>) -> Self {
        Self {
            slot,
            depth: 0,
            serial: 0,
            quarantine: false,
            stack: [Scope::default(); 64],
            last_damage: Record::default(),
        }
    }
    fn find(&self, kind: u32) -> Option<usize> {
        (0..self.depth).rev().find(|&i| self.stack[i].kind == kind)
    }
    fn publish(&mut self, mut record: Record, memory: &impl Memory, mailbox: &Mailbox) -> Record {
        if record.kind == 1 || record.kind == 3 {
            record.player = memory.local_player();
            record.tick = memory.tick();
        }
        self.serial = self.serial.wrapping_add(1);
        record.serial = self.serial;
        mailbox.publish(record);
        record
    }
    /// Return whether assembly may substitute LR. Overflow passes through until the existing chain unwinds.
    pub fn enter(&mut self, kind: u32, frame: &RawFrame, sp: u32, memory: &impl Memory) -> bool {
        if self.slot.is_none() {
            return false;
        }
        if self.quarantine || self.depth == 64 {
            self.quarantine = true;
            return false;
        }
        let mut scope = Scope {
            kind,
            caller: frame.r[13],
            sp,
            hzv: -1,
            ..Scope::default()
        };
        // Gateway wire IDs: 0 collision, 1 finalized damage, 2 feedback, 3 viewport;
        // 6 melee HZV and 7 shot HZV. Taps 4/5 do not substitute a return address.
        match kind {
            0 => {
                self.last_damage = Record::default();
                scope.subject = frame.r[1];
                scope.monster = frame.r[2];
                scope.owner = attack_owner(memory, scope.subject);
                copy_value(memory, frame.r[3], &mut scope.position);
                if memory.readable(sp.wrapping_add(8), 4) {
                    scope.auxiliary = memory.word(sp.wrapping_add(8));
                }
            }
            1 => {
                scope.subject = frame.r[0];
                scope.auxiliary = frame.r[1];
            }
            2 | 3 => scope.subject = frame.r[0],
            6 => scope.monster = frame.r[1],
            7 => {
                scope.subject = frame.r[1];
                scope.monster = frame.r[2];
                if memory.readable(sp.wrapping_add(4), 4) {
                    scope.auxiliary = memory.word(sp.wrapping_add(4));
                }
            }
            _ => return false,
        }
        self.stack[self.depth] = scope;
        self.depth += 1;
        true
    }
    /// Pop exactly one substituted return. A bad SP is a gateway contract error, not a guessed continuation.
    pub fn leave(
        &mut self,
        frame: &RawFrame,
        sp: u32,
        active: bool,
        memory: &impl Memory,
        mailbox: &Mailbox,
    ) -> Option<u32> {
        if self.depth == 0 {
            return None;
        }
        self.depth -= 1;
        let scope = self.stack[self.depth];
        if scope.sp != sp {
            return None;
        }
        if self.quarantine {
            if self.depth == 0 {
                self.quarantine = false;
            }
            return Some(scope.caller);
        }
        if !active {
            return Some(scope.caller);
        }
        match scope.kind {
            1 => {
                if let Some(index) = self.find(0) {
                    let hit = self.stack[index];
                    if hit.monster == scope.subject && memory.readable(hit.subject, 0xb8) {
                        let damage_type = memory.byte(hit.subject + 0x3a);
                        let amount = frame.r[0] as i32;
                        if damage_outcome(damage_type, amount) {
                            let tags = memory.byte(hit.subject + 0x10) as u32
                                | ((memory.byte(hit.subject + 0x6d) as u32) << 8)
                                | ((memory.byte(hit.subject + 0x6e) as u32) << 16);
                            let source = source_outcome(tags, hit.owner);
                            if matches!(hit.owner, Owner::LocalPlayer | Owner::LocalPalico)
                                || source.is_some_and(|s| s != Source::Attack)
                            {
                                let mut record = Record {
                                    kind: 1,
                                    actor: hit.owner as u32,
                                    attack: hit.subject,
                                    monster: hit.monster,
                                    part: scope.auxiliary,
                                    hzv: hit.hzv as i32,
                                    damage: amount,
                                    damage_type: damage_type as u32,
                                    flags: memory.word(hit.subject + 0x30),
                                    critical: memory.byte(hit.subject + 0x54) as u32,
                                    effect: memory.byte(hit.subject + 6) as u32,
                                    source_tags: tags,
                                    position: hit.position,
                                    ..Record::default()
                                };
                                record.feedback[0].reserved = source.map_or(255, |s| s as u8);
                                let player = memory.local_player();
                                let mut counter = [0; 2];
                                if memory.copy(player.wrapping_add(0x2522), &mut counter) {
                                    record.attack_counter = u16::from_le_bytes(counter) as u32;
                                }
                                self.last_damage = self.publish(record, memory, mailbox);
                            }
                        }
                    }
                }
            }
            2 => {
                if scope.subject == memory.local_player() {
                    let mut record = Record {
                        kind: 2,
                        actor: scope.subject,
                        effect: frame.r[0] & 255,
                        ..Record::default()
                    };
                    for i in 0..2 {
                        let attack = scope.subject.wrapping_add(0x3020 + i as u32 * 0xb8);
                        if memory.readable(attack, 0xb8) {
                            record.feedback[i] = Feedback {
                                attack,
                                flags: memory.word(attack + 0x30),
                                active: memory.byte(attack + 5),
                                effect: memory.byte(attack + 6),
                                critical: memory.byte(attack + 0x54),
                                reserved: 0,
                            };
                        }
                    }
                    let mut counter = [0; 2];
                    if memory.copy(scope.subject.wrapping_add(0x2522), &mut counter) {
                        record.attack_counter = u16::from_le_bytes(counter) as u32;
                    }
                    let last = self.last_damage;
                    if last.serial != 0
                        && last.player == scope.subject
                        && last.attack_counter == record.attack_counter
                        && bounce_match(record.effect as u8, record.feedback, last)
                    {
                        record.attack = last.attack;
                        record.monster = last.monster;
                        record.damage = last.serial as i32;
                        record.player = last.player;
                        record.tick = last.tick;
                        record.hzv = 1;
                    }
                    self.last_damage = Record::default();
                    self.publish(record, memory, mailbox);
                }
            }
            7 => {
                if let Some(index) = self.find(0) {
                    let hit = &mut self.stack[index];
                    if hit.monster == scope.monster
                        && hit.subject == scope.subject
                        && memory.readable(scope.subject.wrapping_add(0x30), 4)
                        && (memory.word(scope.subject.wrapping_add(0x30)) & 1) != 0
                    {
                        let mut factor = 0.0f32;
                        if copy_value(memory, scope.auxiliary, &mut factor) {
                            let hzv = factor * 100.0;
                            if hzv.is_finite() && (0.0..=32767.0).contains(&hzv) {
                                hit.hzv = hzv as i16;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Some(scope.caller)
    }
    pub fn tap(&mut self, kind: u32, frame: &RawFrame, memory: &impl Memory, mailbox: &Mailbox) {
        if self.quarantine || self.slot.is_none() {
            return;
        }
        if kind == 5 {
            if let (Some(hit), Some(hzv)) = (self.find(0), self.find(6)) {
                if self.stack[hit].monster == self.stack[hzv].monster {
                    let value = if frame.r[0] <= 1 {
                        f32::from_bits(frame.low_d[if frame.r[0] == 1 { 11 } else { 10 }] as u32)
                    } else {
                        f32::NAN
                    };
                    self.stack[hit].hzv = if value.is_finite() && (0.0..=32767.0).contains(&value) {
                        value as i16
                    } else {
                        -1
                    };
                }
            }
        } else if kind == 4 {
            if let Some(index) = self.find(3) {
                let viewport = self.stack[index];
                if memory.readable(viewport.subject, 0x28) {
                    let camera = memory.word(viewport.subject + 0x24);
                    if memory.readable(camera, 4)
                        && memory.word(camera) == memory.base().wrapping_add(0x1730d0c)
                    {
                        let mut record = Record {
                            kind: 3,
                            actor: camera,
                            monster: viewport.subject,
                            ..Record::default()
                        };
                        if copy_value(memory, frame.r[1], &mut record.view)
                            && copy_value(memory, frame.r[2], &mut record.projection)
                            && copy_value(memory, frame.r[3], &mut record.rect)
                        {
                            self.publish(record, memory, mailbox);
                        }
                    }
                }
            }
        }
    }
}
fn bounce_match(resolved: u8, attacks: [Feedback; 2], last: Record) -> bool {
    if resolved != 0
        || last.effect != 0
        || last.actor != Owner::LocalPlayer as u32
        || last.source_tags as u8 != 0
        || (last.source_tags >> 8) as u8 != 1
    {
        return false;
    }
    let (first, second) = (attacks[0].active != 0, attacks[1].active != 0);
    if first == second {
        return false;
    }
    let attack = attacks[if first { 0 } else { 1 }];
    last.attack == attack.attack
        && last.flags == attack.flags
        && last.critical as u8 == attack.critical
        && attack.effect == 0
}
