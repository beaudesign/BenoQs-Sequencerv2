//! Runs `tests/conformance/panel/**/*.panel` against the panel controller and the real engine.
//!
//! A panel fixture is a timed list of button and encoder events in, and the LED frame, the engine
//! commands and the engine's own state out (ADR-0007 decision 4). It sits beside the engine's
//! fixtures (`crates/octocore/src/fixture.rs`) and shares none of their syntax.
//!
//! ```text
//! # panel fixture: a matrix key turns an off step on     header: title, manual pages, workflow
//! given step 3 5 on|off|skip|chord|event|hyper           engine state before inputs; step is 1 to 16
//! at 0 ms press|release|click <control>                  a click is a press, then a release 40 ms on
//! at 0 ms turn <control> <detents>
//! expect led <control> = off | red|green|orange steady|flash|shine
//! expect mode page|step                                  the controller's mode
//! expect engine mode page|step                           the engine's mode, after the commands
//! expect engine step <track> <step> active|skip = true|false
//! expect command <text>                                  one of the last input's commands
//! expect commands <n>                                    how many commands the last input gave
//! expect intent <text> | expect intents <n>              the same for intents
//! ```
//!
//! A control is an `id` from `contracts/controls.json`, or `step(track,step)` for the matrix key
//! of that track and step (steps are numbered 1 to 16 as the manual numbers them). Expectations
//! read the state at the moment they appear, after every earlier line. "The last input" is the
//! most recent `press`, `release`, `click` or `turn` line; a click counts both its events.
//! Commands are applied to a real `octocore::Engine` as they come out, and the controller is
//! shown a fresh `PageView` of that engine before every input and every `expect led`.

mod support;

use octocore::domain::{Mode, StepEvent, StepEventKind};
use octocore::types::{Command, ControlId};
use octocore::Engine;
use octoface::{Colour, Input, Intent, Led, PageView, Panel, PanelMode, Phase};
use std::collections::BTreeMap;
use support::{inventory, panel_files, Inventory};

const CLICK_MS: u64 = 40;

struct World<'a> {
    numbers: &'a BTreeMap<String, u32>,
    panel: Panel,
    engine: Engine,
    now: u64,
    last_commands: Vec<Command>,
    last_intents: Vec<Intent>,
}

fn show_command(c: &Command) -> String {
    match c {
        Command::SetStep { track, step, attr, value } => format!("SetStep track={track} step={step} attr={attr:?} value={value}"),
        Command::SetMode { mode } => format!("SetMode mode={mode:?}"),
        Command::Stop => "Stop".to_string(),
        Command::Play => "Play".to_string(),
        other => format!("{other:?}"),
    }
}

fn show_intent(i: &Intent) -> String {
    match i {
        Intent::Audition { track, step } => format!("Audition track={track} step={step}"),
        Intent::SnapshotTake => "SnapshotTake".to_string(),
        Intent::SnapshotKeep => "SnapshotKeep".to_string(),
        Intent::SnapshotRecall => "SnapshotRecall".to_string(),
    }
}

fn parse_led(words: &[&str]) -> Result<Led, String> {
    let colour = |w: &str| match w {
        "red" => Ok(Colour::Red),
        "green" => Ok(Colour::Green),
        "orange" => Ok(Colour::Orange),
        other => Err(format!("unknown colour `{other}`")),
    };
    let phase = |w: &str| match w {
        "steady" => Ok(Phase::Steady),
        "flash" => Ok(Phase::Flash),
        "shine" => Ok(Phase::Shine),
        other => Err(format!("unknown phase `{other}`")),
    };
    match words {
        ["off"] => Ok(Led::OFF),
        [c, p] => Ok(Led { colour: colour(c)?, phase: phase(p)? }),
        _ => Err("an LED is `off` or a colour and a phase".to_string()),
    }
}

fn show_led(l: Led) -> String {
    if l.colour == Colour::Off {
        return "off".to_string();
    }
    format!("{} {}", format!("{:?}", l.colour).to_lowercase(), format!("{:?}", l.phase).to_lowercase())
}

