//! A fixed mailbox per producer, independent of its TLS lifetime. Every copied payload word is atomic.
use crate::capture::Record;
use core::sync::atomic::{AtomicU32, Ordering::SeqCst};
const WORDS: usize = 62;
const CAPACITY: usize = 256;
struct Slot {
    ticket: AtomicU32,
    data: [AtomicU32; WORDS],
}
impl Slot {
    const fn new() -> Self {
        Self {
            ticket: AtomicU32::new(0),
            data: [const { AtomicU32::new(0) }; WORDS],
        }
    }
}
pub struct Mailbox {
    slots: [Slot; CAPACITY],
    head: AtomicU32,
}
#[derive(Default)]
pub struct Cursor {
    pub sequence: u32,
    pub overwritten: u32,
}
impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}
impl Mailbox {
    pub const fn new() -> Self {
        Self {
            slots: [const { Slot::new() }; CAPACITY],
            head: AtomicU32::new(0),
        }
    }
    /// One producer owns this mailbox. Odd tickets announce overwrite; even tickets publish complete snapshots.
    pub fn publish(&self, value: Record) {
        let next = self.head.load(SeqCst).wrapping_add(1);
        // SAFETY: Record is repr(C), has no padding, contains only numeric fields, and is exactly 62 words.
        let words: [u32; WORDS] = unsafe { core::mem::transmute(value) };
        let slot = &self.slots[next.wrapping_sub(1) as usize & (CAPACITY - 1)];
        slot.ticket.store((next << 1) | 1, SeqCst);
        for (cell, bits) in slot.data.iter().zip(words) {
            cell.store(bits, SeqCst);
        }
        slot.ticket.store(next << 1, SeqCst);
        self.head.store(next, SeqCst);
    }
    /// The render thread alone owns Cursor. Never spin or retry a torn snapshot.
    pub fn read(&self, cursor: &mut Cursor) -> Option<Record> {
        let head = self.head.load(SeqCst);
        if head == cursor.sequence {
            return None;
        }
        let distance = head.wrapping_sub(cursor.sequence);
        if distance > CAPACITY as u32 {
            cursor.overwritten = cursor.overwritten.wrapping_add(distance - CAPACITY as u32);
            cursor.sequence = head.wrapping_sub(CAPACITY as u32);
        }
        let next = cursor.sequence.wrapping_add(1);
        let ticket = next << 1;
        cursor.sequence = next;
        let slot = &self.slots[next.wrapping_sub(1) as usize & (CAPACITY - 1)];
        if slot.ticket.load(SeqCst) != ticket {
            cursor.overwritten = cursor.overwritten.wrapping_add(1);
            return None;
        }
        let words = core::array::from_fn(|i| slot.data[i].load(SeqCst));
        if slot.ticket.load(SeqCst) != ticket {
            cursor.overwritten = cursor.overwritten.wrapping_add(1);
            return None;
        }
        // SAFETY: All bit patterns are valid for Record's integers and floats. Ticket checks reject concurrent overwrite.
        Some(unsafe { core::mem::transmute::<[u32; WORDS], Record>(words) })
    }
}
