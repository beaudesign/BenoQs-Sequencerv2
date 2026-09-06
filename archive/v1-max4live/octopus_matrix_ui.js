autowatch = 1;

// JSUI matrix renderer (PAGE view MVP).
// Draws 10x16 step buttons + simple track labels.
// Emits messages: "step_toggle <track> <step>" on click.

inlets = 1;
outlets = 1;

mgraphics.init();
mgraphics.relative_coords = 0;
mgraphics.autofill = 0;

var STATE_DICT_NAME = "octopus_state";

var W = 520;
var H = 520;

var rows = 10;
var cols = 16;

var cell = 14;
var gap = 2;
var padLeft = 40;
var padTop = 42;

var mode = "page"; // page|track|step|grid
var selectedTrack = 0;
var selectedStep = 0;
var selectedAttr = "vel"; // vel|pit|len|sta|amt|grv|mcc

var colors = {
  bg: [0.10, 0.10, 0.10, 1.0],
  off: [0.16, 0.16, 0.16, 1.0],
  offBorder: [0.25, 0.25, 0.25, 1.0],
  active: [0.27, 0.87, 0.27, 1.0], // #44dd44
  skip: [0.87, 0.20, 0.20, 1.0], // #dd3333
  chord: [0.87, 0.53, 0.20, 1.0], // #dd8833
  label: [0.8, 0.8, 0.8, 1.0],
  sel: [1.0, 0.53, 0.0, 1.0],
};

function _clampInt(v, min, max) {
  v = Math.round(Number(v));
  if (isNaN(v)) v = min;
  if (v < min) return min;
  if (v > max) return max;
  return v;
}

function _getStep(trackIx, stepIx) {
  var d = new Dict(STATE_DICT_NAME);
  var ap = d.get("active_page") || { bank: 0, page: 0 };
  var bank = _clampInt(ap.bank, 0, 9);
  var page = _clampInt(ap.page, 0, 15);
  var base = ["banks", bank, "pages", page, "tracks", trackIx, "steps", stepIx].join("::");
  var active = !!d.get(base + "::active");
  var skip = !!d.get(base + "::skip");
  var chords = d.get(base + "::chords");
  var hasChord = Array.isArray(chords) && chords.length > 0;
  return { active: active, skip: skip, hasChord: hasChord };
}

function _getActivePageSnapshot() {
  var d = new Dict(STATE_DICT_NAME);
  var ap = d.get("active_page") || { bank: 0, page: 0 };
  var bank = _clampInt(ap.bank, 0, 9);
  var page = _clampInt(ap.page, 0, 15);
  var pageObj = d.get(["banks", bank, "pages", page].join("::")) || {};
  var tracks = pageObj.tracks || [];
  return { d: d, bank: bank, page: page, pageObj: pageObj, tracks: tracks };
}

