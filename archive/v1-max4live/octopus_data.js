autowatch = 1;

/**
 * Octopus state is stored in a Max Dict named by STATE_DICT_NAME.
 * This file provides:
 * - deterministic default state creation
 * - schema enforcement (best-effort) + range clamping helpers
 * - small set of getters/setters for engine/UI
 *
 * NOTE: Max's JS runtime is not Node; avoid require(). Use include() from other JS files.
 */

var STATE_DICT_NAME = "octopus_state";

// ---------- Utilities ----------

function clampInt(v, min, max) {
  v = Math.round(Number(v));
  if (isNaN(v)) v = min;
  if (v < min) return min;
  if (v > max) return max;
  return v;
}

function clampFloat(v, min, max) {
  v = Number(v);
  if (isNaN(v)) v = min;
  if (v < min) return min;
  if (v > max) return max;
  return v;
}

function asBool(v) {
  return !!v;
}

function deepCopy(obj) {
  return JSON.parse(JSON.stringify(obj));
}

function ensureArrayLen(arr, len, fillValue) {
  if (!Array.isArray(arr)) arr = [];
  if (arr.length > len) arr = arr.slice(0, len);
  while (arr.length < len) arr.push(deepCopy(fillValue));
  return arr;
}

// ---------- Defaults (deterministic) ----------

function defaultGlobalScale() {
  return {
    enabled: false,
    root: 0,
    intervals: [0, 2, 4, 5, 7, 9, 11],
  };
}

function defaultPageScale() {
  return {
    enabled: false,
    locked: false,
    root: 60,
    intervals: [0, 2, 4, 5, 7, 9, 11],
  };
}

function defaultStep() {
  return {
    active: false,
    skip: false,
    pit_offset: 0,
    vel_offset: 0,
    len: 12, // 12 ticks = 1/16 at 192 PPQN
    len_multiplier: 1,
    sta_offset: 0,
    amt: 0,
    grv: 0,
    pos: 8,
    mcc_value: null,
    chords: [],
    polyphony: 1,
    ghost: false,
    hyperstep: null,
    strum: 0, // Phase 1: used by chord engine; 0..9 (negative supported in UI layer)
  };
}

function defaultTrack(trackIndex) {
  // Manual default PIT values for tracks 9..0:
  // C3, D3, E3, G3, A3, C5, D5, E5, G5, A5
  // Track indices in this spec are 0..9 and map to the above list in order.
  var manual = [57, 55, 52, 50, 48, 60, 62, 64, 67, 69]; // index 0..9 = [C3,D3,E3,G3,A3,C5,D5,E5,G5,A5]

  return {
    pit: clampInt(manual[trackIndex] != null ? manual[trackIndex] : 60, 0, 127),
    vel: 100,
    len_factor: 8,
    sta_factor: 8,
    dir: 1,
    amt: 0,
    grv: 0,
    mcc: null, // null | 0..127 | "bend" | "pressure"
    mch: 1, // 1..16 port1, 17..32 port2
    multiplier: "1",
    paused: false,
    muted: false,
    soloed: false,
    chain_head: null,
    chain_members: [],
    chain_base: "individual", // "individual" | "head"
    program_change: 0,
    bank_change: null,
    transpose_mch: null,
    transpose_mode: "relative",
    steps: ensureArrayLen([], 16, defaultStep()),
  };
}

function defaultPage() {
  var tracks = [];
  for (var i = 0; i < 10; i++) tracks.push(defaultTrack(i));

  return {
    pit_offset: 0,
    vel_factor: 8,
    len: 16,
    sta: 1,
    scale: defaultPageScale(),
    cluster_mode: false,
    mute_pattern: ensureArrayLen([], 10, false),
    tracks: tracks,
  };
}

function defaultBank() {
  var pages = [];
  for (var i = 0; i < 16; i++) pages.push(defaultPage());
  return { pages: pages };
}

