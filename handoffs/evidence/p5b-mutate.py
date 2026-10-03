import subprocess, os, sys, re
os.chdir(os.environ["WT"])
env = dict(os.environ)
P="crates/octoface/src/panel.rs"; V="crates/octoface/src/view.rs"; L="crates/octoface/src/layout.rs"; R="crates/octoface/tests/panel_fixtures.rs"
M = [
 ("K1 the turn goes the wrong way (a clockwise detent lowers the value)", P, "now.saturating_add(i32::from(detents))", "now.saturating_sub(i32::from(detents))"),
 ("K2 the VEL knob writes the pitch offset", P, "Role::VelKnob => (StepAttr::VelocityOffset, s.velocity_offset),", "Role::VelKnob => (StepAttr::PitchOffset, s.velocity_offset),"),
 ("K3 the LEN knob starts from the velocity offset, not the length", P, "Role::LenKnob => (StepAttr::LengthTicks, s.length_ticks),", "Role::LenKnob => (StepAttr::LengthTicks, s.velocity_offset),"),
 ("K4 a knob turn is acted on in Page view too", P, "let (PanelMode::Step, Some(z)) = (self.mode, self.zoom) else { return };", "let Some(z) = self.zoom.or(Some(Zoom { track: 3, step: 4 })) else { return };"),
 ("K5 the turn edits step 0, not the selected step", P, "out.commands.push(Command::SetStep { track: z.track, step: z.step, attr, value: now.saturating_add(i32::from(detents)) });", "out.commands.push(Command::SetStep { track: z.track, step: 0, attr, value: now.saturating_add(i32::from(detents)) });"),
 ("K6 the turn edits track 0, not the zoomed track", P, "out.commands.push(Command::SetStep { track: z.track, step: z.step, attr, value: now.saturating_add(i32::from(detents)) });", "out.commands.push(Command::SetStep { track: 0, step: z.step, attr, value: now.saturating_add(i32::from(detents)) });"),
 ("K7 a turn sends its command twice", P, "out.commands.push(Command::SetStep { track: z.track, step: z.step, attr, value: now.saturating_add(i32::from(detents)) });", "out.commands.push(Command::SetStep { track: z.track, step: z.step, attr, value: now.saturating_add(i32::from(detents)) });\n        out.commands.push(Command::SetStep { track: z.track, step: z.step, attr, value: now.saturating_add(i32::from(detents)) });"),
 ("K8 a detent count is ignored (always one unit)", P, "now.saturating_add(i32::from(detents))", "now.saturating_add(i32::from(detents.signum()))"),
 ("K9 a zero turn still sends a command", P, "        if detents == 0 {\n            return;\n        }\n        let s = view.step", "        let s = view.step"),
 ("K10 the sum can overflow", P, "now.saturating_add(i32::from(detents))", "now + i32::from(detents)"),
 ("K11 the STA knob writes the LEN attribute", P, "Role::StaKnob => (StepAttr::StartOffset, s.start_offset),", "Role::StaKnob => (StepAttr::LengthTicks, s.start_offset),"),
 ("K12 the knob presses do something (a press of the PIT knob turns it)", P, "Role::VelKnob | Role::PitKnob | Role::LenKnob | Role::StaKnob => {}", "Role::VelKnob | Role::LenKnob | Role::StaKnob => {}\n            Role::PitKnob => self.knob_turn(Role::PitKnob, 1, view, out),"),
 ("V1 the view reads the length from the start offset", V, "length_ticks: i32::from(step.length_ticks),", "length_ticks: i32::from(step.start_offset),"),
 ("V2 the view reads the velocity from the pitch offset", V, "velocity_offset: i32::from(step.velocity_offset),", "velocity_offset: i32::from(step.pitch_offset),"),
 ("V3 the view reads the pitch from the velocity offset", V, "pitch_offset: i32::from(step.pitch_offset),", "pitch_offset: i32::from(step.velocity_offset),"),
 ("V4 the view reads the start offset from the length", V, "start_offset: i32::from(step.start_offset),", "start_offset: i32::from(step.length_ticks),"),
 ("L1 the VEL knob is the PIT knob's control id", L, "Role::VelKnob => \"edit.enc.vel\",", "Role::VelKnob => \"edit.enc.mch\","),
 ("R1 the runner reads vel as the pitch offset", R, "\"vel\" => Ok(StepAttr::VelocityOffset),", "\"vel\" => Ok(StepAttr::PitchOffset),"),
 ("R2 the runner's given ignores the value", R, "self.engine.grid.set_step_attr(t as u8, (s - 1) as u8, attr, value);", "self.engine.grid.set_step_attr(t as u8, (s - 1) as u8, attr, 0);"),
 ("R3 the runner's expect never compares a number", R, "                    if got != want {\n                        return Err(format!(\"engine step {t} {s} {what}: expected {want}, found {got}\"));\n                    }\n                    return Ok(());", "                    let _ = (got, want);\n                    return Ok(());"),
]
res=[]
for name,f,old,new in M:
    s=open(f).read()
    if old not in s:
        print(f"[{name}] PATTERN NOT FOUND"); res.append(None); continue
    open(f,"w").write(s.replace(old,new,1))
    try:
        r=subprocess.run(["/root/.cargo/bin/cargo","test","-p","octoface","--no-fail-fast"],capture_output=True,text=True,env=env)
        out=r.stdout+r.stderr
        compiled = "error[" not in out and "could not compile" not in out
        caught = (r.returncode!=0) and compiled
        tests=re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
        fixtures=sorted(set(re.findall(r"^(step_attributes/\S+?)\.panel:", out, re.M)))
        by = tests + [x.replace("step_attributes/","fixture ") for x in fixtures]
        print(f"[{name}] {'caught' if caught else ('DID NOT COMPILE' if not compiled else 'SURVIVED')}" + (f" by {len(by)}: " + "; ".join(x[:46] for x in by[:4]) + (" ..." if len(by)>4 else "") if by else ""))
        sys.stdout.flush()
        res.append(caught)
    finally:
        subprocess.run(["git","checkout","--",f])
print("caught", sum(1 for r in res if r), "of", len(res))
