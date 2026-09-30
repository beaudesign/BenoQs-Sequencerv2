//! The snapshot buffer: audio thread to render thread, one writer, one reader, both
//! wait-free (SPEC-0001 O6, decision D3, `specs/SPEC-0001/o6-test-plan.md`).
//!
//! Three slots of atomic words and one atomic `middle` that holds the index of the slot
//! between the two ends plus a `DIRTY` bit that says "the writer published this one and the
//! reader has not claimed it". The writer owns one slot and the reader owns one:
//!
//! - **publish** (writer): fill the owned slot, then `swap` it into `middle` with `DIRTY`
//!   set. What comes back is the slot that was in the middle, which is now free to write.
//! - **claim** (reader): if `middle` is dirty, `swap` the reader's own slot into it, clean.
//!   What comes back is the newest published slot, now owned by the reader.
//!
//! Each side does one atomic swap and no loop, so neither can make the other wait, and the
//! reader always gets the newest snapshot and never sees one half-written: the writer only
//! ever writes a slot the reader does not hold. `docs/02` §2 describes the writer as doing a
//! plain `store` of `published`; that cannot work, because the writer would not know which
//! of the other two slots the reader still holds, and the swap is what tells it.
//!
//! The slots are `AtomicU32` words, not `UnsafeCell`, so there is no `unsafe` here. The
//! word stores and loads are `Relaxed`: the `AcqRel` swaps are what order them.

use crate::sync::{Arc, AtomicU32, AtomicU64, Ordering};

const DIRTY: u32 = 0b100;
const INDEX: u32 = 0b011;

struct Shared {
    words: usize,
    slots: [Box<[AtomicU32]>; 3],
    /// Index of the middle slot, with `DIRTY` set if the reader has not claimed it yet.
    middle: AtomicU32,
    /// Number of publishes. Written only by the writer.
    published: AtomicU64,
}

pub struct Writer {
    shared: Arc<Shared>,
    /// The slot only the writer may touch.
    back: u32,
}

pub struct Reader {
    shared: Arc<Shared>,
    /// The slot only the reader may touch.
    front: u32,
}

/// Makes a buffer of three slots of `words` words each, all zero, and its two ends. The
/// only allocation the buffer ever does is here.
pub fn triple_buffer(words: usize) -> (Writer, Reader) {
    let slot = || (0..words).map(|_| AtomicU32::new(0)).collect::<Box<[AtomicU32]>>();
    let shared = Arc::new(Shared {
        words,
        slots: [slot(), slot(), slot()],
        middle: AtomicU32::new(1),
        published: AtomicU64::new(0),
    });
    (Writer { shared: shared.clone(), back: 0 }, Reader { shared, front: 2 })
}

impl Writer {
    /// Writes `data` (which must match the buffer word count / `shared.words`) into the
    /// slot the writer owns, then publishes it. Never waits.
    pub fn publish(&mut self, data: &[u32]) {
        assert_eq!(data.len(), self.shared.words, "publish: wrong slot length");
        let slot = &self.shared.slots[self.back as usize];
        for (cell, &w) in slot.iter().zip(data) {
            cell.store(w, Ordering::Relaxed);
        }
        let old = self.shared.middle.swap(self.back | DIRTY, Ordering::AcqRel);
        self.back = old & INDEX;
        self.shared.published.fetch_add(1, Ordering::Relaxed);
    }

    /// Number of publishes so far.
    pub fn published(&self) -> u64 {
        self.shared.published.load(Ordering::Relaxed)
    }
}

impl Reader {
    /// Takes the newest published slot if there is one newer than the last claim. Returns
    /// whether it did. Never waits.
    pub fn claim(&mut self) -> bool {
        if self.shared.middle.load(Ordering::Relaxed) & DIRTY == 0 {
            return false;
        }
        let old = self.shared.middle.swap(self.front, Ordering::AcqRel);
        self.front = old & INDEX;
        true
    }

    /// Copies the claimed slot into `out` (which must match the buffer word count /
    /// `shared.words`). Before the first claim that is the initial all-zero slot.
    pub fn read(&self, out: &mut [u32]) {
        assert_eq!(out.len(), self.shared.words, "read: wrong slot length");
        let slot = &self.shared.slots[self.front as usize];
        for (o, cell) in out.iter_mut().zip(slot.iter()) {
            *o = cell.load(Ordering::Relaxed);
        }
    }

    /// Number of publishes so far.
    pub fn published(&self) -> u64 {
        self.shared.published.load(Ordering::Relaxed)
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    fn slot(g: u32) -> [u32; 4] {
        [g, g.wrapping_mul(3), !g, g ^ 0xA5A5_A5A5]
    }

    fn read4(r: &Reader) -> [u32; 4] {
        let mut out = [0; 4];
        r.read(&mut out);
        out
    }

    #[test]
    fn t1_reader_gets_the_newest_publish_and_a_second_claim_reports_nothing_new() {
        let (mut w, mut r) = triple_buffer(4);
        w.publish(&slot(1));
        w.publish(&slot(2));
        assert!(r.claim());
        assert_eq!(read4(&r), slot(2));
        assert!(!r.claim(), "nothing new since the last claim");
        assert_eq!(read4(&r), slot(2), "the claimed slot is still there");
        w.publish(&slot(3));
        assert!(r.claim());
        assert_eq!(read4(&r), slot(3));
    }

    #[test]
    fn t2_claim_before_any_publish_returns_the_zeroed_initial_slot() {
        let (_w, mut r) = triple_buffer(4);
        assert!(!r.claim());
        assert_eq!(read4(&r), [0; 4]);
        assert_eq!(r.published(), 0);
    }

    #[test]
    fn t3_a_thousand_publishes_with_no_reader_never_block_and_the_reader_sees_the_last() {
        let (mut w, mut r) = triple_buffer(4);
        for g in 1..=1000 {
            w.publish(&slot(g));
        }
        assert_eq!(w.published(), 1000);
        assert!(r.claim());
        assert_eq!(read4(&r), slot(1000));
        assert!(!r.claim());
        assert_eq!(r.published(), 1000);
    }

    #[test]
    fn t4_claims_never_go_backwards_and_report_new_exactly_when_something_was_published() {
        use crate::rng::Rng;
        for seed in 0..500u64 {
            let mut rng = Rng::new(seed);
            let (mut w, mut r) = triple_buffer(4);
            let mut latest = 0u32;
            let mut seen = 0u32;
            let mut published_since_claim = false;
            for step in 0..300 {
                if rng.next_below(2) == 0 {
                    latest += 1;
                    w.publish(&slot(latest));
                    published_since_claim = true;
                } else {
                    let new = r.claim();
                    assert_eq!(new, published_since_claim, "seed {seed} step {step}");
                    published_since_claim = false;
                    let got = read4(&r);
                    if new {
                        assert_eq!(got, slot(latest), "seed {seed} step {step}: a claim returns the newest");
                    }
                    assert!(got[0] >= seen, "seed {seed} step {step}: {} after {seen}", got[0]);
                    seen = got[0];
                }
            }
        }
    }
}
