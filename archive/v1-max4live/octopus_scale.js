autowatch = 1;

// Pure-ish helpers for force-to-scale quantization.

function _clampMidi(v) {
  v = Math.round(Number(v));
  if (isNaN(v)) return 0;
  if (v < 0) return 0;
  if (v > 127) return 127;
  return v;
}

function _normPc(pc) {
  pc = Math.round(Number(pc));
  if (isNaN(pc)) pc = 0;
  pc = pc % 12;
  if (pc < 0) pc += 12;
  return pc;
}

function buildScalePitchClasses(rootMidiOrPc, intervals) {
  var rootPc = _normPc(rootMidiOrPc);
  var pcs = {};
  if (!Array.isArray(intervals) || !intervals.length) intervals = [0, 2, 4, 5, 7, 9, 11];
  for (var i = 0; i < intervals.length; i++) {
    pcs[_normPc(rootPc + intervals[i])] = 1;
  }
  pcs[rootPc] = 1;
  return pcs; // object map pc->1
}

/**
 * Quantize pitch to nearest pitch class in scale.
 * Tie-break: prefer downward motion (stable for live use).
 */
function quantizeToScale(pitch, scalePcs) {
  pitch = _clampMidi(pitch);
  var pc = pitch % 12;
  if (scalePcs && scalePcs[pc]) return pitch;

  var best = pitch;
  var bestDist = 999;
  // search +/- up to 11 semitones
  for (var d = 1; d <= 11; d++) {
    var down = pitch - d;
    if (down >= 0 && scalePcs[_normPc(down)]) {
      best = down;
      bestDist = d;
      break; // downward tie-break wins
    }
    var up = pitch + d;
    if (up <= 127 && scalePcs[_normPc(up)]) {
      if (d < bestDist) {
        best = up;
        bestDist = d;
      }
      // don't break: we still want to see if a down match at same d exists, but we checked down first.
      break;
    }
  }
  return _clampMidi(best);
}

