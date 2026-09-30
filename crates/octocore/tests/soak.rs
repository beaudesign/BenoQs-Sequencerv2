//! Two-thread soak tests for the command ring and the snapshot buffer (SPEC-0001 O6,
//! tests S1 to S3). Loom proves the protocol on small models; these run it for millions of
//! operations on real threads to find anything the models were too small to reach.
//!
//! The counts are results, never timings, so the tests do not flake on a slow runner. The
//! two snapshot soaks do 5 to 7 times fewer operations in a debug build than in a release
//! build, so that the default `cargo test` stays quick; a release run does the full counts
//! named in the test plan. The `#[ignore]`d versions run the same code for 100 million
//! operations and are run by hand (`cargo test -p octocore --release --test soak -- --ignored`).


#![cfg(not(loom))]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use octocore::ring::command_ring;
use octocore::snapshot::{read_words, write_words, SNAPSHOT_WORDS};
use octocore::triple::triple_buffer;
use octocore::types::Snapshot;
use octocore::wire::Words;

/// Publishes for S2 (ten million in a release build) and S3 (two hundred thousand).
const S2_PUBLISHES: u32 = if cfg!(debug_assertions) { 2_000_000 } else { 10_000_000 };
const S3_PUBLISHES: u64 = if cfg!(debug_assertions) { 30_000 } else { 200_000 };

fn mk(seq: u64) -> Words {
    [seq, seq.wrapping_mul(0x9E37_79B9_7F4A_7C15), !seq]
}

fn intact(w: Words) -> bool {
    w == mk(w[0])
}

/// The producer pushes `total` commands as fast as it can. The consumer takes them, burning
/// `consumer_delay` spin iterations after each so it can be made much slower than the
/// producer (which forces overflow). Returns `(received, dropped)`.
fn ring_soak(capacity: usize, total: u64, consumer_delay: u32) -> (u64, u64) {
    let (mut tx, mut rx) = command_ring(capacity).unwrap();
    let done = Arc::new(AtomicBool::new(false));
    let producer_done = done.clone();
    let producer = thread::spawn(move || {
        for s in 1..=total {
            tx.push(mk(s));
        }
        producer_done.store(true, Ordering::Release);
        tx
    });

    let mut last = 0u64;
    let mut received = 0u64;
    let take = |rx: &mut octocore::ring::Receiver, last: &mut u64, received: &mut u64| -> bool {
        match rx.pop() {
            Some(w) => {
                assert!(intact(w), "torn command {w:x?} after {last}");
                assert!(w[0] > *last, "command {} arrived after {}", w[0], *last);
                *last = w[0];
                *received += 1;
                true
            }
            None => false,
        }
    };
    loop {
        // Read the flag first: if it is set, every push has happened, and one more pass
        // that finds nothing means the ring is empty for good.
        let finished = done.load(Ordering::Acquire);
        let got = take(&mut rx, &mut last, &mut received);
        for _ in 0..consumer_delay {
            std::hint::spin_loop();
        }
        if !got && finished {
            break;
        }
    }
    let tx = producer.join().unwrap();

    assert_eq!(tx.stats().pushed, total);
    let s = rx.stats();
    assert_eq!(s.received, received);
    assert_eq!(s.received + s.dropped, total, "every command is either received or counted as dropped: {s:?}");
    assert_eq!(rx.pending(), 0);
    assert_eq!(last, total, "the newest command is never the one that is lost");
    (s.received, s.dropped)
}

#[test]
fn s1_ring_keeps_order_and_accounts_for_every_command_when_the_consumer_keeps_up() {
    let (received, _dropped) = ring_soak(1024, 2_000_000, 0);
    assert!(received > 0);
}

#[test]
fn s1_ring_overflows_cleanly_when_the_consumer_is_far_slower() {
    let (received, dropped) = ring_soak(64, 2_000_000, 4_000);
    assert!(dropped > 0, "the consumer was slow enough that the producer had to lap it");
    assert!(received > 0);
}

#[test]
#[ignore = "deep soak, run by hand"]
fn s1_deep_ring_soak_100_million() {
    ring_soak(1024, 100_000_000, 0);
    ring_soak(64, 100_000_000, 200);
}

