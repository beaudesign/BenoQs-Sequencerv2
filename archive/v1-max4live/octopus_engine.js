autowatch = 1;

include("octopus_data.js"); // for STATE_DICT_NAME + dict schema
include("octopus_scale.js");

// Inlets/outlets
inlets = 1;
outlets = 1; // messages to patcher: noteon/noteoff/cc/allnotesoff/debug

// Constants
var TICKS_PER_QN = 192;
var DEFAULT_STEP_TICKS_X1 = 12; // 1/16 note = 12 ticks at 192 PPQN

// Engine state (runtime only; not serialized)
var running = 0;
var globalTick = 0; // monotonically increasing while running

// Per-track runtime state
var trackRt = []; // length 10

function _rtInit() {
  trackRt = [];
  for (var i = 0; i < 10; i++) {
    trackRt.push({
      accum: 0.0,
      pos: 0,
      pingDir: +1,
      chain: { memberIdx: 0, segPos: 0, pingDir: +1 },
      // note bookkeeping for panic / noteoffs: map key "port:chan:pitch" -> count
      held: {},
    });
  }
}
_rtInit();

// Tick-scheduled MIDI events
// {t, type, port, ch, pitch, vel, cc, val}
var eventQ = [];

function reset_runtime() {
  running = 0;
  globalTick = 0;
  eventQ = [];
  _rtInit();
  outlet(0, "debug", "engine_reset");
}

// --- Helpers ---

function _parseMultiplier(multStr) {
  if (multStr === null || multStr === undefined) return 1.0;
  var s = String(multStr).trim();
  if (!s.length) return 1.0;
  if (s.indexOf("/") !== -1) {
    var parts = s.split("/");
    if (parts.length === 2) {
      var a = parseFloat(parts[0]);
      var b = parseFloat(parts[1]);
      if (!isNaN(a) && !isNaN(b) && b !== 0) return a / b;
    }
  }
  var f = parseFloat(s);
  if (!isNaN(f) && f > 0) return f;
  return 1.0;
}

function _grvDelayTicks(grv) {
  // Manual table (ticks at 1/192) for track shuffle; even settings are random in a range.
  var g = Math.max(0, Math.min(16, Math.round(Number(grv) || 0)));
  switch (g) {
    case 0:
      return 0;
    case 1:
      return 1;
    case 2:
      return Math.floor(Math.random() * 3); // 0..2
    case 3:
      return 2;
    case 4:
      return 1 + Math.floor(Math.random() * 3); // 1..3
    case 5:
      return 3;
    case 6:
      return 2 + Math.floor(Math.random() * 3); // 2..4
    case 7:
      return 4;
    case 8:
      return 3 + Math.floor(Math.random() * 3); // 3..5
    case 9:
      return 5;
    case 10:
      return 4 + Math.floor(Math.random() * 3); // 4..6
    case 11:
      return 6;
    case 12:
      return 5 + Math.floor(Math.random() * 3); // 5..7
    case 13:
      return 7;
    case 14:
      return 6 + Math.floor(Math.random() * 3); // 6..8
    case 15:
      return 8;
    case 16:
      return 7 + Math.floor(Math.random() * 3); // 7..9
  }
  return 0;
}

function _clampMidi(v) {
  v = Math.round(Number(v));
  if (isNaN(v)) return 0;
  if (v < 0) return 0;
  if (v > 127) return 127;
  return v;
}

function _resolvePortChan(mch, routingMode, fixedRouting, trackIndex) {
  // Returns {port:1|2, ch:1..16}
  if (routingMode === "fixed") {
    // Simple fixed mapping: trackIndex 0..9 -> base + trackIndex, wrapped 1..16 on port1 by default.
    var base1 = fixedRouting && fixedRouting.base_channel_port1 ? fixedRouting.base_channel_port1 : 1;
    var ch1 = ((base1 - 1 + trackIndex) % 16) + 1;
    return { port: 1, ch: ch1 };
  }

  var v = Math.round(Number(mch) || 1);
  if (v <= 16) return { port: 1, ch: Math.max(1, Math.min(16, v)) };
  return { port: 2, ch: Math.max(1, Math.min(16, v - 16)) };
}

function _schedule(ev) {
  // Keep queue sorted by event tick for efficient due-event flushing.
  var lo = 0;
  var hi = eventQ.length;
  while (lo < hi) {
    var mid = (lo + hi) >> 1;
    if (eventQ[mid].t <= ev.t) lo = mid + 1;
    else hi = mid;
  }
  eventQ.splice(lo, 0, ev);
}

