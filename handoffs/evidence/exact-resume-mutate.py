"""Mutation run for the exact-resume change (WENGE-0016, Metronome hat). Each line breaks the new engine code in one place; the suite must notice.
Run from the repository root: WT=$PWD CARGO_TARGET_DIR=... python3 handoffs/evidence/exact-resume-mutate.py"""
import subprocess, os, sys, re
os.chdir(os.environ["WT"])
env = dict(os.environ)
E = "crates/octocore/src/engine.rs"
M = [
 ("R1 Stop drops every held event, as it did before", E, "                _ => true,\n            };\n            if keep {", "                _ => false,\n            };\n            if keep {"),
 ("R2 Stop keeps every Note Off, including those of notes the flush has ended", E, "order.2 > 0 && note_ons[..n].binary_search(&(order.2 - 1)).is_ok(),", "true,"),
 ("R3 Stop drops every Note Off, including those of notes still to start", E, "order.2 > 0 && note_ons[..n].binary_search(&(order.2 - 1)).is_ok(),", "false,"),
 ("R4 a Note Off is paired with the event made after it, not before", E, "binary_search(&(order.2 - 1))", "binary_search(&(order.2 + 1))"),
 ("R5 the queue is not rebuilt as a heap after Stop took entries out", E, "        for i in (0..self.queue_len / 2).rev() {\n            self.heap_sift_down(i);\n        }\n        self.flush_pending = true;", "        self.flush_pending = true;"),
 ("R6 Play does not move the held events by the time stopped", E, "                held.due_sample += gap;", "                held.due_sample += 0.0 * gap;"),
 ("R7 Play moves the held events but not their sort key", E, "                held.order.0 = held.due_sample.floor() as i64;", "                held.order.0 = held.order.0;"),
 ("R8 stepping goes on from the Play, as it did before, not from the tick after the last one stepped", E, "        self.next_tick_due = (self.next_tick_due + gap).max(self.sample_clock);\n    }", "        self.next_tick_due = self.sample_clock;\n    }"),
 ("R9 the tick ring is not moved with the held ticks", E, "                self.tick_due[(k % TICK_RING as u64) as usize] += gap;", "                self.tick_due[(k % TICK_RING as u64) as usize] += 0.0 * gap;"),
 ("R10 the last tick that sounded is not moved with the held ones", E, "for k in r.tick.saturating_sub(1).max(", "for k in r.tick.max("),
 ("R11 held events are emitted while stopped", E, "            if self.resume.is_none() {\n                self.drain_due(", "            if true {\n                self.drain_due("),
 ("R12 a tick due on the very sample of the Stop counts as sounded", E, "self.tick_due[((k - 1) % TICK_RING as u64) as usize] >= self.sample_clock", "self.tick_due[((k - 1) % TICK_RING as u64) as usize] > self.sample_clock"),
 ("R13 the pulses of the held ticks are not sent after the Play", E, "                    self.rt_pulses_ahead(r.tick);\n", ""),
 ("R14 the pulse of the first held tick is not sent", E, "        for k in from_tick.max(self.global_tick.saturating_sub(TICK_RING as u64))..self.global_tick {\n            if k % TICKS_PER_CLOCK", "        for k in (from_tick + 1).max(self.global_tick.saturating_sub(TICK_RING as u64))..self.global_tick {\n            if k % TICKS_PER_CLOCK"),
 ("R15 pulses of held ticks are sent with the clock off", E, "    fn rt_pulses_ahead(&mut self, from_tick: u64) {\n        if !self.clock_master {\n            return;\n        }", "    fn rt_pulses_ahead(&mut self, from_tick: u64) {"),
 ("R16 tick_position while stopped is the engine's own count", E, "            return self.resume.map_or(self.global_tick as f64, |r| r.position);", "            return self.global_tick as f64;"),
 ("R17 the position is read after the engine has stepped ahead, not where the audio is", E, "            let position = self.tick_position();", "            let position = self.global_tick as f64;"),
 ("R18 Reset keeps what Stop was holding", E, "        self.resume = None;\n        self.deferred_actions", "        self.deferred_actions"),
 ("R19 Start or Continue is chosen from the engine's count, not the tick it carries on from", E, "                    self.rt_transport_started(r.tick);", "                    self.rt_transport_started(self.global_tick);"),
 ("R20 Play after a Stop starts from now with the held events dropped, as before", E, "                Some(r) => {\n                    self.carry_on(r);", "                Some(r) => {\n                    self.queue = [None; QUEUE_CAP];\n                    self.queue_len = 0;\n                    self.carry_on(r);"),
]
res=[]
only = os.environ.get("ONLY")
for name,f,old,new in M:
    if only and name.split()[0] not in only.split(","):
        continue
    s=open(f).read()
    if old not in s:
        print(f"[{name}] PATTERN NOT FOUND"); res.append(None); continue
    open(f,"w").write(s.replace(old,new,1))
    try:
        r=subprocess.run(["/root/.cargo/bin/cargo","test","-p","octocore","-p","octorun","--no-fail-fast"],capture_output=True,text=True,env=env)
        out=r.stdout+r.stderr
        compiled = "error[" not in out and "could not compile" not in out
        caught = (r.returncode!=0) and compiled
        tests=re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
        print(f"[{name}] {'caught' if caught else ('DID NOT COMPILE' if not compiled else 'SURVIVED')}" + (f" by {len(tests)}: " + "; ".join(x[:60] for x in tests[:4]) + (" ..." if len(tests)>4 else "") if tests else ""))
        sys.stdout.flush()
        res.append(caught)
    finally:
        subprocess.run(["git","checkout","--",f])
print("caught", sum(1 for r in res if r), "of", len(res))
