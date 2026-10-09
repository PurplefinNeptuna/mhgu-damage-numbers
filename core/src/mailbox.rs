//! A fixed mailbox per producer, independent of its TLS lifetime. Every copied payload word is atomic.
use crate::capture::Record;
use core::sync::atomic::{AtomicU32, Ordering::SeqCst};
// Record is 248 bytes = 62 four-byte words. Numeric fields contain no padding
// or Rust references, so every bit can travel through AtomicU32 storage.
// The 256-entry ring is a power of two: index & 255 wraps without division.
const WORDS: usize = 62;
const CAPACITY: usize = 256;
// ticket identifies both the generation and whether a write is in progress.
// Atomic payload words avoid a Rust data race even if the producer wraps the
// ring and overwrites this slot during a read.
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
// Only the consumer owns this progress marker. The mailbox stays in static
// plugin storage, so ending the producer thread cannot destroy queued data.
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
        // Sequences start at 1; head is the last fully published sequence. Wrapping
        // integer operations keep the ring arithmetic defined across u32 overflow.
        // SeqCst places ticket and word operations in one shared atomic order.
        let next = self.head.load(SeqCst).wrapping_add(1);
        // SAFETY: Record is repr(C), has no padding, contains only numeric fields, and is exactly 62 words.
        let words: [u32; WORDS] = unsafe { core::mem::transmute(value) };
        let slot = &self.slots[next.wrapping_sub(1) as usize & (CAPACITY - 1)];
        // Mark busy before touching data; publish the even ticket after all words,
        // then advance head. A reader accepts only its expected even generation.
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
        // If the consumer fell behind, skip irretrievably overwritten generations
        // and resume at the oldest of the newest 256 records. Never block the game.
        if distance > CAPACITY as u32 {
            cursor.overwritten = cursor.overwritten.wrapping_add(distance - CAPACITY as u32);
            cursor.sequence = head.wrapping_sub(CAPACITY as u32);
        }
        let next = cursor.sequence.wrapping_add(1);
        let ticket = next << 1;
        // Advance even when this sample is torn; the next frame can read later
        // records instead of spinning on a producer currently writing this slot.
        cursor.sequence = next;
        let slot = &self.slots[next.wrapping_sub(1) as usize & (CAPACITY - 1)];
        if slot.ticket.load(SeqCst) != ticket {
            cursor.overwritten = cursor.overwritten.wrapping_add(1);
            return None;
        }
        // Check the ticket before and after copying. A changed generation/busy bit
        // means the snapshot may mix writes; discard it instead of decoding a Record.
        let words = core::array::from_fn(|i| slot.data[i].load(SeqCst));
        if slot.ticket.load(SeqCst) != ticket {
            cursor.overwritten = cursor.overwritten.wrapping_add(1);
            return None;
        }
        // SAFETY: All bit patterns are valid for Record's integers and floats. Ticket checks reject concurrent overwrite.
        Some(unsafe { core::mem::transmute::<[u32; WORDS], Record>(words) })
    }
}
