#!/usr/bin/env python3
"""Tests the loom tests: breaks the ring and the snapshot buffer one way at a time and checks
that tests/loom_sync.rs fails each time (SPEC-0001 O6, test plan section 8).

    crates/octocore/loom/mutants.py            # the control run, then every mutant
    crates/octocore/loom/mutants.py M2 M5      # only these (no control run)

Each mutant is a copy of the crate with one exact-text edit. An edit that does not match
exactly once is an error, so the script cannot pass because a mutant silently did not apply.
A mutant that is not caught, or that the loom run cannot finish within MUTANT_TIMEOUT seconds
(default 900), is a hole in the loom models: the models get stronger, the mutant stays.
Exit status: 0 all caught, 1 a mutant survived or timed out, 2 a mutant did not compile or
apply, or the control failed. Needs network access for the first build (loom).
"""
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)
LOCK = os.path.join(CRATE, "..", "..", "Cargo.lock")
TIMEOUT = int(os.environ.get("MUTANT_TIMEOUT", "900"))

# (id, file, what the mutant does, exact text that must appear once, replacement)
MUTANTS = [
    ("M1", "src/ring.rs", "producer never marks a slot as being written",
     "        slot.stamp.store(2 * i + 1, Ordering::Relaxed);\n",
     "        // mutant M1: no odd stamp\n"),
    ("M2", "src/ring.rs", "consumer skips the second stamp check",
     "            if before == want && after == want {\n",
     "            if before == want && (after == want || true) {\n"),
    ("M3", "src/ring.rs", "producer publishes head before the payload",
     "        let slot = &self.shared.slots[(i & self.shared.mask) as usize];\n",
     "        self.shared.head.store(i + 1, Ordering::Release);\n"
     "        let slot = &self.shared.slots[(i & self.shared.mask) as usize];\n"),
    ("M4", "src/ring.rs", "payload words are read Relaxed instead of Acquire",
     "                slot.words[0].load(Ordering::Acquire),\n"
     "                slot.words[1].load(Ordering::Acquire),\n"
     "                slot.words[2].load(Ordering::Acquire),\n",
     "                slot.words[0].load(Ordering::Relaxed),\n"
     "                slot.words[1].load(Ordering::Relaxed),\n"
     "                slot.words[2].load(Ordering::Relaxed),\n"),
    ("M5", "src/triple.rs", "writer stores into the middle instead of swapping, so it never learns a free slot",
     "        let old = self.shared.middle.swap(self.back | DIRTY, Ordering::AcqRel);\n"
     "        self.back = old & INDEX;\n",
     "        self.shared.middle.store(self.back | DIRTY, Ordering::Release);\n"),
    ("M6", "src/triple.rs", "reader claims without checking the dirty bit",
     "        if self.shared.middle.load(Ordering::Relaxed) & DIRTY == 0 {\n            return false;\n        }\n",
     "        // mutant M6: no dirty check\n"),
    ("M7", "src/triple.rs", "the writer's swap is Relaxed, so the words are not published",
     "self.shared.middle.swap(self.back | DIRTY, Ordering::AcqRel)",
     "self.shared.middle.swap(self.back | DIRTY, Ordering::Relaxed)"),
]


def prepare(dst):
    shutil.copytree(CRATE, dst, ignore=shutil.ignore_patterns("target", "loom"))
    with open(os.path.join(dst, "Cargo.toml"), "a") as f:
        f.write("\n[workspace]\n")
    if os.path.exists(LOCK):
        shutil.copy(LOCK, os.path.join(dst, "Cargo.lock"))


def run_loom(dst, env):
    return subprocess.run(
        ["cargo", "test", "-q", "--release", "--test", "loom_sync"],
        cwd=dst, env=env, capture_output=True, text=True, timeout=TIMEOUT,
    )


def failing_tests(out):
    names, in_list = [], False
    for line in out.splitlines():
        if line.strip() == "failures:":
            in_list = True
        elif in_list and line.startswith("    ") and line.strip() and " " not in line.strip():
            names.append(line.strip())
        elif in_list and not line.strip() and names:
            break
    return names


def main():
    want = sys.argv[1:]
    work = tempfile.mkdtemp(prefix="octocore-mutants-")
    env = dict(os.environ, RUSTFLAGS="--cfg loom", CARGO_TARGET_DIR=os.path.join(work, "target"))
    status = 0
    try:
        if not want:
            dst = os.path.join(work, "control")
            prepare(dst)
            r = run_loom(dst, env)
            if r.returncode != 0:
                print("control  FAILED: the unmodified crate does not pass, so no mutant result means anything")
                print((r.stdout + r.stderr)[-3000:])
                return 2
            print("control  the unmodified crate passes the loom tests")
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
                r = run_loom(dst, env)
            except subprocess.TimeoutExpired:
                print(f"{mid}  TIMEOUT after {TIMEOUT}s, not caught ({what})")
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
                print(f"{mid}  caught by {', '.join(failing_tests(out)) or 'a loom test'} ({what})")
            sys.stdout.flush()
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return status


if __name__ == "__main__":
    sys.exit(main())