function paint() {
  var g = mgraphics;
  var snapHdr = _getActivePageSnapshot();
  var uiMode = String(snapHdr.d.get("ui_mode") || mode || "page").toLowerCase();
  var uiMix = String(snapHdr.d.get("ui_mix_target") || selectedAttr || "vel").toLowerCase();
  var scalePath = ["banks", snapHdr.bank, "pages", snapHdr.page, "scale"].join("::");
  var scaleEnabled = !!snapHdr.d.get(scalePath + "::enabled");
  var scaleMode = String(snapHdr.d.get(scalePath + "::mode") || "maj").toUpperCase();
  mode = uiMode;
  selectedAttr = uiMix;

  g.set_source_rgba(colors.bg);
  g.rectangle(0, 0, box.rect[2], box.rect[3]);
  g.fill();

  // Persistent context labels (mode + edit lens + focus + page scale state)
  g.set_source_rgba(colors.label);
  g.select_font_face("Courier", "normal", "normal");
  g.set_font_size(10);
  g.move_to(8, 18);
  g.show_text("MODE " + mode.toUpperCase() + "  ATR " + selectedAttr.toUpperCase() + "  TRK " + selectedTrack + "  STP " + selectedStep);
  g.move_to(8, 30);
  g.show_text("BANK " + snapHdr.bank + "  PAGE " + snapHdr.page + "  SCALE " + (scaleEnabled ? "ON" : "OFF") + " " + scaleMode);

  if (mode === "grid") {
    _paintGrid(g);
    return;
  }
  if (mode === "track") {
    _paintTrack(g);
    return;
  }
  if (mode === "step") {
    _paintStep(g);
    return;
  }

  var snap = _getActivePageSnapshot();
  // Draw grid
  for (var r = 0; r < rows; r++) {
    // track label
    g.set_source_rgba(colors.label);
    g.move_to(10, padTop + r * (cell + gap) + cell - 3);
    g.show_text(String(r));

    for (var c = 0; c < cols; c++) {
      var x = padLeft + c * (cell + gap);
      var y = padTop + r * (cell + gap);
      var stObj = (snap.tracks[r] && snap.tracks[r].steps) ? snap.tracks[r].steps[c] : null;
      var st = {
        active: !!(stObj && stObj.active),
        skip: !!(stObj && stObj.skip),
        hasChord: !!(stObj && stObj.chords && stObj.chords.length > 0),
      };

      var fill = colors.off;
      if (st.skip && st.active) fill = colors.skip; // MVP: no flashing, just red
      else if (st.skip) fill = colors.skip;
      else if (st.hasChord) fill = colors.chord;
      else if (st.active) fill = colors.active;

      // button
      g.set_source_rgba(fill);
      g.rectangle_rounded(x, y, cell, cell, 3, 3);
      g.fill();

      g.set_source_rgba(colors.offBorder);
      g.rectangle_rounded(x + 0.5, y + 0.5, cell - 1, cell - 1, 3, 3);
      g.stroke();
    }
  }
}

function _paintGrid(g) {
  // 10x16 pages: rows=banks, cols=pages
  var d = new Dict(STATE_DICT_NAME);
  var ap = d.get("active_page") || { bank: 0, page: 0 };
  var ab = _clampInt(ap.bank, 0, 9);
  var apg = _clampInt(ap.page, 0, 15);

  for (var r = 0; r < rows; r++) {
    g.set_source_rgba(colors.label);
    g.move_to(10, padTop + r * (cell + gap) + cell - 3);
    g.show_text(String(r));

    for (var c = 0; c < cols; c++) {
      var x = padLeft + c * (cell + gap);
      var y = padTop + r * (cell + gap);
      var isActive = r === ab && c === apg;

      g.set_source_rgba(isActive ? colors.sel : colors.off);
      g.rectangle_rounded(x, y, cell, cell, 3, 3);
      g.fill();
      g.set_source_rgba(colors.offBorder);
      g.rectangle_rounded(x + 0.5, y + 0.5, cell - 1, cell - 1, 3, 3);
      g.stroke();
    }
  }
}

function _paintTrack(g) {
  // Row 0: step toggles for selectedTrack
  g.set_source_rgba(colors.label);
  g.move_to(8, 36);
  g.show_text("TRK " + selectedTrack + "  ATR " + selectedAttr.toUpperCase());

  var snap = _getActivePageSnapshot();
  var tr = snap.tracks[selectedTrack] || {};
  var trSteps = tr.steps || [];
  for (var c = 0; c < cols; c++) {
    var x = padLeft + c * (cell + gap);
    var y = padTop;
    var stObj = trSteps[c] || null;
    var st = {
      active: !!(stObj && stObj.active),
      skip: !!(stObj && stObj.skip),
      hasChord: !!(stObj && stObj.chords && stObj.chords.length > 0),
    };
    var fill = st.active ? colors.active : colors.off;
    if (st.skip) fill = colors.skip;
    g.set_source_rgba(fill);
    g.rectangle_rounded(x, y, cell, cell, 3, 3);
    g.fill();
    g.set_source_rgba(colors.offBorder);
    g.rectangle_rounded(x + 0.5, y + 0.5, cell - 1, cell - 1, 3, 3);
    g.stroke();
  }

  // Rows 1-9: simple bar map for selected attribute (MVP: vel_offset only)
  var d = snap.d;
  var base = ["banks", snap.bank, "pages", snap.page, "tracks", selectedTrack, "steps"].join("::");

  for (var c2 = 0; c2 < cols; c2++) {
    var stepBase = base + "::" + c2;
    var v = 0;
    if (selectedAttr === "vel") v = Number(d.get(stepBase + "::vel_offset") || 0);
    else if (selectedAttr === "pit") v = Number(d.get(stepBase + "::pit_offset") || 0);
    else if (selectedAttr === "sta") v = Number(d.get(stepBase + "::sta_offset") || 0);
    else if (selectedAttr === "amt") v = Number(d.get(stepBase + "::amt") || 0);
    else if (selectedAttr === "len") v = Number(d.get(stepBase + "::len") || 12);

    var x2 = padLeft + c2 * (cell + gap);
    var yBase = padTop + (cell + gap) * 2;
    var barH = 9 * (cell + gap);

    // normalize
    var n = 0.0;
    if (selectedAttr === "len") n = Math.max(0, Math.min(1, v / 192));
    else if (selectedAttr === "sta") n = (Math.max(-5, Math.min(5, v)) + 5) / 10.0;
    else if (selectedAttr === "pit") n = (Math.max(-64, Math.min(63, v)) + 64) / 127.0;
    else if (selectedAttr === "amt" || selectedAttr === "vel") n = (Math.max(-127, Math.min(127, v)) + 127) / 254.0;
    else n = 0.5;

    var bh = Math.max(2, Math.round(n * barH));
    var y2 = yBase + (barH - bh);
    g.set_source_rgba(colors.label);
    g.rectangle(x2 + 4, y2, cell - 8, bh);
    g.fill();
  }
}

