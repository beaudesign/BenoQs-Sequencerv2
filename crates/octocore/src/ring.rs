//! The command ring: main thread to audio thread, one producer, one consumer, both
//! wait-free (SPEC-0001 O6, decision D2, `specs/SPEC-0001/o6-test-plan.md`).
//!
//! It is an *overwrite* ring. The producer owns a private counter `head`, writes the slot
//! `head % capacity`, and publishes the new `head`. It never reads anything the consumer
//! writes, so a full ring cannot make it wait or fail: the oldest commands are simply
//! overwritten. The consumer owns a private `tail`. If it finds `head - tail` larger than
//! the capacity it was lapped, jumps forward to the oldest command still there, and counts
//! the ones it skipped as dropped. That is the policy of `docs/02` §2: "the oldest command
//! is dropped and a counter increments".
//!
//! A command is three 64-bit words, stored as atomics, so there is no `unsafe` here and
//! `docs/02` §3 still holds. Each slot has a stamp, a seqlock: the producer sets it to an
//! odd number before it writes the words and to an even one after, and the consumer checks
//! that the stamp is the even number it expects both before and after it reads. A slot that
//! was overwritten while the consumer read it fails that check and is counted as dropped,
//! never returned half old and half new. The payload stores are `Release` and the loads
//! `Acquire`, which is what orders the odd stamp before any new word the consumer might see
//! (loom checks this; see `tests/loom_sync.rs`).
//!
//! Stamps and counters are 64 bits. They would wrap after 2^63 pushes, about 292 years at a
//! billion commands a second.

use crate::sync::{Arc, AtomicU64, Ordering};
use crate::wire::Words;

/// Counters. `pushed == received + dropped + pending` whenever no push or pop is in flight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RingStats {
    /// Commands the producer has pushed.
    pub pushed: u64,
    /// Commands the consumer found overwritten (or otherwise unusable) and skipped.
    pub dropped: u64,
    /// Commands the consumer has taken.
    pub received: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingError {
    /// The capacity must be a power of two and at least 2.
    BadCapacity,
}

/// How many slots one `pop` will examine before it gives up and returns `None`. A `pop`
/// only loses a slot to a producer that has lapped it, so more than a couple of attempts
/// means the producer is pushing faster than the consumer can read: the bound keeps `pop`
/// wait-free, and the commands that are left are found by the next call.
const MAX_ATTEMPTS: usize = 4;

struct Slot {
    stamp: AtomicU64,
    words: [AtomicU64; 3],
}

struct Shared {
    slots: Box<[Slot]>,
    mask: u64,
    /// Commands published so far. Written only by the producer.
    head: AtomicU64,
    /// Written only by the consumer.
    dropped: AtomicU64,
    received: AtomicU64,
}

impl Shared {
    fn stats(&self) -> RingStats {
        RingStats {
            pushed: self.head.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            received: self.received.load(Ordering::Relaxed),
        }
    }
}

pub struct Sender {
    shared: Arc<Shared>,
    head: u64,
}

pub struct Receiver {
    shared: Arc<Shared>,
    tail: u64,
}

/// Makes a ring of `capacity` slots (a power of two, at least 2) and its two ends. The
/// only allocation the ring ever does is here.
pub fn command_ring(capacity: usize) -> Result<(Sender, Receiver), RingError> {
    if capacity < 2 || !capacity.is_power_of_two() {
        return Err(RingError::BadCapacity);
    }
    let slots: Box<[Slot]> = (0..capacity)
        .map(|_| Slot { stamp: AtomicU64::new(0), words: [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)] })
        .collect();
    let shared = Arc::new(Shared {
        slots,
        mask: capacity as u64 - 1,
        head: AtomicU64::new(0),
        dropped: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });
    Ok((Sender { shared: shared.clone(), head: 0 }, Receiver { shared, tail: 0 }))
}

impl Sender {
    /// Publishes one command. Never waits, never fails, never allocates. If the ring is
    /// full, this overwrites the oldest command.
    pub fn push(&mut self, w: Words) {
        let i = self.head;
        let slot = &self.shared.slots[(i & self.shared.mask) as usize];
        // Odd: this slot is being written. Relaxed is enough because every payload store
        // below is Release, and a Release store cannot be reordered before an earlier store.
        slot.stamp.store(2 * i + 1, Ordering::Relaxed);
        for (cell, word) in slot.words.iter().zip(w) {
            cell.store(word, Ordering::Release);
        }
        // Even: command `i` is complete.
        slot.stamp.store(2 * (i + 1), Ordering::Release);
        self.head = i + 1;
        self.shared.head.store(i + 1, Ordering::Release);
    }

