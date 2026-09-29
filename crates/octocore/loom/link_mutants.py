#!/usr/bin/env python3
"""Mutation check of the engine hooks and the command decoder (SPEC-0001 O6). The independent review of PR #11 found that
the slice-4 tests were shown failing only on a stub that returned `None`, which proves the API was missing, not that the
assertions can fail (and one of them, C6, could not). Each mutant here is a one-line breakage of the code that applies
commands and publishes snapshots; every one must make `cargo test` fail. Run from the repo root:

    python3 crates/octocore/loom/link_mutants.py            # all mutants
    python3 crates/octocore/loom/link_mutants.py LM3 LM4    # some

It copies octocore to a scratch directory and never touches the repo. Exit status 0 only if every mutant is caught.
"""
import os, shutil, subprocess, sys, tempfile

CRATE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
LOCK = os.path.join(CRATE, "..", "..", "Cargo.lock")
TIMEOUT = int(os.environ.get("MUTANT_TIMEOUT", "900"))

MUTANTS = [
    ("LM1", "src/engine.rs", "commands are applied after the render they arrive for, not before",
     "        self.apply_link_commands();\n        self.render_core(ctx, out);\n        self.publish_link_snapshot();",
     "        self.render_core(ctx, out);\n        self.apply_link_commands();\n        self.publish_link_snapshot();"),
    ("LM2", "src/engine.rs", "one command more than the per-render limit is applied",
     "for _ in 0..crate::link::MAX_COMMANDS_PER_RENDER {", "for _ in 0..crate::link::MAX_COMMANDS_PER_RENDER + 1 {"),
    ("LM3", "src/engine.rs", "an undecodable word is not counted as dropped",
     "None => link.rx.reject(),", "None => {}"),
    ("LM4", "src/engine.rs", "the snapshot is never published",
     "        link.publish();\n        self.link = Some(link);\n    }\n\n    pub fn diagnostics", "        self.link = Some(link);\n    }\n\n    pub fn diagnostics"),
    ("LM5", "src/engine.rs", "the snapshot is published before the render, so it is one render stale",
     "        self.apply_link_commands();\n        self.render_core(ctx, out);\n        self.publish_link_snapshot();",
     "        self.apply_link_commands();\n        self.publish_link_snapshot();\n        self.render_core(ctx, out);"),
    ("LM6", "src/engine.rs", "the snapshot reports the opposite transport state",
     "into.transport = TransportState { playing: self.running, tick: self.global_tick };",
     "into.transport = TransportState { playing: !self.running, tick: self.global_tick };"),
    ("LM7", "src/engine.rs", "the snapshot reports tick 0",
     "into.transport = TransportState { playing: self.running, tick: self.global_tick };",
     "into.transport = TransportState { playing: self.running, tick: 0 };"),
    ("LM8", "src/engine.rs", "the snapshot reports the wrong active page",
     "into.active = ActiveRefs { bank: self.grid.active_page.bank, page: self.grid.active_page.page };",
     "into.active = ActiveRefs { bank: self.grid.active_page.page, page: self.grid.active_page.bank };"),
    ("LM9", "src/attrs.rs", "a track index of exactly TRACK_COUNT is accepted",
     "(track as usize) < TRACK_COUNT", "(track as usize) <= TRACK_COUNT"),
    ("LM10", "src/attrs.rs", "a step index of exactly STEP_COUNT is accepted",
     "(step as usize) < STEP_COUNT", "(step as usize) <= STEP_COUNT"),
    ("LM11", "src/attrs.rs", "pitch is not clamped",
     "TrackAttr::Pitch => t.pitch = value.clamp(0, 127) as u8,", "TrackAttr::Pitch => t.pitch = value as u8,"),
    ("LM12", "src/engine.rs", "the command Stop does not stop the engine",
     "                self.set_running(false);\n            }\n            Command::Reset",
     "                self.set_running(self.running);\n            }\n            Command::Reset"),
]


def prepare(dst):
    shutil.copytree(CRATE, dst, ignore=shutil.ignore_patterns("target", "loom"))
    with open(os.path.join(dst, "Cargo.toml"), "a") as f:
        f.write("\n[workspace]\n")
    if os.path.exists(LOCK):
        shutil.copy(LOCK, os.path.join(dst, "Cargo.lock"))


def cargo_test(dst, env):
    return subprocess.run(["cargo", "test", "-q", "--lib", "--test", "link", "--test", "attrs"],
                          cwd=dst, env=env, capture_output=True, text=True, timeout=TIMEOUT)


def failing(out):
    return sorted({l.split("(")[0].split("'")[-1].strip() if False else l.strip().split(" ")[0]
                   for l in out.splitlines() if l.strip().startswith(("thread '",))})


def main():
    want = sys.argv[1:]
    work = tempfile.mkdtemp(prefix="octocore-linkmut-")
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(work, "target"))
    status = 0
    try:
        dst = os.path.join(work, "control")
        prepare(dst)
        r = cargo_test(dst, env)
        if r.returncode != 0:
            print("control  FAILED: the unmodified crate does not pass, so no mutant result means anything")
            print((r.stdout + r.stderr)[-3000:])
            return 2
        print("control  the unmodified crate passes")
        for mid, path, what, old, new in MUTANTS:
            if want and mid not in want:
                continue
            dst = os.path.join(work, mid)
            prepare(dst)
            p = os.path.join(dst, path)
            text = open(p).read()
            if text.count(old) != 1:
                print(f"{mid}  ERROR: the edit matched {text.count(old)} times in {path}, expected 1 ({what})")
                status = max(status, 2)
                continue
            open(p, "w").write(text.replace(old, new))
            try:
                r = cargo_test(dst, env)
            except subprocess.TimeoutExpired:
                print(f"{mid}  TIMEOUT ({what})")
                status = max(status, 1)
                continue
            out = r.stdout + r.stderr
            if r.returncode == 0:
                print(f"{mid}  SURVIVED ({what})")
                status = max(status, 1)
            elif "error[E" in out or "could not compile" in out:
                print(f"{mid}  ERROR: the mutant did not compile ({what})")
                status = max(status, 2)
            else:
                names = sorted({l.split("'")[1] for l in out.splitlines() if l.startswith("thread '") and "'" in l})
                print(f"{mid}  caught by {', '.join(names) or 'a test'} ({what})")
            sys.stdout.flush()
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return status


if __name__ == "__main__":
    sys.exit(main())
