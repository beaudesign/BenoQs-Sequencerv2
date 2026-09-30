#!/usr/bin/env python3
"""Mutation check of the integer step clock (SPEC-0001 O4 PR 4a; WENGE-0004; `crates/octocore/src/steps.rs`).

Each mutant is a one-line breakage of `steps.rs` or of the way the engine uses it, made in a scratch copy of the
workspace (the repo is never touched). A mutant is CAUGHT if any of these fail:

    S    cargo test -p octocore --lib steps::        (R1a to R1e and the edge cases)
    E    cargo test -p octocore --test o4_steps      (R1 through the engine)
    G    the guards and goldens: o4_guards, invariants, octorun golden
    U    every unit test of octocore (`--lib`), which includes the engine's own hyperstep tests

Run from the repo root:

    python3 handoffs/evidence/o4-guards/steps_mutants.py            # all
    python3 handoffs/evidence/o4-guards/steps_mutants.py SM3 SM5    # some

Exit status 0 only if every mutant is caught (or is listed as equivalent) and the control passes.
"""
import os, shutil, subprocess, sys, tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
TIMEOUT = int(os.environ.get("MUTANT_TIMEOUT", "900"))

FAMILIES = [
    ("S", ["cargo", "test", "-q", "-p", "octocore", "--lib", "steps::"]),
    ("E", ["cargo", "test", "-q", "-p", "octocore", "--test", "o4_steps"]),
    ("G", ["cargo", "test", "-q", "-p", "octocore", "--test", "o4_guards", "--test", "invariants"]),
    ("G", ["cargo", "test", "-q", "-p", "octorun", "--test", "golden"]),
    ("U", ["cargo", "test", "-q", "-p", "octocore", "--lib"]),
]

STEPS = "crates/octocore/src/steps.rs"
ENGINE = "crates/octocore/src/engine.rs"

# (id, file, what, old, new). EQUIVALENT lists the mutants that cannot change behaviour, so surviving is correct.
MUTANTS = [
    ("SM1", STEPS, "a step starts when the phase exceeds the step, not when it reaches it (`>=` becomes `>`)",
     "        if self.phase >= rate.den {", "        if self.phase > rate.den {"),
    ("SM2", STEPS, "the step is not subtracted when it starts (the phase grows for ever)",
     "            self.phase -= rate.den;\n            true", "            true"),
    ("SM3", STEPS, "the whole step is discarded when it starts (`phase = 0`), losing the remainder",
     "            self.phase -= rate.den;\n            true", "            self.phase = 0;\n            true"),
    ("SM4", STEPS, "the phase advances by one unit, not by `num`",
     "        self.phase += rate.num;", "        self.phase += 1;"),
    ("SM5", STEPS, "a change of multiplier rounds the carried phase up, not down",
     "self.phase = (self.phase as u64 * num as u64 / self.unit as u64) as u32;",
     "self.phase = ((self.phase as u64 * num as u64).div_ceil(self.unit as u64)) as u32;"),
    ("SM6", STEPS, "a change of multiplier drops the phase",
     "self.phase = (self.phase as u64 * num as u64 / self.unit as u64) as u32;", "self.phase = 0;"),
    ("SM7", STEPS, "a change of multiplier keeps the phase number and ignores the unit change",
     "self.phase = (self.phase as u64 * num as u64 / self.unit as u64) as u32;", "self.phase = self.phase;"),
    ("SM8", STEPS, "a step of exactly one tick is not treated as every-tick (`>=` becomes `>`)",
     "        self.num >= self.den\n", "        self.num > self.den\n"),
    ("SM9", STEPS, "a zero denominator is read as x2, not x1",
     "return StepRate { num: 1, den: default_step_ticks };", "return StepRate { num: 2, den: default_step_ticks };"),
    ("SM10", STEPS, "a zero numerator fires like x1",
     "        if rate.num == 0 {\n            return false;\n        }\n        if rate.num != self.unit {",
     "        if rate.num == 0 {\n            return true;\n        }\n        if rate.num != self.unit {"),
    ("SM11", STEPS, "the phase of a track at the step-of-a-tick-or-less state is reset each tick",
     "            return true;\n        }\n        self.phase += rate.num;",
     "            self.phase = 0;\n            return true;\n        }\n        self.phase += rate.num;"),
    ("SM12", ENGINE, "the engine gives a hyperstep-linked track a 96-tick step",
     "StepRate::fixed(192)", "StepRate::fixed(96)"),
    ("SM13", ENGINE, "the engine reads the numerator and denominator the wrong way round",
     "StepRate::from_multiplier(base_track.multiplier_num, base_track.multiplier_den, DEFAULT_STEP_TICKS)",
     "StepRate::from_multiplier(base_track.multiplier_den, base_track.multiplier_num, DEFAULT_STEP_TICKS)"),
    ("SM14", ENGINE, "the engine ignores the multiplier",
     "StepRate::from_multiplier(base_track.multiplier_num, base_track.multiplier_den, DEFAULT_STEP_TICKS)",
     "StepRate::from_multiplier(1, 1, DEFAULT_STEP_TICKS)"),
]

# SM8: at num == den a step is exactly one tick. The general path then adds `num`, reaches `den`, subtracts `den` and
# returns true: the phase is unchanged and a step starts on every tick, which is what the every-tick path does. The two
# paths agree for every input, so no test can tell the mutant from the original.
EQUIVALENT = {"SM8"}

SKIP_DIRS = {"target", ".git", "reference"}


def prepare(dst):
    def ignore(d, names):
        return [n for n in names if n in SKIP_DIRS and os.path.abspath(d) == ROOT]
    shutil.copytree(ROOT, dst, ignore=ignore)


def run(dst, env, cmd):
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
        if line.startswith("thread '") and "'" in line[8:]:
            names.add(line.split("'")[1].replace("steps::tests::", ""))
    return sorted(names)


def main():
    want = sys.argv[1:]
    work = tempfile.mkdtemp(prefix="o4-steps-mutants-")
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(work, "target"))
    status = 0
    try:
        dst = os.path.join(work, "control")
        prepare(dst)
        print("control (the unmodified workspace; every family must pass, or no mutant result means anything)")
        for fam, cmd in FAMILIES:
            res, out = run(dst, env, cmd)
            print(f"  {fam}  {' '.join(cmd[3:])}: {res}")
            if res != "pass":
                print(out[-3000:])
                return 2
        sys.stdout.flush()
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
            caught, names, broke = set(), {}, False
            for fam, cmd in FAMILIES:
                res, out = run(dst, env, cmd)
                if res == "compile":
                    broke = True
                    break
                if res in ("fail", "timeout"):
                    caught.add(fam)
                    names.setdefault(fam, []).extend(failing_names(out))
            if broke:
                print(f"{mid}  ERROR: the mutant did not compile ({what})")
                status = max(status, 2)
                continue
            verdict = "CAUGHT" if caught else ("SURVIVED (equivalent: see EQUIVALENT in the script)" if mid in EQUIVALENT else "SURVIVED")
            print(f"{mid}  {verdict} by {', '.join(sorted(caught)) or 'nothing'}: {what}")
            for fam in sorted(names):
                n = sorted(set(names[fam]))
                print(f"      {fam}: {', '.join(n[:4])}{f', and {len(n) - 4} more' if len(n) > 4 else ''}")
            if not caught and mid not in EQUIVALENT:
                status = max(status, 1)
            sys.stdout.flush()
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return status


if __name__ == "__main__":
    sys.exit(main())