function _strumOffsetTicks(levelAbs, noteNumber1Based) {
  // Manual table: "Chord Strum Timings in Ticks" for strum 1..9, note 2..7.
  // noteNumber1Based: 1 is first note (no delay)
  if (noteNumber1Based <= 1) return 0;
  var lvl = Math.max(0, Math.min(9, Math.round(levelAbs || 0)));
  if (lvl === 0) return 0;
  // index by level 1..9
  var table = {
    2: [0, 1, 1, 2, 2, 3, 3, 4, 5],
    3: [1, 2, 3, 4, 5, 6, 7, 8, 10],
    4: [1, 2, 4, 6, 8, 9, 10, 13, 17],
    5: [2, 3, 5, 9, 11, 13, 15, 19, 23],
    6: [2, 3, 6, 12, 15, 18, 21, 27, 30],
    7: [3, 6, 9, 16, 19, 24, 29, 36, 45],
  };
  var arr = table[noteNumber1Based];
  if (!arr) return 0;
  return arr[lvl - 1] || 0;
}

function _flushDueEvents(nowTick) {
  if (!eventQ.length) return;
  // Since queue is sorted by tick, due events are always a prefix.
  var i = 0;
  while (i < eventQ.length && eventQ[i].t <= nowTick) {
    var ev = eventQ[i];
    if (ev.type === "noteon") outlet(0, "noteon", ev.port, ev.ch, ev.pitch, ev.vel);
    else if (ev.type === "noteoff") outlet(0, "noteoff", ev.port, ev.ch, ev.pitch, ev.vel || 0);
    else if (ev.type === "cc") outlet(0, "cc", ev.port, ev.ch, ev.cc, ev.val);
    i++;
  }
  if (i > 0) eventQ = eventQ.slice(i);
}

function _allNotesOff() {
  // Emit CC123 (all notes off) across both ports/channels for robust panic behavior.
  for (var port = 1; port <= 2; port++) {
    for (var ch = 1; ch <= 16; ch++) {
      outlet(0, "cc", port, ch, 123, 0);
    }
  }
  // Keep legacy panic event for any existing patcher listeners.
  outlet(0, "allnotesoff");
  eventQ = [];
  for (var t = 0; t < trackRt.length; t++) trackRt[t].held = {};
}

// --- Public messages from patcher ---

function transport_state(v) {
  running = Number(v) ? 1 : 0;
  if (!running) {
    _allNotesOff();
  }
}

