#!/usr/bin/env python3
"""Turns measured.json (extract.py) into apps/web/layout/panel.layout.json.

Usage:  python3 assign.py <repo root>        (writes apps/web/layout/panel.layout.json; needs only the standard library)

extract.py measured where things are. This file says what each measured thing is. Where the photograph settles that
(a legible label, a glyph, a block the manual names) the entry's basis says so; where only the key's place in a group
settles it, the basis is "order" and the entry is provisional; where nothing does, the key is drawn and left unnamed
("hw.*"). Controls the photograph shows no place for are listed under "unplaced", with the reason, and are not drawn.
Positions are in matrix column pitches from the plate's top-left corner; a pitch is a key plus a gap, so no length is
written in the layout (apps/web/src/layout.ts turns it into a length with the tokens).
"""
import json, math, sys

root = sys.argv[1]
m = json.load(open(f"{root}/handoffs/evidence/panel-layout/measured.json"))
controls = {c["id"]: c for c in json.load(open(f"{root}/contracts/controls.json"))["controls"]}
tokens = json.load(open(f"{root}/contracts/design.tokens.json"))

# ---- the frame -------------------------------------------------------------------------------------------------
M = m["matrix"]
ox, oy = M["origin_px"]
colx, coly = M["col_px"]
rowx, rowy = M["row_px"]
ratio = M["row_over_col"]
det = colx * rowy - rowx * coly


def rectify(px, py):
    dx, dy = px - ox, py - oy
    c = (dx * rowy - rowx * dy) / det
    r = (colx * dy - coly * dx) / det
    return c, (9 - r) * ratio


corners = m["plate_u"]
X0 = min(c[0] for c in corners)
Y0 = min(c[1] for c in corners)
W = max(c[0] for c in corners) - X0
H = max(c[1] for c in corners) - Y0


def at(px, py):
    X, Y = rectify(px, py)
    return round(X - X0, 2), round(Y - Y0, 2)


found = {(round(f["px"][0]), round(f["px"][1])): f for f in m["found"]}


def nearest(kind, px, py, tol=3):
    best = None
    for (x, y), f in found.items():
        if f["kind"] != kind:
            continue
        d = math.hypot(x - px, y - py)
        if d <= tol and (best is None or d < best[0]):
            best = (d, f)
    if best is None:
        raise SystemExit(f"no {kind} within {tol}px of {(px, py)}")
    return best[1]


def led_side(f, default="n"):
    return (f.get("led") or {}).get("side", default)


controls_out, keys_out, legends_out, unplaced = [], [], [], []
used = set()


def put(cid, f, basis, text=None, text_basis=None, led=None):
    if cid in used:
        raise SystemExit(f"{cid} placed twice")
    used.add(cid)
    x, y = at(*f["px"])
    e = {"id": cid, "x": x, "y": y}
    if controls[cid].get("led"):
        e["led"] = led or led_side(f)
    if text:
        e["text"] = text
        if text_basis:
            e["text_basis"] = text_basis
    e["basis"] = basis
    controls_out.append(e)


def hw(hid, f, text=None, basis="photo", led=None):
    x, y = at(*f["px"])
    e = {"id": hid, "x": x, "y": y, "led": led or led_side(f)}
    if text:
        e["text"] = text
    e["basis"] = basis
    keys_out.append(e)


# ---- the knobs -------------------------------------------------------------------------------------------------
knobs = sorted([f for f in m["found"] if f["kind"] == "knob"], key=lambda f: (f["px"][0] > 300, f["px"][1]))
mix = [f for f in knobs if f["px"][0] < 200]
edit = [f for f in knobs if 400 < f["px"][0] < 600]
main = [f for f in knobs if f["px"][0] > 700]
assert (len(mix), len(edit), len(main)) == (10, 10, 1), (len(mix), len(edit), len(main))
mix.sort(key=lambda f: -f["px"][1])  # bottom (row 0) first
for r, f in enumerate(mix):
    put(f"mix.enc.r{r}", f, "photo")
