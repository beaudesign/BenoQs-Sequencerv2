//! The engine's link to the other threads: a command ring in, a snapshot buffer out
//! (SPEC-0001 O6, `specs/SPEC-0001/o6-test-plan.md`).
//!
//! `Engine::open_link` makes both and hands back the outer ends. From then on
//! `Engine::render` applies the commands the main thread pushed (at most
//! `MAX_COMMANDS_PER_RENDER` per call, oldest first, before any tick of that call) and, at
//! the end, publishes a snapshot for the render thread. The engine's own thread is the only
//! one that ever touches the engine, so nothing else needs to lock it.

use crate::ring::{command_ring, Receiver, RingStats, Sender};
use crate::snapshot::{read_words, write_words, SNAPSHOT_WORDS};
use crate::triple::{triple_buffer, Reader, Writer};
use crate::types::{Command, Snapshot};
use crate::wire::Words;

/// Capacity of the command ring, as in `docs/02` §2.
pub const COMMAND_RING_CAPACITY: usize = 1024;

/// The most commands `Engine::render` applies per call. What is left waits for the next call.
pub const MAX_COMMANDS_PER_RENDER: usize = 256;

fn zeroed_words() -> Box<[u32; SNAPSHOT_WORDS]> {
    vec![0u32; SNAPSHOT_WORDS].into_boxed_slice().try_into().expect("SNAPSHOT_WORDS words")
}

/// The main-thread end of the command ring. Owned by one thread at a time.
pub struct CommandSender {
    tx: Sender,
}

/// The render-thread end of the snapshot buffer. Owned by one thread at a time.
pub struct SnapshotReader {
    rx: Reader,
    words: Box<[u32; SNAPSHOT_WORDS]>,
    current: Box<Snapshot>,
}

impl CommandSender {
    /// Queues a command for the engine's next `render`. Never waits and never fails: if the
    /// ring is full the oldest command is overwritten and counted in `stats().dropped`.
    pub fn push(&mut self, cmd: Command) {
        self.tx.push(cmd.to_words());
    }

    /// Testing seam: queues raw words. The engine counts words that do not decode to a
    /// `Command` as dropped and applies nothing.
    pub fn push_words(&mut self, w: Words) {
        self.tx.push(w);
    }

    /// Commands pushed, applied (`received`) and dropped. `pushed == received + dropped +
    /// pending` whenever no push or drain is in flight; read from another thread while the
    /// engine is draining, the three can be a step apart.
    pub fn stats(&self) -> RingStats {
        self.tx.stats()
    }
}

impl SnapshotReader {
    /// Takes the newest snapshot the engine has published, if it is newer than the last one
    /// claimed. Returns whether it was. Never waits.
    pub fn claim(&mut self) -> bool {
        if !self.rx.claim() {
            return false;
        }
        self.rx.read(&mut self.words[..]);
        read_words(&self.words, &mut self.current)
    }

    /// The last snapshot claimed, or an all-zero one with generation 0 before the first. The
    /// reference stays valid, and unchanged, until the next `claim`.
    pub fn snapshot(&self) -> &Snapshot {
        &self.current
    }
}

/// The engine's ends of the ring and the buffer, plus the scratch it publishes from.
pub(crate) struct EngineLink {
    pub(crate) rx: Receiver,
    writer: Writer,
    words: Box<[u32; SNAPSHOT_WORDS]>,
    scratch: Box<Snapshot>,
    generation: u64,
}

impl EngineLink {
    pub(crate) fn open() -> (EngineLink, CommandSender, SnapshotReader) {
        let (tx, rx) = command_ring(COMMAND_RING_CAPACITY).expect("the ring capacity is a power of two");
        let (writer, reader) = triple_buffer(SNAPSHOT_WORDS);
        let link = EngineLink { rx, writer, words: zeroed_words(), scratch: Box::new(Snapshot::zeroed()), generation: 0 };
        let sender = CommandSender { tx };
        let reader = SnapshotReader { rx: reader, words: zeroed_words(), current: Box::new(Snapshot::zeroed()) };
        (link, sender, reader)
    }

    /// The snapshot to fill and publish next, with its generation already set.
    pub(crate) fn next_scratch(&mut self) -> &mut Snapshot {
        self.generation += 1;
        self.scratch.generation = self.generation;
        &mut self.scratch
    }

    pub(crate) fn publish(&mut self) {
        write_words(&self.scratch, &mut self.words);
        self.writer.publish(&self.words[..]);
    }
}