function defaultGrid() {
  var banks = [];
  for (var i = 0; i < 10; i++) banks.push(defaultBank());

  // page_sets: 16 slots, each an array of {bank,page} pairs
  var pageSets = [];
  for (var s = 0; s < 16; s++) pageSets.push([]);

  return {
    tempo_bpm: 120.0,
    midi_port1_channel: 1,
    active_page: { bank: 0, page: 0 },
    page_sets: pageSets,
    global_scale: defaultGlobalScale(),
    banks: banks,
    // routing mode: "octopus" | "fixed"
    midi_routing_mode: "octopus",
    fixed_routing: { base_channel_port1: 1, base_channel_port2: 1 },
  };
}

// ---------- Dict access ----------

function getStateDict() {
  return new Dict(STATE_DICT_NAME);
}

function reset_state() {
  var d = getStateDict();
  d.parse(JSON.stringify(defaultGrid()));
  outlet(0, "state_reset");
}

function ensure_state() {
  var d = getStateDict();
  if (!d.getkeys || d.getkeys() === null) {
    reset_state();
    return;
  }
  var changed = 0;
  var def = defaultGrid();
  function ensureRoot(key) {
    if (d.get(key) == null) {
      d.set(key, def[key]);
      changed = 1;
    }
  }
  ensureRoot("banks");
  ensureRoot("active_page");
  ensureRoot("page_sets");
  ensureRoot("global_scale");
  ensureRoot("midi_routing_mode");
  ensureRoot("fixed_routing");
  ensureRoot("tempo_bpm");

  // Repair active page shape/ranges without wiping other state.
  var ap = d.get("active_page") || {};
  var repairedAp = {
    bank: clampInt(ap.bank, 0, 9),
    page: clampInt(ap.page, 0, 15),
  };
  if (ap.bank !== repairedAp.bank || ap.page !== repairedAp.page) {
    d.set("active_page", repairedAp);
    changed = 1;
  }

  // Light schema marker for future migrations.
  if (d.get("schema_version") == null) {
    d.set("schema_version", 1);
    changed = 1;
  }

  if (changed) outlet(0, "state_repaired");
  outlet(0, "state_ok");
}

function get_active_page() {
  var d = getStateDict();
  var ap = d.get("active_page");
  if (!ap) ap = { bank: 0, page: 0 };
  var bank = clampInt(ap.bank, 0, 9);
  var page = clampInt(ap.page, 0, 15);
  outlet(0, "active_page", bank, page);
}

function _activePageRef(d) {
  var ap = d.get("active_page") || { bank: 0, page: 0 };
  return {
    bank: clampInt(ap.bank, 0, 9),
    page: clampInt(ap.page, 0, 15),
  };
}

function _stepBasePath(d, trackIndex, stepIndex) {
  var ap = _activePageRef(d);
  return [
    "banks", ap.bank,
    "pages", ap.page,
    "tracks", trackIndex,
    "steps", stepIndex,
  ].join("::");
}

function set_active_page(bank, page) {
  bank = clampInt(bank, 0, 9);
  page = clampInt(page, 0, 15);
  var d = getStateDict();
  d.set("active_page", { bank: bank, page: page });
  outlet(0, "active_page", bank, page);
}

function set_midi_routing_mode(mode) {
  var m = String(mode || "octopus").toLowerCase();
  if (m !== "octopus" && m !== "fixed") m = "octopus";
  var d = getStateDict();
  d.set("midi_routing_mode", m);
  outlet(0, "midi_routing_mode", m);
}

function set_fixed_routing(base_channel_port1, base_channel_port2) {
  var b1 = clampInt(base_channel_port1, 1, 16);
  var b2 = clampInt(base_channel_port2, 1, 16);
  var d = getStateDict();
  d.set("fixed_routing", { base_channel_port1: b1, base_channel_port2: b2 });
  outlet(0, "fixed_routing", b1, b2);
}