function _paintStep(g) {
  var snap = _getActivePageSnapshot();
  var d = snap.d;
  var base = ["banks", snap.bank, "pages", snap.page, "tracks", selectedTrack, "steps", selectedStep].join("::");

  var active = !!d.get(base + "::active");
  var skip = !!d.get(base + "::skip");
  var pit = Number(d.get(base + "::pit_offset") || 0);
  var vel = Number(d.get(base + "::vel_offset") || 0);
  var len = Number(d.get(base + "::len") || 12);
  var sta = Number(d.get(base + "::sta_offset") || 0);
  var strum = Number(d.get(base + "::strum") || 0);

  g.set_source_rgba(colors.label);
  g.move_to(8, 36);
  g.show_text("TRK " + selectedTrack + " STEP " + selectedStep);
  g.move_to(8, 56);
  g.show_text("ACTIVE " + active + "  SKIP " + skip);
  g.move_to(8, 76);
  g.show_text("PIT " + pit + "  VEL " + vel);
  g.move_to(8, 96);
  g.show_text("LEN " + len + "  STA " + sta + "  STR " + strum);
}

function onclick(x, y, but, cmd, shift, caps, opt, ctrl) {
  // Map to cell
  var gx = x - padLeft;
  var gy = y - padTop;
  if (gx < 0 || gy < 0) return;

  var col = Math.floor(gx / (cell + gap));
  var row = Math.floor(gy / (cell + gap));
  if (row < 0 || row >= rows || col < 0 || col >= cols) return;

  // Inside rect?
  var rx = gx % (cell + gap);
  var ry = gy % (cell + gap);
  if (rx > cell || ry > cell) return;

  if (mode === "grid") {
    outlet(0, "set_active_page", row, col);
  } else if (mode === "page") {
    // Shift-click -> skip toggle; otherwise active toggle
    if (shift) outlet(0, "step_skip_toggle", row, col);
    else outlet(0, "step_toggle", row, col);
  } else if (mode === "track") {
    // Row 0 toggles; clicking elsewhere selects step for STEP view quick access
    if (row === 0) {
      if (shift) outlet(0, "step_skip_toggle", selectedTrack, col);
      else outlet(0, "step_toggle", selectedTrack, col);
    } else {
      selectedStep = col;
      outlet(0, "bang");
    }
  } else if (mode === "step") {
    // Click in top row area selects step by x position
    selectedStep = col;
    outlet(0, "bang");
  }
  mgraphics.redraw();
}

function bang() {
  mgraphics.redraw();
}

function set_mode(m) {
  mode = String(m || "page").toLowerCase();
  mgraphics.redraw();
}

function select_track(t) {
  selectedTrack = _clampInt(t, 0, 9);
  mgraphics.redraw();
}

function select_step(s) {
  selectedStep = _clampInt(s, 0, 15);
  mgraphics.redraw();
}

function set_attr(a) {
  selectedAttr = String(a || "vel").toLowerCase();
  mgraphics.redraw();
}