EDIT_ORDER = ["vel", "pit", "len", "sta", "pos", "dir", "amt", "grv", "mcc", "mch"]  # top to bottom, as the photograph's labels read
for name, f in zip(EDIT_ORDER, sorted(edit, key=lambda f: f["px"][1])):
    put(f"edit.enc.{name}", f, "photo", text=name.upper())
put("main.enc", main[0], "photo")

# ---- the columns beside the matrix -----------------------------------------------------------------------------
keys = [f for f in m["found"] if f["kind"] == "key"]
sel = sorted([f for f in keys if 130 < f["px"][0] < 150 and 300 < f["px"][1] < 535], key=lambda f: -f["px"][1])
mut = sorted([f for f in keys if 440 < f["px"][0] < 452 and 300 < f["px"][1] < 535], key=lambda f: f["px"][1])
assert len(sel) == 10 and len(mut) == 10, (len(sel), len(mut))
for r, f in enumerate(sel):
    put(f"selector.r{r}", f, "photo")
MUT_ORDER = ["tgl", "sol", "clr", "rnd", "flt", "zom", "rmx", "cpy", "pst", "eff"]  # top to bottom; the order is a guess, flagged
MUT_TEXT = {"cpy": "Copy", "pst": "Paste"}
for name, f in zip(MUT_ORDER, mut):
    put(f"mutator.{name}", f, "order", text=MUT_TEXT.get(name, name.upper()))

# ---- the row under the matrix ----------------------------------------------------------------------------------
bottom = sorted([f for f in keys if 548 < f["px"][1] < 560 and f["px"][0] < 600], key=lambda f: f["px"][0])
assert len(bottom) == 16, len(bottom)
(b_mix, b_sel, *target, b_mut, b_edit, b_esc) = bottom
assert len(target) == 11
put("mix.master", b_mix, "photo", text="MIX", led="n")
hw("hw.sel", b_sel, text="SEL")
for text, f in zip(["ATR", "VOL", "PAN", "MOD", "EXP", "MAP", "1", "2", "3", "4", "5"], target):
    hw(f"hw.target.{text.lower()}", f, text=text)
put("mutator.mute", b_mut, "photo", text="Mute")
put("mode.edit", b_edit, "position", text="EDIT")
put("keys.esc", b_esc, "photo", text="ESC")