// Minimal setters used by early UI scaffolding.
function step_toggle(trackIndex, stepIndex) {
  trackIndex = clampInt(trackIndex, 0, 9);
  stepIndex = clampInt(stepIndex, 0, 15);
  var d = getStateDict();
  var base = _stepBasePath(d, trackIndex, stepIndex);
  var cur = d.get(base + "::active");
  d.set(base + "::active", !cur);
  outlet(0, "step_active", trackIndex, stepIndex, !cur);
}

function step_skip_toggle(trackIndex, stepIndex) {
  trackIndex = clampInt(trackIndex, 0, 9);
  stepIndex = clampInt(stepIndex, 0, 15);
  var d = getStateDict();
  var base = _stepBasePath(d, trackIndex, stepIndex);
  var cur = d.get(base + "::skip");
  d.set(base + "::skip", !cur);
  outlet(0, "step_skip", trackIndex, stepIndex, !cur);
}

function step_pitch(trackIndex, stepIndex, pitOffset) {
  trackIndex = clampInt(trackIndex, 0, 9);
  stepIndex = clampInt(stepIndex, 0, 15);
  pitOffset = clampInt(pitOffset, -127, 127);
  var d = getStateDict();
  var base = _stepBasePath(d, trackIndex, stepIndex);
  d.set(base + "::pit_offset", pitOffset);
  outlet(0, "step_pitch", trackIndex, stepIndex, pitOffset);
}

function set_scale(scaleId) {
  var s = String(scaleId || "maj").toLowerCase();
  if (s === "chromatic") s = "chr";
  else if (s === "major") s = "maj";
  else if (s === "minor") s = "min";
  else if (s === "modal") s = "mod";
  var intervalsByMode = {
    chr: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    maj: [0, 2, 4, 5, 7, 9, 11],
    min: [0, 2, 3, 5, 7, 8, 10],
    mod: [0, 2, 3, 5, 7, 9, 10],
  };
  if (!intervalsByMode[s]) s = "maj";
  var d = getStateDict();
  var ap = _activePageRef(d);
  var pageScalePath = ["banks", ap.bank, "pages", ap.page, "scale"].join("::");
  d.set(pageScalePath + "::enabled", 1);
  d.set(pageScalePath + "::intervals", intervalsByMode[s]);
  d.set(pageScalePath + "::mode", s);
  outlet(0, "scale_mode", s);
}

function set_scale_enabled(v) {
  var enabled = asBool(v) ? 1 : 0;
  var d = getStateDict();
  var ap = _activePageRef(d);
  var pageScalePath = ["banks", ap.bank, "pages", ap.page, "scale"].join("::");
  d.set(pageScalePath + "::enabled", enabled);
  outlet(0, "scale_enabled", enabled);
}

function set_attr(attr) {
  var allowed = {
    vel: 1, pit: 1, len: 1, sta: 1, pos: 1,
    dir: 1, amt: 1, grv: 1, mcc: 1, mch: 1
  };
  var a = String(attr || "vel").toLowerCase();
  if (!allowed[a]) a = "vel";
  var d = getStateDict();
  d.set("ui_mix_target", a);
  outlet(0, "mix_target", a);
}

function set_mode(mode) {
  var allowed = { grid: 1, page: 1, track: 1, step: 1 };
  var m = String(mode || "grid").toLowerCase();
  if (!allowed[m]) m = "grid";
  var d = getStateDict();
  d.set("ui_mode", m);
  outlet(0, "ui_mode", m);
}

function transport(running) {
  outlet(0, "transport_state", asBool(running) ? 1 : 0);
}

function rec_toggle() {
  var d = getStateDict();
  var armed = asBool(d.get("record_armed"));
  d.set("record_armed", !armed);
  outlet(0, "record_armed", !armed ? 1 : 0);
}

// ---------- Dump helpers ----------

function dump_page() {
  var d = getStateDict();
  var ap = d.get("active_page") || { bank: 0, page: 0 };
  var pageObj = d.get(
    ["banks", clampInt(ap.bank, 0, 9), "pages", clampInt(ap.page, 0, 15)].join("::")
  );
  outlet(0, "page_dump", JSON.stringify(pageObj));
}

