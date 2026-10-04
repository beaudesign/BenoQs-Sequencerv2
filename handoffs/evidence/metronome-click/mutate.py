"""Mutation run for the metronome click (WENGE-0016, Forge hat). Each line breaks the click in one place; the web suite must notice.
Run from the repository root with apps/web/node_modules in place: WT=$PWD python3 handoffs/evidence/metronome-click/mutate.py"""
import subprocess, os
os.chdir(os.environ["WT"] + "/apps/web")
C = "src/click.ts"
W = "src/worklet.ts"
M = [
 ("K1 a beat on the end of a block is in it (the end is not strict)", C, "for (; k * QUARTER_TICKS < p1; k++)", "for (; k * QUARTER_TICKS <= p1; k++)"),
 ("K2 a beat on the start of a block is not in it", C, "while ((k - 1) * QUARTER_TICKS >= p0) k--;", "while ((k - 1) * QUARTER_TICKS >= p0) k--;\n  if (k * QUARTER_TICKS === p0) k++;"),
 ("K3 a span that does not move holds a beat", C, "if (!(p1 > p0)) return out;", "if (p1 < p0) return out;"),
 ("K4 the beat's frame is cut down and not rounded", C, "const at = Math.round(", "const at = Math.floor("),
 ("K5 the beat's frame can fall outside the block", C, "return Math.min(frames - 1, Math.max(0, at));", "return at;"),
 ("K6 a click is a quarter of the clock's beat", C, "export const QUARTER_TICKS = 24 * TICKS_PER_CLOCK;", "export const QUARTER_TICKS = 12 * TICKS_PER_CLOCK;"),
 ("K7 the first frame of a click is silence", C, "Math.sin(this.omega * (this.age + 1))", "Math.sin(this.omega * this.age)"),
 ("K8 the click never ends", C, "if (++this.age >= this.length) this.live = false;", "++this.age;"),
 ("K9 the click does not decay", C, "Math.exp(-this.age * this.decay)", "1"),
 ("K10 a new click adds to the one that is ringing", C, "    this.age = 0;\n    this.live = true;", "    this.live = true;"),
 ("K11 the block's old content is kept", C, "    out.fill(0);\n", ""),
 ("K12 the metronome clicks while it is off", C, "if (this.on && running) {", "if (running) {"),
 ("K13 the metronome clicks while stopped", C, "if (this.on && running) {", "if (this.on) {"),
 ("K14 switching off cuts the ringing click", C, "    this.voice.render(out, from, out.length);\n  }\n}", "    if (this.on) this.voice.render(out, from, out.length);\n  }\n}"),
 ("K15 the tail of a click is not written into the next block", C, "    this.voice.render(out, from, out.length);\n  }\n}", "  }\n}"),
 ("K16 the worklet reads the position after the render for both ends", W, "const from = this.engine.tickPosition();", "const from = 0;"),
 ("K17 the worklet asks whether the transport runs after the render, not before", W, "      const running = this.engine.running();\n      const from", "      const from"),
 ("K18 the metronome message is ignored", W, "this.metronome.on = m.on;", "this.metronome.on = false;"),
 ("K19 the metronome message makes the panel message and the engine is told nothing else", W, "          this.metronome.on = m.on;\n          return;", "          this.metronome.on = m.on;\n          break;"),
 ("K20 the worklet writes the click to the wrong end of the block", W, "if (out) this.metronome.render(out, from, this.engine.tickPosition(), running);", "if (out) this.metronome.render(out, this.engine.tickPosition(), from, running);"),
 ("K21 a choice made before Start is not sent at Start", "pages/app.ts", "        if (metronomeOn) started.setMetronome(true);\n", ""),
 ("K22 the button does not tell the worklet", "pages/app.ts", "    host?.setMetronome(metronomeOn);\n", ""),
 ("K23 the button does not say which way it is", "src/strip.ts", "showMetronome: (on) => metronome.setAttribute(\"aria-pressed\", on ? \"true\" : \"false\"),", "showMetronome: () => undefined,"),
]
caught = survived = missing = 0
for name, f, old, new in M:
    s = open(f).read()
    if old not in s:
        print(f"[{name}] PATTERN NOT FOUND"); missing += 1; continue
    open(f, "w").write(s.replace(old, new, 1))
    try:
        r = subprocess.run(["npm", "test", "--silent"], capture_output=True, text=True)
        out = r.stdout + r.stderr
        failed = [l for l in out.splitlines() if l.startswith("not ok")]
        t = subprocess.run(["npm", "run", "typecheck", "--silent"], capture_output=True, text=True)
        broke_types = t.returncode != 0
    finally:
        open(f, "w").write(s)
    # the page's own wiring (K21 to K23) is seen by the browser tests, not by the node suite
    if failed or broke_types:
        caught += 1; print(f"[{name}] CAUGHT ({len(failed)} failed{', types' if broke_types else ''}): {failed[0][:100] if failed else ''}")
    else:
        survived += 1; print(f"[{name}] SURVIVED by the node suite")
print(f"caught {caught} of {len(M)}, survived {survived}, pattern missing {missing}")