function tick() {
  if (!running) return;

  ensure_state(); // from octopus_data.js; ensures dict exists
  var d = new Dict(STATE_DICT_NAME);
  var grid = d.getkeys ? d : null;
  if (!grid) return;

  globalTick++;

  var routingMode = d.get("midi_routing_mode") || "octopus";
  var fixedRouting = d.get("fixed_routing") || { base_channel_port1: 1, base_channel_port2: 1 };

  var ap = d.get("active_page") || { bank: 0, page: 0 };
  var bank = Math.max(0, Math.min(9, Math.round(ap.bank || 0)));
  var pageIx = Math.max(0, Math.min(15, Math.round(ap.page || 0)));

  var pagePath = ["banks", bank, "pages", pageIx].join("::");
  var page = d.get(pagePath);
  if (!page || !page.tracks) {
    _flushDueEvents(globalTick);
    return;
  }

  // process each track
  for (var ti = 0; ti < 10; ti++) {
    var track = page.tracks[ti];
    if (!track) continue;
    if (track.paused || track.muted) continue;
    // If this track is a chain member (not head), it is played by its chain head.
    if (track.chain_head !== null && track.chain_head !== undefined) continue;

    // Chain head: decide which track's steps are currently active.
    var isChainHead = track.chain_members && track.chain_members.length;
    var activeTrack = track;
    var baseTrack = track;
    var rt = trackRt[ti];
    // MVP guard: UI exposes 16 physical steps, so keep runtime page length in that range.
    var pageLen = Math.max(1, Math.min(16, Math.round(page.len || 16)));

    if (isChainHead) {
      var order = [ti];
      for (var cm = 0; cm < track.chain_members.length; cm++) {
        var idx = Math.max(0, Math.min(9, Math.round(track.chain_members[cm])));
        if (order.indexOf(idx) === -1) order.push(idx);
      }
      // Hardware default play order is top-to-bottom (row 9 -> row 0).
      order.sort(function (a, b) { return b - a; });
      var memberIdx = rt.chain.memberIdx % order.length;
      var playingTi = order[memberIdx];
      activeTrack = page.tracks[playingTi] || track;
      if (track.chain_base === "head") baseTrack = track;
      else baseTrack = activeTrack;
    }

    var mult = _parseMultiplier(baseTrack.multiplier);
    var stepTicks = DEFAULT_STEP_TICKS_X1 / mult;
    if (stepTicks <= 0) stepTicks = DEFAULT_STEP_TICKS_X1;

    rt.accum += 1.0;

    if (rt.accum + 1e-9 < stepTicks) continue;
    rt.accum -= stepTicks;

    // step boundary reached: compute next step index according to DIR
    var stepIndex = (isChainHead ? rt.chain.segPos : rt.pos) % pageLen;
    if (stepIndex < 0) stepIndex += pageLen;

    // Skip logic: if current step is skipped, still advance position but no output.
    // Octopus has 16 physical steps per track; for pageLen>16 this becomes multi-cycle;
    // Phase 1 keeps 16 steps per track, so we wrap to 0..15 for lookup.
    var stepObj = activeTrack.steps ? activeTrack.steps[stepIndex % 16] : null;

    // Determine shuffle delay for even steps (2,4,...16) -> indices 1,3,...15 in 0-based.
    var shuffleDelay = 0;
    if ((stepIndex % 2) === 1) shuffleDelay = _grvDelayTicks(baseTrack.grv);

    if (stepObj && !stepObj.skip && stepObj.active) {
      var finalPit = (page.pit_offset || 0) + (baseTrack.pit || 60) + (stepObj.pit_offset || 0);
      finalPit = _clampMidi(finalPit);
      if (page.scale && page.scale.enabled) {
        var pcs = buildScalePitchClasses(page.scale.root, page.scale.intervals);
        finalPit = quantizeToScale(finalPit, pcs);
      }

      var finalVel = _clampMidi((baseTrack.vel || 0) + (stepObj.vel_offset || 0));
      // page vel factor: 8 = neutral
      var vf = Number(page.vel_factor);
      if (isNaN(vf)) vf = 8;
      finalVel = _clampMidi(Math.round((finalVel * vf) / 8));

      // STA: simple factor scaling (neutral 8 => 1.0; 0 => 0; 16 => 2.0)
      var staScale = Math.max(0, Math.min(2, Number(baseTrack.sta_factor) / 8));
      var staTicks = Math.round((stepObj.sta_offset || 0) * staScale);

      // LEN: apply len_multiplier then scale offsets similarly (neutral 8)
      var baseLen = Math.max(1, Math.min(192, Math.round(stepObj.len || 12)));
      var lenMult = Math.max(1, Math.min(8, Math.round(stepObj.len_multiplier || 1)));
      var rawLen = Math.min(192, baseLen * lenMult);
      var lenScale = Math.max(0, Math.min(2, Number(baseTrack.len_factor) / 8));
      var finalLen = Math.max(1, Math.min(192, Math.round(rawLen * lenScale)));

      var portChan = _resolvePortChan(baseTrack.mch, routingMode, fixedRouting, ti);
      var onTick = globalTick + shuffleDelay + staTicks;

      // Chords (Phase 1 engine supports up to 7; UI may cap later)
      var notes = [0];
      if (stepObj.chords && stepObj.chords.length) {
        // chord offsets are semitones above base; pool includes base as 0 implicitly
        var pool = [0];
        for (var ci = 0; ci < stepObj.chords.length; ci++) pool.push(stepObj.chords[ci]);
        var chordSize = pool.length;
        var poly = Math.max(1, Math.min(7, Math.round(stepObj.polyphony || 1)));
        var toPlay = Math.min(poly, chordSize);
        notes = [];
        if (poly === chordSize) {
          for (var pi = 0; pi < toPlay; pi++) notes.push(pool[pi]);
        } else {
          // random picks without replacement
          var used = {};
          while (notes.length < toPlay) {
            var r = pool[Math.floor(Math.random() * chordSize)];
            if (used[r]) continue;
            used[r] = 1;
            notes.push(r);
          }
        }
      }

      // Strum (manual behavior): strum>0 = up, strum<0 = down. Applies to single-note steps too (duplicates).
      var strum = stepObj.strum || 0;
      strum = Math.max(-9, Math.min(9, Math.round(Number(strum) || 0)));
      var levelAbs = Math.abs(strum);

      var played = [];
      for (var ni = 0; ni < notes.length; ni++) played.push(_clampMidi(finalPit + notes[ni]));
      played.sort(function (a, b) {
        return a - b;
      });
      if (strum < 0) played.reverse();

      // Single note + strum -> emit 6 duplicates at strum timings
      if (levelAbs > 0 && played.length === 1) {
        var basePitch = played[0];
        for (var k = 1; k <= 7; k++) {
          var off = _strumOffsetTicks(levelAbs, k);
          _schedule({ t: onTick + off, type: "noteon", port: portChan.port, ch: portChan.ch, pitch: basePitch, vel: finalVel });
          _schedule({ t: onTick + off + finalLen, type: "noteoff", port: portChan.port, ch: portChan.ch, pitch: basePitch, vel: 0 });
        }
      } else {
        for (var pi = 0; pi < played.length; pi++) {
          var off2 = _strumOffsetTicks(levelAbs, pi + 1);
          _schedule({ t: onTick + off2, type: "noteon", port: portChan.port, ch: portChan.ch, pitch: played[pi], vel: finalVel });
          _schedule({ t: onTick + off2 + finalLen, type: "noteoff", port: portChan.port, ch: portChan.ch, pitch: played[pi], vel: 0 });
        }
      }

      // Step-level CC value if track.mcc set + step has mcc_value
      if (baseTrack.mcc !== null && baseTrack.mcc !== undefined && stepObj.mcc_value !== null && stepObj.mcc_value !== undefined) {
        if (baseTrack.mcc === "bend") {
          // Phase 1: emit as generic "cc" message with cc=-1; patcher can map to bend.
          _schedule({ t: onTick, type: "cc", port: portChan.port, ch: portChan.ch, cc: -1, val: _clampMidi(stepObj.mcc_value) });
        } else if (baseTrack.mcc === "pressure") {
          _schedule({ t: onTick, type: "cc", port: portChan.port, ch: portChan.ch, cc: -2, val: _clampMidi(stepObj.mcc_value) });
        } else {
          _schedule({ t: onTick, type: "cc", port: portChan.port, ch: portChan.ch, cc: _clampMidi(baseTrack.mcc), val: _clampMidi(stepObj.mcc_value) });
        }
      }
    }

    // Advance playhead per direction
    var dir = Math.round(Number(activeTrack.dir) || 1);
    if (isChainHead) {
      // Chain segment advances using the active member's direction; segment wraps to next member on completion.
      if (dir === 1) rt.chain.segPos = (rt.chain.segPos + 1) % pageLen;
      else if (dir === 2) rt.chain.segPos = (rt.chain.segPos - 1 + pageLen) % pageLen;
      else if (dir === 3) {
        if (rt.chain.segPos <= 0) rt.chain.pingDir = +1;
        else if (rt.chain.segPos >= pageLen - 1) rt.chain.pingDir = -1;
        rt.chain.segPos = rt.chain.segPos + rt.chain.pingDir;
      } else if (dir === 4) {
        rt.chain.segPos = (rt.chain.segPos + (Math.random() < 0.6667 ? 1 : -1) + pageLen) % pageLen;
      } else if (dir === 5) {
        rt.chain.segPos = Math.floor(Math.random() * pageLen);
      } else {
        rt.chain.segPos = (rt.chain.segPos + 1) % pageLen;
      }

      // When segment completes a full page cycle in forward-ish modes, advance member.
      // For MVP we advance member when segPos wraps to 0.
      if (rt.chain.segPos === 0) {
        var chainLen = 1 + (track.chain_members ? track.chain_members.length : 0);
        rt.chain.memberIdx = (rt.chain.memberIdx + 1) % Math.max(1, chainLen);
      }
    } else {
      if (dir === 1) rt.pos = (rt.pos + 1) % pageLen;
      else if (dir === 2) rt.pos = (rt.pos - 1 + pageLen) % pageLen;
      else if (dir === 3) {
        // ping-pong: bounce at edges
        if (rt.pos <= 0) rt.pingDir = +1;
        else if (rt.pos >= pageLen - 1) rt.pingDir = -1;
        rt.pos = rt.pos + rt.pingDir;
      } else if (dir === 4) {
        // brownian: 2/3 forward, 1/3 reverse
        rt.pos = (rt.pos + (Math.random() < 0.6667 ? 1 : -1) + pageLen) % pageLen;
      } else if (dir === 5) {
        rt.pos = Math.floor(Math.random() * pageLen);
      } else {
        rt.pos = (rt.pos + 1) % pageLen;
      }
    }
  }

  _flushDueEvents(globalTick);
}

