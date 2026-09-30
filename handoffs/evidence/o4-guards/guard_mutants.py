#!/usr/bin/env python3
"""Mutation check of the O4 guard tests (SPEC-0001 O4 release plan, section 4; WENGE-0004 step 1).

A guard test is written on the parent commit, where it passes. That alone proves nothing: a test that
cannot fail also passes. Each mutant here is a one-line breakage of the engine's clock or emission, made in
a scratch copy of the workspace (the repo is never touched), and the script reports which guard families
notice it:

    G1  crates/octocore/tests/o4_guards.rs          (constant-tempo timing, every buffer size)
    G3  crates/octocore/tests/invariants.rs         (event times do not depend on the host's buffer)
    G2  crates/octorun/tests/golden.rs              (byte-identical event streams)
    G4  crates/octoffi  e4_* and g4_*               (no allocation in render)
    G5  crates/octocore/tests/conformance.rs        (the Octopus conformance fixtures)
    U   crates/octocore  lib unit tests             (not a guard; shown so redundancy is visible)

A mutant is CAUGHT if any guard family fails, SURVIVED if none does. Some survivors are expected and are
listed as such: a guard is only asked to protect what it claims to protect. Run from the repo root:

    python3 handoffs/evidence/o4-guards/guard_mutants.py            # all mutants
    python3 handoffs/evidence/o4-guards/guard_mutants.py OM3 OM7    # some

Exit status 0 only if every mutant marked `must_catch` is caught and every control passes.
"""
import os, shutil, subprocess, sys, tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
TIMEOUT = int(os.environ.get("MUTANT_TIMEOUT", "900"))

FAMILIES = [
    ("G1", ["cargo", "test", "-q", "-p", "octocore", "--test", "o4_guards"]),
    ("G3", ["cargo", "test", "-q", "-p", "octocore", "--test", "invariants"]),
    ("G2", ["cargo", "test", "-q", "-p", "octorun", "--test", "golden"]),
    ("G4", ["cargo", "test", "-q", "-p", "octoffi", "--", "e4_", "g4_"]),
    ("G5", ["cargo", "test", "-q", "-p", "octocore", "--test", "conformance"]),
    ("U", ["cargo", "test", "-q", "-p", "octocore", "--lib"]),
]

ENGINE = "crates/octocore/src/engine.rs"

# (id, file, what, old, new, must_catch, note)
MUTANTS = [
    ("OM1", ENGINE, "a tick's start is rounded to a whole sample before the next is added (drift, no carry)",
     "                            self.next_tick_due += spt;\n                        }\n                    }\n                    None => {",
     "                            self.next_tick_due = at.floor() + spt;\n                        }\n                    }\n                    None => {",
     True, ""),
    ("OM2", ENGINE, "samples per tick is rounded to a whole number",
     "Some(sample_rate as f64 * 60.0 / (bpm as f64 * TICKS_PER_QUARTER as f64))",
     "Some((sample_rate as f64 * 60.0 / (bpm as f64 * TICKS_PER_QUARTER as f64)).round())",
     True, ""),
    ("OM3", ENGINE, "the tick accumulator is an f32 (the precision the plan measured at 1 tick out in 30 minutes)",
     "                            self.next_tick_due += spt;\n                        }\n                    }\n                    None => {",
     "                            self.next_tick_due = (self.next_tick_due as f32 + spt as f32) as f64;\n                        }\n                    }\n                    None => {",
     True, "predicted to survive (drift over minutes, not the 10 s a guard can afford); G1 caught it, so G1 is more sensitive than expected. R1 and R2 of PR 4a still own the 30-minute case"),
    ("OM4", ENGINE, "every event is placed at the first sample of its buffer",
     "let at_sample = (top.due_sample - buffer_start).max(0.0) as u32;",
     "let at_sample = 0u32;",
     True, ""),
    ("OM5", ENGINE, "every event is one sample late",
     "let at_sample = (top.due_sample - buffer_start).max(0.0) as u32;",
     "let at_sample = (top.due_sample - buffer_start).max(0.0) as u32 + 1;",
     True, ""),
    ("OM6", ENGINE, "ticks are stepped only when due, not ahead of the buffer (the behaviour before O3)",
     "let horizon = chunk_end + MAX_EARLY_TICKS as f64 * spt;",
     "let horizon = chunk_end;",
     True, "G1 does not see this: at a constant tempo, stepping only when due gives the same times. Buffer-size dependence is G3's job, and G3 sees it"),
    ("OM7", ENGINE, "Play starts one sample after now",
     "            self.next_tick_due = self.sample_clock;\n",
     "            self.next_tick_due = self.sample_clock + 1.0;\n",
     False, "G1 does not see a one-sample (about 20 us) offset, by design: it is inside the 1.05-sample fence, which is for drift and spread. G3 and G2 do see it"),
    ("OM8", ENGINE, "Play starts one whole tick late",
     "            self.next_tick_due = self.sample_clock;\n",
     "            self.next_tick_due = self.sample_clock + 125.0;\n",
     True, ""),
    ("OM9", ENGINE, "the tempo used is 0.1% fast",
     "Some(sample_rate as f64 * 60.0 / (bpm as f64 * TICKS_PER_QUARTER as f64))",
     "Some(sample_rate as f64 * 60.0 / (bpm as f64 * 1.001 * TICKS_PER_QUARTER as f64))",
     True, ""),
    ("OM10", ENGINE, "one allocation on the render path",
     "    fn render_core(&mut self, ctx: &RenderContext, out: &mut EventBuffer) {\n",
     "    fn render_core(&mut self, ctx: &RenderContext, out: &mut EventBuffer) {\n        std::hint::black_box(Vec::<u8>::with_capacity(8));\n",
     True, ""),
    ("OM11", ENGINE, "the tempo the engine follows is the one from the previous render call",
     "        let spt = samples_per_tick(ctx.sample_rate, ctx.bpm);\n        if self.running && spt.is_none() {",
     "        let spt = samples_per_tick(ctx.sample_rate, if self.next_tick_due == 0.0 { ctx.bpm } else { ctx.bpm });\n        if self.running && spt.is_none() {",
     False, "placeholder replaced below"),
]