    pub fn stats(&self) -> RingStats {
        self.shared.stats()
    }
}

impl Receiver {
    /// Takes the oldest command still in the ring. `None` means there is nothing to take
    /// right now, or that `MAX_ATTEMPTS` slots in a row were overwritten by a producer that
    /// is lapping this consumer while it read (some commands may then still be pending: the
    /// next call finds them, or counts them as dropped).
    pub fn pop(&mut self) -> Option<Words> {
        let shared = &*self.shared;
        let capacity = shared.mask + 1;
        for _ in 0..MAX_ATTEMPTS {
            let head = shared.head.load(Ordering::Acquire);
            if self.tail == head {
                return None;
            }
            let behind = head - self.tail;
            if behind > capacity {
                // Lapped: the producer has overwritten the commands before `head - capacity`.
                let lost = behind - capacity;
                self.tail += lost;
                shared.dropped.fetch_add(lost, Ordering::Relaxed);
            }
            let i = self.tail;
            let slot = &shared.slots[(i & shared.mask) as usize];
            let want = 2 * (i + 1);
            let before = slot.stamp.load(Ordering::Acquire);
            let w = [
                slot.words[0].load(Ordering::Acquire),
                slot.words[1].load(Ordering::Acquire),
                slot.words[2].load(Ordering::Acquire),
            ];
            let after = slot.stamp.load(Ordering::Relaxed);
            self.tail += 1;
            if before == want && after == want {
                shared.received.fetch_add(1, Ordering::Relaxed);
                return Some(w);
            }
            // Overwritten while it was being read. It is gone.
            shared.dropped.fetch_add(1, Ordering::Relaxed);
        }
        None
    }

    /// Calls `f` on up to `max` commands, oldest first, and returns how many it took.
    pub fn drain(&mut self, max: usize, mut f: impl FnMut(Words)) -> usize {
        let mut n = 0;
        while n < max {
            match self.pop() {
                Some(w) => {
                    f(w);
                    n += 1;
                }
                None => break,
            }
        }
        n
    }

    /// Commands pushed and not yet taken or skipped. Includes commands the producer has
    /// already overwritten, which the next `pop` will find and count as dropped.
    pub fn pending(&self) -> u64 {
        self.shared.head.load(Ordering::Acquire) - self.tail
    }

    pub fn stats(&self) -> RingStats {
        self.shared.stats()
    }

