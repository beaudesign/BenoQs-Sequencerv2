"""The three mutants of mutate.py that live in the page (K21 to K23) are seen by the Chromium tests, not the Node suite. Each is built and
run against the C8 tests. Run from the repository root: WT=$PWD python3 handoffs/evidence/metronome-click/mutate-browser.py"""
import os, subprocess
os.chdir(os.environ["WT"] + "/apps/web")
M = [
 ("K21 a choice made before Start is not sent at Start", "pages/app.ts", "        if (metronomeOn) started.setMetronome(true);\n", ""),
 ("K22 the button does not tell the worklet", "pages/app.ts", "    host?.setMetronome(metronomeOn);\n", ""),
 ("K23 the button does not say which way it is", "src/strip.ts", "showMetronome: (on) => metronome.setAttribute(\"aria-pressed\", on ? \"true\" : \"false\"),", "showMetronome: () => undefined,"),
 ("K24 the click is not written to the output", "src/worklet.ts", "if (out) this.metronome.render(out, from, this.engine.tickPosition(), running);", "void out;"),
]
for name, f, old, new in M:
    s = open(f).read()
    assert old in s, name
    open(f, "w").write(s.replace(old, new, 1))
    try:
        subprocess.run(["npm", "run", "build", "--silent"], capture_output=True, text=True, check=True)
        r = subprocess.run(["node", "--test", "--test-concurrency=1", "--test-name-pattern=C8", "test/browser/app.browser.test.ts", "test/browser/host.browser.test.ts"], capture_output=True, text=True, timeout=240)
        failed = [l for l in (r.stdout + r.stderr).splitlines() if l.startswith("not ok")]
        print(f"[{name}] {'CAUGHT' if failed else 'SURVIVED'} ({len(failed)} failed): {failed[0][:110] if failed else ''}")
    finally:
        open(f, "w").write(s)
subprocess.run(["npm", "run", "build", "--silent"], capture_output=True, text=True, check=True)
