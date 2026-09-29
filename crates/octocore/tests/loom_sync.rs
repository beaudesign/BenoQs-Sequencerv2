//! Loom models of the command ring and the snapshot buffer (SPEC-0001 O6, tests L1 to L4).
//! Loom runs the closure once for every interleaving of the atomic operations, so an
//! ordering mistake that a real run would hit once in a billion times is found here.
//!
//! Run with:
//!   RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=target/loom cargo test -p octocore --test loom_sync --release
//!
//! The models are small on purpose (a ring of 2, three or four pushes) and bounded to three
//! preemptions, the setting the loom documentation recommends. `crates/octocore/loom/mutants.py`
//! breaks the code five ways and checks that these tests notice. Not part of a required
//! gate yet: see the test plan, section 8.

#![cfg(loom)]

use loom::thread;
use octocore::ring::command_ring;
use octocore::triple::triple_buffer;
use octocore::wire::Words;

fn mk(seq: u64) -> Words {
    [seq, seq.wrapping_mul(0x9E37_79B9_7F4A_7C15), !seq]
}

fn intact(w: Words) -> bool {
    w == mk(w[0])
}

fn model(f: impl Fn() + Sync + Send + 'static) {
    let mut b = loom::model::Builder::new();
    b.preemption_bound = Some(3);
    b.check(f);
}

/// The sequence numbers received so far must strictly increase and every one must be intact.
fn check_received(got: &[Words]) {
    let mut last = 0;
    for &w in got {
        assert!(intact(w), "torn command {w:x?}");
        assert!(w[0] > last, "command {} after {}", w[0], last);
        last = w[0];
    }
}

#[test]
fn l1_ring_never_returns_a_torn_or_reordered_command_while_the_producer_laps_it() {
    model(|| {
        let (mut tx, mut rx) = command_ring(2).unwrap();
        let producer = thread::spawn(move || {
            for s in 1..=4u64 {
                tx.push(mk(s));
            }
            tx
        });
        let mut got = Vec::new();
        rx.drain(4, |w| got.push(w));
        let tx = producer.join().unwrap();
        while let Some(w) = rx.pop() {
            got.push(w);
        }
        check_received(&got);
        assert_eq!(got.last().map(|w| w[0]), Some(4), "the newest command survives");
        let s = rx.stats();
        assert_eq!(tx.stats().pushed, 4);
        assert_eq!(s.received as usize, got.len());
        assert_eq!(s.received + s.dropped, 4);
    });
}

#[test]
fn l2_ring_never_returns_a_command_twice_across_separate_drains() {
    model(|| {
        let (mut tx, mut rx) = command_ring(2).unwrap();
        let producer = thread::spawn(move || {
            for s in 1..=3u64 {
                tx.push(mk(s));
            }
        });
        let mut got = Vec::new();
        rx.drain(2, |w| got.push(w));
        rx.drain(2, |w| got.push(w));
        producer.join().unwrap();
        while let Some(w) = rx.pop() {
            got.push(w);
        }
        check_received(&got);
        let s = rx.stats();
        assert_eq!(s.received + s.dropped, 3);
    });
}

/// Two words that agree with each other, or the initial pair of zeros.
fn pair_ok(w: [u32; 2]) -> bool {
    w == [0, 0] || w[1] == !w[0]
}

#[test]
fn l3_snapshot_buffer_never_shows_a_torn_pair_and_ends_on_the_last_publish() {
    model(|| {
        let (mut w, mut r) = triple_buffer(2);
        let writer = thread::spawn(move || {
            for g in 1..=3u32 {
                w.publish(&[g, !g]);
            }
        });
        let mut last = 0u32;
        for _ in 0..3 {
            if r.claim() {
                let mut out = [0u32; 2];
                r.read(&mut out);
                assert!(pair_ok(out), "torn pair {out:x?}");
                assert!(out[0] > last, "generation {} after {last}", out[0]);
                last = out[0];
            }
        }
        writer.join().unwrap();
        r.claim();
        let mut out = [0u32; 2];
        r.read(&mut out);
        assert_eq!(out, [3, !3], "after the writer is done the reader sees the last publish");
    });
}

#[test]
fn l4_the_writer_never_writes_the_slot_the_reader_holds() {
    model(|| {
        let (mut w, mut r) = triple_buffer(2);
        let writer = thread::spawn(move || {
            for g in 1..=3u32 {
                w.publish(&[g, !g]);
            }
        });
        for _ in 0..2 {
            if r.claim() {
                // While the reader holds a slot, its contents must not change.
                let mut first = [0u32; 2];
                r.read(&mut first);
                thread::yield_now();
                let mut second = [0u32; 2];
                r.read(&mut second);
                assert_eq!(first, second, "the writer changed the slot the reader was holding");
                assert!(pair_ok(first));
            }
        }
        writer.join().unwrap();
    });
}