# OM11 is dropped: a stale tempo needs state the engine does not keep, and at a constant tempo it is invisible to any
# constant-tempo guard by definition. That is a ramp property, tested by R4 to R6 in PR 4b. It stays listed as a
# documented non-target rather than a fake mutant.
MUTANTS = [m for m in MUTANTS if m[0] != "OM11"]

SKIP_DIRS = {"target", ".git", "reference"}


def prepare(dst):
    def ignore(d, names):
        return [n for n in names if n in SKIP_DIRS and os.path.abspath(d) == ROOT]
    shutil.copytree(ROOT, dst, ignore=ignore)


def run_family(dst, env, cmd):
    try:
        r = subprocess.run(cmd, cwd=dst, env=env, capture_output=True, text=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return "timeout", ""
    out = r.stdout + r.stderr
    if "could not compile" in out or "error[E" in out:
        return "compile", out
    return ("pass" if r.returncode == 0 else "fail"), out


def failing_names(out):
    names = set()
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("test ") and line.endswith("FAILED"):
            names.add(line[5:].split(" ...")[0])
        if line.startswith("thread '") and "'" in line[8:]:
            names.add(line.split("'")[1])
    return sorted(names)


def main():
    want = sys.argv[1:]
    work = tempfile.mkdtemp(prefix="o4-guard-mutants-")
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(work, "target"))
    status = 0
    try:
        dst = os.path.join(work, "control")
        prepare(dst)
        print("control (the unmodified workspace; every family must pass, or no mutant result means anything)")
        for fam, cmd in FAMILIES:
            res, out = run_family(dst, env, cmd)
            print(f"  {fam:3} {res}")
            if res != "pass":
                print(out[-3000:])
                return 2
        sys.stdout.flush()
        for mid, path, what, old, new, must, note in MUTANTS:
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
            results, caught_by = {}, []
            for fam, cmd in FAMILIES:
                res, out = run_family(dst, env, cmd)
                results[fam] = res
                if res == "compile":
                    break
                if res in ("fail", "timeout") and fam != "U":
                    caught_by.append(fam)
                if res == "fail":
                    results[fam + "_names"] = failing_names(out)
            if "compile" in results.values():
                print(f"{mid}  ERROR: the mutant did not compile ({what})")
                status = max(status, 2)
                continue
            verdict = "CAUGHT" if caught_by else "SURVIVED"
            line = f"{mid}  {verdict} by {', '.join(caught_by) or 'no guard'}"
            if results.get("U") == "fail":
                line += " (unit tests too)"
            print(f"{line}: {what}")
            for fam, _ in FAMILIES:
                names = results.get(fam + "_names")
                if names and fam != "U":
                    shown = ", ".join(names[:4]) + (f", and {len(names) - 4} more" if len(names) > 4 else "")
                    print(f"      {fam}: {shown}")
            if note:
                print(f"      note: {note}")
            if verdict == "SURVIVED" and must:
                print("      NOT ACCEPTABLE: this mutant must be caught")
                status = max(status, 1)
            sys.stdout.flush()
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return status


if __name__ == "__main__":
    sys.exit(main())