# ---- the circle ------------------------------------------------------------------------------------------------
# Photograph coordinates of each key in the circle. The list is what the eye settled on in the photograph.
CLUSTER = {
    # the number keys on the top-left arc, 1 to 10 (the manual says Roman numerals, Q33; the photograph's are too small to settle it)
    (530, 416): ("hw", "hw.num.1", "I"), (534, 394): ("hw", "hw.num.2", "II"), (540, 372): ("hw", "hw.num.3", "III"), (550, 352): ("hw", "hw.num.4", "IV"),
    (565, 334): ("hw", "hw.num.5", "V"), (583, 320): ("hw", "hw.num.6", "VI"), (603, 310): ("hw", "hw.num.7", "VII"), (624, 303): ("hw", "hw.num.8", "VIII"),
    (647, 301): ("hw", "hw.num.9", "IX"), (670, 303): ("hw", "hw.num.10", "X"),
    # the outer ring
    (711, 319): ("c", "circle.tempo", "Tempo", "photo"), (730, 334): ("c", "keys.program", "Program", "photo"), (744, 352): ("c", "circle.select", "Select", "photo"),
    (581, 513): ("c", "circle.chain", "X", "glyph"), (789, 342): ("c", "circle.clock200", "200", "order"),
    (692, 309): ("hw", "hw.ring.1", None), (754, 371): ("hw", "hw.ring.2", None), (761, 393): ("hw", "hw.ring.3", None), (764, 416): ("hw", "hw.ring.4", None),
    (762, 439): ("hw", "hw.ring.5", None), (755, 460): ("hw", "hw.ring.6", None), (744, 481): ("hw", "hw.ring.7", None), (729, 499): ("hw", "hw.ring.8", None),
    (712, 512): ("hw", "hw.ring.9", None), (565, 498): ("hw", "hw.ring.10", None), (550, 482): ("hw", "hw.ring.11", None), (539, 461): ("hw", "hw.ring.12", None),
    (532, 439): ("hw", "hw.ring.13", None),
    # transport, the bottom of the outer ring
    (691, 524): ("c", "transport.play", "Play", "glyph"), (670, 530): ("c", "transport.pause", "Pause", "glyph"), (647, 534): ("c", "transport.stop", "Stop", "glyph"),
    (624, 530): ("c", "transport.rec", "Record", "glyph"), (603, 524): ("hw", "hw.transport.left", None),
    # the chord arm, top to bottom
    (807, 398): ("c", "chord.b1", "1", "order"), (807, 429): ("c", "chord.b2", "2", "order"), (801, 458): ("c", "chord.b3", "3", "order"), (790, 488): ("c", "chord.b4", "4", "order"),
    (773, 514): ("c", "chord.b5", "5", "order"), (752, 535): ("c", "chord.b6", "6", "order"), (727, 553): ("c", "chord.b7", "7", "order"),
    # the note ring: C at the bottom left, round the top, to the C at the bottom right
    (601, 462): ("c", "circle.note.k1", "C", "photo"), (588, 441): ("c", "circle.note.k2", "C#", "photo"), (583, 416): ("c", "circle.note.k3", "D", "photo"),
    (588, 392): ("c", "circle.note.k4", "D#", "photo"), (601, 370): ("c", "circle.note.k5", "E", "photo"), (622, 357): ("c", "circle.note.k6", "F", "photo"),
    (647, 352): ("c", "circle.note.k7", "F#", "photo"), (672, 357): ("c", "circle.note.k8", "G", "photo"), (694, 370): ("c", "circle.note.k9", "G#", "photo"),
    (706, 391): ("c", "circle.note.k10", "A", "photo"), (711, 416): ("c", "circle.note.k11", "A#", "photo"), (706, 440): ("c", "circle.note.k12", "B", "photo"),
    (693, 462): ("c", "circle.upper_c", "C", "photo"),
    # the scale keys under the note ring
    (622, 477): ("c", "circle.scale_mod", "MOD", "photo"), (647, 481): ("c", "circle.scale_sel", "SEL", "photo"), (672, 477): ("c", "circle.scale_cad", "CAD", "photo"),
    # the MODE block
    (665, 383): ("c", "mode.grid", "GRID", "photo"), (645, 400): ("c", "mode.page", "PAGE", "photo"), (629, 415): ("c", "mode.track", "TRACK", "photo"),
    (678, 418): ("c", "mode.step", "STEP", "photo"), (667, 437): ("c", "mode.play", "PLAY", "photo"), (644, 444): ("hw", "hw.mode.map", "MAP"),
}
for (px, py), spec in CLUSTER.items():
    f = nearest("key", px, py)
    if spec[0] == "hw":
        hw(spec[1], f, text=spec[2], basis="photo" if spec[2] else "photo")
    else:
        _, cid, text, basis = spec
        if basis == "glyph" or basis == "photo" and cid.startswith("circle.note") or cid in ("circle.tempo",):
            pass
        notes = cid.startswith("circle.note") or cid == "circle.upper_c"
        put(cid, f, "photo" if basis in ("photo", "glyph") else basis, text=text, text_basis="photo" if notes else None)

