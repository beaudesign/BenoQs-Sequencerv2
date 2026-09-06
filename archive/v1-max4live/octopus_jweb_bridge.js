autowatch = 1;

/**
 * Bridge between jweb (HTML UI) and octopus_data.js.
 *
 * Inlet 0  <- jweb outlet   (user interactions from HTML: step_toggle, etc.)
 * Inlet 1  <- octopus_data  (state change notifications: step_active, etc.)
 *
 * Outlet 0 -> jweb inlet    (script calls: script updateStep …, script updatePlayhead …)
 * Outlet 1 -> octopus_data  (forwarded commands from HTML)
 */

inlets  = 2;
outlets = 2;

// ── From jweb → octopus_data ─────────────────────────────────────────────────
// jweb sends messages like "step_toggle 3 7" — Max routes these here as
// the selector "step_toggle" with integer arguments [3, 7].

function step_toggle(track, step) {
  if (inlet !== 0) return;
  outlet(1, "step_toggle", track, step);
}

function step_skip_toggle(track, step) {
  if (inlet !== 0) return;
  outlet(1, "step_skip_toggle", track, step);
}

function set_active_page(bank, page) {
  if (inlet !== 0) return;
  outlet(1, "set_active_page", bank, page);
}

function set_attr(attr) {
  if (inlet !== 0) return;
  outlet(1, "set_attr", attr);
}

function set_mode(mode) {
  if (inlet !== 0) return;
  outlet(1, "set_mode", mode);
}

function step_pitch(track, step, pitch) {
  if (inlet !== 0) return;
  outlet(1, "step_pitch", track, step, pitch);
}

function set_scale(scaleId) {
  if (inlet !== 0) return;
  outlet(1, "set_scale", scaleId);
}

function rec_toggle() {
  if (inlet !== 0) return;
  outlet(1, "rec_toggle");
}

function transport(running) {
  if (inlet !== 0) return;
  outlet(1, "transport", running);
}

function reset_phase() {
  if (inlet !== 0) return;
  outlet(1, "reset_state");
}

function randomize(track) { if (inlet !== 0) return; outlet(1, "randomize", track); }
function clear(track)     { if (inlet !== 0) return; outlet(1, "clear",     track); }
function reverse(track)   { if (inlet !== 0) return; outlet(1, "reverse",   track); }
function rotate(track)    { if (inlet !== 0) return; outlet(1, "rotate",    track); }

// ── From octopus_data → jweb ─────────────────────────────────────────────────
// data.js outputs: step_active <track> <step> <1|0>
//                  step_skip   <track> <step> <1|0>
// We forward as:   script updateStep <track> <step> <active> <skip> <hasChord>
//
// We buffer the last active & skip values per cell so we can merge them
// into one updateStep call regardless of which arrives first.

var _stepCache = {};   // key "ti_si" -> {active:0, skip:0}

function _cacheKey(ti, si) { return ti + "_" + si; }

function step_active(ti, si, val) {
  if (inlet !== 1) return;
  var k = _cacheKey(ti, si);
  if (!_stepCache[k]) _stepCache[k] = { active: 0, skip: 0 };
  _stepCache[k].active = val ? 1 : 0;
  outlet(0, "script", "updateStep", ti, si, _stepCache[k].active, _stepCache[k].skip, 0);
}

function step_skip(ti, si, val) {
  if (inlet !== 1) return;
  var k = _cacheKey(ti, si);
  if (!_stepCache[k]) _stepCache[k] = { active: 0, skip: 0 };
  _stepCache[k].skip = val ? 1 : 0;
  outlet(0, "script", "updateStep", ti, si, _stepCache[k].active, _stepCache[k].skip, 0);
}

// Engine can call: playhead_update <track> <pos>
function playhead_update(ti, pos) {
  if (inlet !== 1) return;
  outlet(0, "script", "updatePlayhead", ti, pos);
}

// Tempo updates from transport
function tempo_update(bpm) {
  if (inlet !== 1) return;
  outlet(0, "script", "setTempo", bpm);
}

// Transport state changes
function transport_state(running) {
  if (inlet !== 1) return;
  outlet(0, "script", "setTransport", running ? 1 : 0);
}

// Bulk page load (JSON string from dump_page)
function page_dump(json) {
  if (inlet !== 1) return;
  outlet(0, "script", "loadPage", json);
}

// Catch-all for unhandled messages (avoids Max error spam)
function anything() {}