/// A 16-word slot whose words all follow from the generation. A reader that ever sees words
/// from two different generations sees words that disagree.
fn small(g: u32) -> [u32; 16] {
    let mut w = [g; 16];
    w[15] = !g;
    w
}

fn small_ok(w: &[u32; 16]) -> bool {
    let g = w[0];
    *w == small(g) || *w == [0; 16]
}

fn triple_soak_small(total: u32) {
    let (mut w, mut r) = triple_buffer(16);
    let done = Arc::new(AtomicBool::new(false));
    let writer_done = done.clone();
    let writer = thread::spawn(move || {
        for g in 1..=total {
            w.publish(&small(g));
        }
        writer_done.store(true, Ordering::Release);
        w
    });
    let mut last = 0u32;
    let mut claims = 0u64;
    loop {
        let finished = done.load(Ordering::Acquire);
        if r.claim() {
            claims += 1;
            let mut out = [0u32; 16];
            r.read(&mut out);
            assert!(small_ok(&out), "torn snapshot {out:?} after generation {last}");
            assert!(out[0] > last, "generation {} after {last}", out[0]);
            last = out[0];
        } else if finished {
            break;
        }
    }
    let w = writer.join().unwrap();
    assert_eq!(w.published(), total as u64);
    assert_eq!(last, total, "the last publish is always the last thing the reader sees");
    assert!(claims > 0);
}

#[test]
fn s2_snapshot_buffer_never_shows_a_torn_slot_across_millions_of_publishes() {
    triple_soak_small(S2_PUBLISHES);
}

fn fill(g: u64) -> Snapshot {
    let mut s = Snapshot::zeroed();
    s.generation = g;
    for (i, led) in s.leds.iter_mut().enumerate() {
        let v = g.wrapping_add(i as u64) as f32;
        led.color.r = v;
        led.color.g = v + 1.0;
        led.color.b = v + 2.0;
        led.target = v + 3.0;
    }
    for (i, e) in s.encoders.iter_mut().enumerate() {
        e.angle_radians = (g + i as u64) as f32;
        e.detent_index = g as i32 + i as i32;
    }
    for (i, p) in s.playheads.iter_mut().enumerate() {
        p.step_index = (g as u8).wrapping_add(i as u8);
        p.track_index = i as u8;
    }
    s.transport.playing = g % 2 == 0;
    s.transport.tick = g * 3;
    s.active.bank = g as u8;
    s.active.page = !(g as u8);
    s
}

fn fill_ok(s: &Snapshot) -> bool {
    let e = fill(s.generation);
    let (mut a, mut b) = ([0u32; SNAPSHOT_WORDS], [0u32; SNAPSHOT_WORDS]);
    write_words(s, &mut a);
    write_words(&e, &mut b);
    a == b
}

fn triple_soak_snapshot(total: u64) {
    let (mut w, mut r) = triple_buffer(SNAPSHOT_WORDS);
    let done = Arc::new(AtomicBool::new(false));
    let writer_done = done.clone();
    let writer = thread::spawn(move || {
        let mut words = [0u32; SNAPSHOT_WORDS];
        for g in 1..=total {
            write_words(&fill(g), &mut words);
            w.publish(&words);
        }
        writer_done.store(true, Ordering::Release);
    });
    let mut last = 0u64;
    let mut words = [0u32; SNAPSHOT_WORDS];
    let mut snap = Snapshot::zeroed();
    let mut claims = 0u64;
    loop {
        let finished = done.load(Ordering::Acquire);
        if r.claim() {
            claims += 1;
            r.read(&mut words);
            assert!(read_words(&words, &mut snap));
            assert!(fill_ok(&snap), "torn snapshot with generation {}", snap.generation);
            assert!(snap.generation > last);
            last = snap.generation;
        } else if finished {
            break;
        }
    }
    writer.join().unwrap();
    assert_eq!(last, total);
    assert!(claims > 0);
}

#[test]
fn s3_real_snapshot_is_never_torn_across_tens_of_thousands_of_publishes() {
    triple_soak_snapshot(S3_PUBLISHES);
}

#[test]
#[ignore = "deep soak, run by hand"]
fn s2_deep_snapshot_soak_100_million() {
    triple_soak_small(100_000_000);
    triple_soak_snapshot(2_000_000);
}