# ---- what the photograph shows no place for ---------------------------------------------------------------------
for cid, why in [
    ("circle.low_c", "The note ring in the photograph has 13 keys (C, C#, D, D#, E, F, F#, G, G#, A, A#, B, C), and the inventory has 14 (twelve notes, an upper C and a low C). Q33."),
    ("circle.chr", "The 'Chr.' key is not among the keys the photograph lets be read; the outer ring has 13 keys (hw.ring.*) whose names are too small to read."),
    ("circle.follow", "The Follow key is not among the keys the photograph lets be read (hw.ring.*)."),
    ("mutator.aln", "The mutator column has ten keys and a MUT key under it; the inventory has twelve mutators, and the photograph does not say where the twelfth is."),
    ("circle.numeric", "The 'numeric field' is a display, not a key, and the photograph shows no numeric display."),
]:
    assert cid in controls and cid not in used, cid
    unplaced.append({"id": cid, "why": why})

placed = {c["id"] for c in controls_out}
missing = [cid for cid, c in controls.items() if c["zone"] != "matrix" and cid not in placed and cid not in {u["id"] for u in unplaced}]
assert not missing, f"controls with neither a place nor a reason: {missing}"

# ---- the printed legends ---------------------------------------------------------------------------------------
def legend(text, px, py, kind, anchor="middle"):
    x, y = at(px, py)
    legends_out.append({"text": text, "x": x, "y": y, "kind": kind, "anchor": anchor})


for text, px, py in [("MIX", 108, 570), ("SEL", 142, 570), ("MUT", 447, 570), ("EDIT", 515, 570), ("CHORD", 753, 566), ("MODE", 627, 381), ("SCALE", 649, 462), ("TRANSPORT", 647, 505)]:
    legend(text, px, py, "section")
legend("MIX TARGET", 285, 543, "caption")
for r in range(10):
    legend(str(r), 124, 527.15 - r * 24.573, "row")
legend("\U0001d11e", 78, 357, "clef")  # treble, at the top of the track column
legend("\U0001d122", 78, 458, "clef")  # bass, at the bottom

# ---- the file --------------------------------------------------------------------------------------------------
tl = at(ox, oy)
mx, my = at(ox, oy - 9 * -M["row_px"][1] if False else oy + 9 * M["row_px"][1])  # matrix row 9, column 1: the top left key
out = {
    "schema": "panel.layout/2",
    "status": "provisional",
    "note": "Where the panel puts every control the inventory holds, taken from reference/plates/web-frontal-01.jpg by handoffs/evidence/panel-layout/extract.py "
            "(the measurement) and assign.py (what each measured key is). Positions are in matrix column pitches from the plate's top left corner: one pitch is "
            "the key size plus the gap, from the tokens, so this file holds no length. A matrix row is row_pitch pitches apart. 'basis' says how an entry's "
            "identity is known: lattice or photo (read from the plate), glyph (read from its printed symbol), position (from where it sits among named "
            "blocks), order (from its place in a group whose order the photograph does not settle: provisional). 'hw' keys are on the plate and in no "
            "inventory (controls.json); they are drawn and do nothing. 'unplaced' controls have no place on the plate and are not drawn.",
    "source": {"photograph": "reference/plates/web-frontal-01.jpg", "measurement": "handoffs/evidence/panel-layout/measured.json", "roll_degrees": M["roll_degrees"]},
    "size": {"key": "space.4", "gap": "space.3", "margin": "space.4", "dot": "space.2", "knob": "space.5"},
    "knob": {"ball": 0.62},
    "led": {"distance": 0.5},
    "plate": {"w": round(W, 2), "h": round(H, 2)},
    "matrix": {"x": mx, "y": my, "row_pitch": round(ratio, 4)},
    "controls": controls_out,
    "keys": keys_out,
    "legends": legends_out,
    "unplaced": unplaced,
}
json.dump(out, open(f"{root}/apps/web/layout/panel.layout.json", "w"), indent=1, ensure_ascii=False)
open(f"{root}/apps/web/layout/panel.layout.json", "a").write("\n")
from collections import Counter
print(f"plate {W:.2f} x {H:.2f} pitches; matrix at {mx},{my}; {len(controls_out)} controls, {len(keys_out)} hw keys, {len(legends_out)} legends, {len(unplaced)} unplaced", file=sys.stderr)
print(Counter(c["basis"] for c in controls_out), file=sys.stderr)