    /// Moves the command `pop` just returned from "received" to "dropped". The engine calls
    /// it when the words do not decode to a `Command`, so the counters still add up.
    /// A call with nothing to move (no command received yet) does nothing, so a stray call
    /// cannot wrap the counter.
    pub fn reject(&mut self) {
        if self.shared.received.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(1)).is_ok() {
            self.shared.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    /// A command payload that says what it is: the sequence number, a multiple of it, and its
    /// complement. A torn read (words from two different pushes) fails `intact`.
    pub(crate) fn mk(seq: u64) -> Words {
        [seq, seq.wrapping_mul(0x9E37_79B9_7F4A_7C15), !seq]
    }

    pub(crate) fn intact(w: Words) -> bool {
        w == mk(w[0])
    }

    fn drain_all(rx: &mut Receiver) -> Vec<u64> {
        let mut v = Vec::new();
        while let Some(w) = rx.pop() {
            assert!(intact(w), "torn command {w:x?}");
            v.push(w[0]);
        }
        v
    }

    #[test]
    fn r1_push_up_to_capacity_then_drain_returns_them_in_order() {
        let (mut tx, mut rx) = command_ring(8).unwrap();
        for s in 1..=8 {
            tx.push(mk(s));
        }
        assert_eq!(drain_all(&mut rx), (1..=8).collect::<Vec<_>>());
        assert_eq!(rx.stats(), RingStats { pushed: 8, dropped: 0, received: 8 });
    }

    #[test]
    fn r2_draining_an_empty_ring_returns_nothing_and_changes_no_counter() {
        let (tx, mut rx) = command_ring(4).unwrap();
        for _ in 0..3 {
            assert_eq!(rx.pop(), None);
            assert_eq!(rx.drain(10, |_| panic!("nothing to drain")), 0);
        }
        assert_eq!(rx.stats(), RingStats::default());
        assert_eq!(tx.stats(), RingStats::default());
        assert_eq!(rx.pending(), 0);
    }

    #[test]
    fn r3_overflow_keeps_the_newest_capacity_commands_and_counts_the_rest() {
        for k in [1u64, 2, 3, 7, 100] {
            let (mut tx, mut rx) = command_ring(4).unwrap();
            for s in 1..=(4 + k) {
                tx.push(mk(s));
            }
            let got = drain_all(&mut rx);
            assert_eq!(got, ((k + 1)..=(k + 4)).collect::<Vec<_>>(), "k = {k}");
            assert_eq!(rx.stats(), RingStats { pushed: 4 + k, dropped: k, received: 4 });
        }
    }

    #[test]
    fn r4_many_wraps_lose_nothing_when_the_consumer_keeps_up() {
        let (mut tx, mut rx) = command_ring(4).unwrap();
        let mut next = 1;
        for round in 0..100_000u64 {
            let n = 1 + round % 4;
            for _ in 0..n {
                tx.push(mk(next));
                next += 1;
            }
            let got = drain_all(&mut rx);
            assert_eq!(got.len() as u64, n);
            assert_eq!(*got.last().unwrap(), next - 1);
        }
        assert_eq!(rx.stats().dropped, 0);
        assert_eq!(rx.stats().received, next - 1);
    }

    #[test]
    fn r5_capacity_must_be_a_power_of_two_of_at_least_two() {
        for bad in [0usize, 1, 3, 5, 6, 7, 1000, 1023] {
            assert_eq!(command_ring(bad).err(), Some(RingError::BadCapacity), "capacity {bad}");
        }
        for good in [2usize, 4, 8, 1024, 4096] {
            assert!(command_ring(good).is_ok(), "capacity {good}");
        }
    }

    #[test]
    fn r6_drain_stops_at_max_and_leaves_the_rest() {
        let (mut tx, mut rx) = command_ring(16).unwrap();
        for s in 1..=10 {
            tx.push(mk(s));
        }
        let mut got = Vec::new();
        assert_eq!(rx.drain(3, |w| got.push(w[0])), 3);
        assert_eq!(got, vec![1, 2, 3]);
        assert_eq!(rx.pending(), 7);
        assert_eq!(rx.drain(100, |w| got.push(w[0])), 7);
        assert_eq!(got, (1..=10).collect::<Vec<_>>());
        assert_eq!(rx.drain(100, |_| panic!("empty")), 0);
    }

    #[test]
    fn r7_counters_add_up_after_every_operation_in_random_sequences() {
        use crate::rng::Rng;
        for seed in 0..1000u64 {
            let mut rng = Rng::new(seed);
            let cap = 1usize << (1 + rng.next_below(4)); // 2, 4, 8 or 16
            let (mut tx, mut rx) = command_ring(cap).unwrap();
            let mut next = 1u64;
            let mut last_received = 0u64;
            for step in 0..200 {
                match rng.next_below(3) {
                    0 => {
                        for _ in 0..(1 + rng.next_below(6)) {
                            tx.push(mk(next));
                            next += 1;
                        }
                    }
                    1 => {
                        if let Some(w) = rx.pop() {
                            assert!(intact(w), "seed {seed} step {step}: torn");
                            assert!(w[0] > last_received, "seed {seed} step {step}: {} after {last_received}", w[0]);
                            last_received = w[0];
                        }
                    }
                    _ => {
                        let max = 1 + rng.next_below(5) as usize;
                        rx.drain(max, |w| {
                            assert!(intact(w));
                            assert!(w[0] > last_received);
                            last_received = w[0];
                        });
                    }
                }
                let s = rx.stats();
                assert_eq!(s.pushed, next - 1, "seed {seed} step {step}");
                assert_eq!(s.pushed, s.received + s.dropped + rx.pending(), "seed {seed} step {step}: {s:?} pending {}", rx.pending());
            }
        }
    }

    #[test]
    fn r8_a_rejected_command_moves_from_received_to_dropped_and_the_counters_still_add_up() {
        let (mut tx, mut rx) = command_ring(4).unwrap();
        tx.push(mk(1));
        tx.push(mk(2));
        let w = rx.pop().unwrap();
        assert_eq!(w[0], 1);
        rx.reject();
        assert_eq!(rx.stats(), RingStats { pushed: 2, dropped: 1, received: 0 });
        assert_eq!(rx.pop().map(|w| w[0]), Some(2));
        let s = rx.stats();
        assert_eq!(s.pushed, s.received + s.dropped + rx.pending());
    }
}