impl<'a> World<'a> {
    fn new(inv: &'a Inventory) -> World<'a> {
        World {
            numbers: &inv.numbers,
            panel: Panel::new(inv.layout.clone()),
            engine: Engine::new(0),
            now: 0,
            last_commands: Vec::new(),
            last_intents: Vec::new(),
        }
    }

    fn view(&self) -> PageView {
        PageView::from_page(self.engine.grid.active_page())
    }

    /// `step(3,5)` or an `id` from the inventory.
    fn control(&self, token: &str) -> Result<ControlId, String> {
        let id = match token.strip_prefix("step(").and_then(|t| t.strip_suffix(')')) {
            Some(args) => {
                let (t, s) = args.split_once(',').ok_or("step(track,step) needs two numbers")?;
                let (t, s): (usize, usize) = (t.trim().parse().map_err(|_| "bad track")?, s.trim().parse().map_err(|_| "bad step")?);
                format!("matrix.r{t}.c{s}")
            }
            None => token.to_string(),
        };
        self.numbers.get(&id).map(|n| ControlId(*n)).ok_or_else(|| format!("`{id}` is not a control in contracts/controls.json"))
    }

    fn send(&mut self, at: u64, input: Input) {
        let view = self.view();
        let out = self.panel.input(at, input, &view);
        for c in &out.commands {
            self.engine.handle_command(*c);
        }
        self.last_commands.extend(out.commands);
        self.last_intents.extend(out.intents);
    }

    fn given(&mut self, words: &[&str]) -> Result<(), String> {
        let [t, s, flag] = words else { return Err("usage: given step <track> <step> <flag>".to_string()) };
        let (t, s): (usize, usize) = (t.parse().map_err(|_| "bad track")?, s.parse().map_err(|_| "bad step")?);
        if t >= 10 || !(1..=16).contains(&s) {
            return Err("track is 0 to 9 and step is 1 to 16".to_string());
        }
        let step = &mut self.engine.grid.active_page_mut().tracks[t].steps[s - 1];
        match *flag {
            "on" => step.active = true,
            "off" => step.active = false,
            "skip" => step.skip = true,
            "chord" => {
                step.chord.count = 1;
                step.chord.offsets[0] = 4;
            }
            "event" => step.event = Some(StepEvent { target_track: t as u8, kind: StepEventKind::SetPos(1) }),
            "hyper" => step.hyperstep = true,
            other => return Err(format!("unknown step flag `{other}`")),
        }
        Ok(())
    }

    fn at(&mut self, words: &[&str]) -> Result<(), String> {
        let [ms, "ms", verb, rest @ ..] = words else { return Err("usage: at <ms> ms press|release|click|turn <control>".to_string()) };
        let ms: u64 = ms.parse().map_err(|_| "bad time")?;
        if ms < self.now {
            return Err(format!("time goes backwards: {ms} ms after {} ms", self.now));
        }
        self.last_commands.clear();
        self.last_intents.clear();
        match (*verb, rest) {
            ("press", [c]) => {
                let c = self.control(c)?;
                self.send(ms, Input::Down(c));
                self.now = ms;
            }
            ("release", [c]) => {
                let c = self.control(c)?;
                self.send(ms, Input::Up(c));
                self.now = ms;
            }
            ("click", [c]) => {
                let c = self.control(c)?;
                self.send(ms, Input::Down(c));
                self.send(ms + CLICK_MS, Input::Up(c));
                self.now = ms + CLICK_MS;
            }
            ("turn", [c, d]) => {
                let c = self.control(c)?;
                let detents: i16 = d.parse().map_err(|_| "bad detent count")?;
                self.send(ms, Input::Turn { control: c, detents });
                self.now = ms;
            }
            _ => return Err("usage: at <ms> ms press|release|click|turn <control>".to_string()),
        }
        Ok(())
    }

    fn expect(&mut self, words: &[&str]) -> Result<(), String> {
        match words {
            ["led", control, "=", led @ ..] => {
                let id = self.control(control)?;
                let want = parse_led(led)?;
                let got = self.panel.leds(&self.view()).get(id);
                if got != want {
                    return Err(format!("{control}: expected {}, found {}", show_led(want), show_led(got)));
                }
            }
            ["mode", m] => {
                let want = match *m {
                    "page" => PanelMode::Page,
                    "step" => PanelMode::Step,
                    other => return Err(format!("unknown mode `{other}`")),
                };
                if self.panel.mode() != want {
                    return Err(format!("controller mode: expected {want:?}, found {:?}", self.panel.mode()));
                }
            }
            ["engine", "mode", m] => {
                let want = match *m {
                    "page" => Mode::Page,
                    "step" => Mode::Step,
                    other => return Err(format!("unknown mode `{other}`")),
                };
                if self.engine.grid.mode != want {
                    return Err(format!("engine mode: expected {want:?}, found {:?}", self.engine.grid.mode));
                }
            }
            ["engine", "step", t, s, what, "=", value] => {
                let (t, s): (usize, usize) = (t.parse().map_err(|_| "bad track")?, s.parse().map_err(|_| "bad step")?);
                if t >= 10 || !(1..=16).contains(&s) {
                    return Err("track is 0 to 9 and step is 1 to 16".to_string());
                }
                let step = &self.engine.grid.active_page().tracks[t].steps[s - 1];
                let got = match *what {
                    "active" => step.active,
                    "skip" => step.skip,
                    other => return Err(format!("unknown step field `{other}`")),
                };
                let want: bool = value.parse().map_err(|_| "expected true or false")?;
                if got != want {
                    return Err(format!("engine step {t} {s} {what}: expected {want}, found {got}"));
                }
            }
            ["command", text @ ..] => {
                let want = text.join(" ");
                if !self.last_commands.iter().any(|c| show_command(c) == want) {
                    let got: Vec<String> = self.last_commands.iter().map(show_command).collect();
                    return Err(format!("expected command `{want}`, the last input gave {got:?}"));
                }
            }
            ["commands", n] => {
                let want: usize = n.parse().map_err(|_| "bad count")?;
                if self.last_commands.len() != want {
                    let got: Vec<String> = self.last_commands.iter().map(show_command).collect();
                    return Err(format!("expected {want} command(s), the last input gave {got:?}"));
                }
            }
            ["intent", text @ ..] => {
                let want = text.join(" ");
                if !self.last_intents.iter().any(|i| show_intent(i) == want) {
                    let got: Vec<String> = self.last_intents.iter().map(show_intent).collect();
                    return Err(format!("expected intent `{want}`, the last input gave {got:?}"));
                }
            }
            ["intents", n] => {
                let want: usize = n.parse().map_err(|_| "bad count")?;
                if self.last_intents.len() != want {
                    let got: Vec<String> = self.last_intents.iter().map(show_intent).collect();
                    return Err(format!("expected {want} intent(s), the last input gave {got:?}"));
                }
            }
            _ => return Err("unknown expectation".to_string()),
        }
        Ok(())
    }
}

/// Runs one fixture. The error says which line failed and why.
fn run(source: &str, inv: &Inventory) -> Result<(), String> {
    let mut w = World::new(inv);
    for (n, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let result = match words.as_slice() {
            ["given", "step", rest @ ..] => w.given(rest),
            ["at", rest @ ..] => w.at(rest),
            ["expect", rest @ ..] => w.expect(rest),
            _ => Err("unknown directive".to_string()),
        };
        result.map_err(|e| format!("line {}: `{line}`: {e}", n + 1))?;
    }
    Ok(())
}

#[test]
fn all_panel_fixtures_pass() {
    let inv = inventory();
    let (fixtures, _pending) = panel_files();
    assert!(!fixtures.is_empty(), "no panel fixtures found");
    let mut failures = Vec::new();
    for path in &fixtures {
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        if let Err(e) = run(&source, &inv) {
            let name = path.strip_prefix(support::repo_root().join("tests/conformance/panel")).unwrap_or(path);
            failures.push(format!("{}: {e}", name.display()));
        }
    }
    assert!(failures.is_empty(), "{} of {} panel fixtures failed:\n{}", failures.len(), fixtures.len(), failures.join("\n"));
}

#[test]
fn a_fixture_with_a_wrong_expectation_fails_and_the_right_one_passes() {
    let inv = inventory();
    assert!(run("expect led step(3,5) = off\nexpect mode page\nexpect commands 0\n", &inv).is_ok());
    assert!(run("expect led step(3,5) = green steady\n", &inv).is_err());
    assert!(run("expect mode step\n", &inv).is_err());
    assert!(run("expect commands 1\n", &inv).is_err());
    assert!(run("expect intent SnapshotTake\n", &inv).is_err());
    assert!(run("expect engine step 3 5 active = true\n", &inv).is_err());
    assert!(run("given step 3 5 on\nexpect engine step 3 5 active = true\n", &inv).is_ok());
}

#[test]
fn the_runner_rejects_what_it_does_not_understand() {
    let inv = inventory();
    for bad in [
        "frobnicate\n",
        "expect led no.such.control = off\n",
        "expect led step(3,5) = purple steady\n",
        "at 0 ms click step(3,5)\nat 10 ms click step(3,5)\n", // 10 ms is inside the 40 ms click
        "at 0 ms wiggle step(3,5)\n",
        "given step 3 17 on\n",
        "given step 10 1 on\n",
    ] {
        assert!(run(bad, &inv).is_err(), "should have been rejected: {bad:?}");
    }
}
