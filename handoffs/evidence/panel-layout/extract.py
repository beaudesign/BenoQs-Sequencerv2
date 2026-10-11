#!/usr/bin/env python3
"""Measures where the Octopus's keys, knobs and LEDs are in reference/plates/web-frontal-01.jpg.

Usage:  python3 extract.py <plate.jpg> <out.json>        (needs numpy, scipy, opencv-python)

What it does, in order (every number below is a measurement, not a choice):
  1. Finds every key (a chrome ball, about 10 px across in the 900 x 700 photograph) by matching a template cut from
     one matrix key. Hex-nut knobs are found the same way with a template cut from one MIX knob.
  2. Fits the 160 matrix keys to a lattice with an affine transform. This also measures the photograph's roll
     (about 0.4 degrees) and the matrix's pitch ratio (a row is 1.495 column pitches).
  3. Rectifies every point with that transform, so the matrix is axis-aligned and one unit is one matrix column pitch.
     X runs right from matrix column 1, Y runs down from matrix row 9.
  4. Looks for the LED beside each key: a small dot within 14 px of the ball, and says on which side (n, e, s, w).
The plate's own outline is the cream region's bounding box. Nothing here reads a label: the printed names are too small
in this photograph (about 4 px high) and are not read; assignment.py says which keys are named and how it knows.
"""
import json, sys
import numpy as np, cv2

src, out = sys.argv[1], sys.argv[2]
im = cv2.imread(src)
g = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY).astype(np.float32)
H, W = g.shape


def match(template, thr, r_nms):
    res = cv2.matchTemplate(g, template, cv2.TM_CCOEFF_NORMED)
    th, tw = template.shape
    peaks = []
    work = res.copy()
    while True:
        mv = float(work.max())
        if mv < thr:
            break
        y, x = np.unravel_index(work.argmax(), work.shape)
        peaks.append((int(x) + tw // 2, int(y) + th // 2, mv))
        work[max(0, y - r_nms):y + r_nms + 1, max(0, x - r_nms):x + r_nms + 1] = -1
    return peaks


# 1. Template: the matrix key at row 5, column 8 (found by eye once, then fixed here).
KEY0 = (284, 404)
r = 7
t_key = g[KEY0[1] - r:KEY0[1] + r + 1, KEY0[0] - r:KEY0[0] + r + 1]
keys = [(x, y, s) for x, y, s in match(t_key, 0.6, 6) if 60 < y < 700]
# The hex-nut knobs: template cut from the first MIX knob.
KNOB0 = (104, 307)
rk = 11
t_knob = g[KNOB0[1] - rk:KNOB0[1] + rk + 1, KNOB0[0] - rk:KNOB0[0] + rk + 1]
knobs = [(x, y, s) for x, y, s in match(t_knob, 0.65, 9) if 230 < y < 600 and 60 < x < 860]

# 2. The matrix lattice. Columns start near x = 169.7, a column is 16.4 px, a row is 24.6 px.
pts = [(x, y) for x, y, _ in keys if 160 <= x <= 425 and 295 <= y <= 540]
idx = [(round((x - 168.7) / 16.45), round((527.0 - y) / 24.5), x, y) for x, y in pts]
assert len(idx) == 160 and len({(c, r_) for c, r_, _, _ in idx}) == 160, f"matrix: {len(idx)} keys fitted"
A = np.array([[1, c, r_] for c, r_, _, _ in idx], float)
bx = np.linalg.lstsq(A, np.array([x for *_, x, y in idx]), rcond=None)[0]
by = np.linalg.lstsq(A, np.array([y for *_, x, y in idx]), rcond=None)[0]
resid = np.hypot(A @ bx - np.array([x for *_, x, y in idx]), A @ by - np.array([y for *_, x, y in idx]))
M = np.array([[bx[1], bx[2]], [by[1], by[2]]])  # photo px per (column, row-up)
Minv = np.linalg.inv(M)
origin = np.array([bx[0], by[0]])
col_pitch = float(np.hypot(*M[:, 0]))
row_pitch = float(np.hypot(*M[:, 1]))
ratio = row_pitch / col_pitch


def rectify(x, y):
    c, r_ = Minv @ (np.array([x, y]) - origin)
    return float(c), float((9 - r_) * ratio)  # X right, Y down, in matrix column pitches


# 3. LEDs: dark or coloured dots outside the balls.
bg = np.array([245, 244, 244], float)
diff = np.abs(im.astype(float) - bg).max(axis=2)
mask = (diff > 45).astype(np.uint8)
all_balls = [(x, y) for x, y, _ in keys] + [(x, y) for x, y, _ in knobs]
for x, y in all_balls:
    cv2.circle(mask, (int(x), int(y)), 6, 0, -1)
n, lab, stats, cent = cv2.connectedComponentsWithStats(mask)
dots = []
for i in range(1, n):
    a = stats[i, cv2.CC_STAT_AREA]
    w, h = stats[i, cv2.CC_STAT_WIDTH], stats[i, cv2.CC_STAT_HEIGHT]
    if 5 <= a <= 40 and 2 <= w <= 7 and 2 <= h <= 7:
        dots.append((float(cent[i][0]), float(cent[i][1])))


def led_for(x, y):
    best = None
    for dx_, dy_ in dots:
        d = np.hypot(dx_ - x, dy_ - y)
        if 6.8 <= d <= 11 and (best is None or d < best[0]):
            best = (d, dx_ - x, dy_ - y)
    if best is None:
        return None
    _, dx_, dy_ = best
    side = ("e" if dx_ > 0 else "w") if abs(dx_) > abs(dy_) else ("s" if dy_ > 0 else "n")
    return {"side": side, "dx_px": round(dx_, 1), "dy_px": round(dy_, 1)}


rows = []
for x, y, s in keys:
    X, Y = rectify(x, y)
    rows.append({"kind": "key", "px": [round(x, 1), round(y, 1)], "u": [round(X, 3), round(Y, 3)], "score": round(s, 2), "led": led_for(x, y)})
for x, y, s in knobs:
    X, Y = rectify(x, y)
    rows.append({"kind": "knob", "px": [round(x, 1), round(y, 1)], "u": [round(X, 3), round(Y, 3)], "score": round(s, 2), "led": None})

# The plate: the cream region's bounding box.
cream = ((g > 215) * 1).astype(np.uint8)
cream = cv2.morphologyEx(cream, cv2.MORPH_CLOSE, np.ones((7, 7), np.uint8))
nlab, lab2, st, _ = cv2.connectedComponentsWithStats(cream)
big = 1 + int(np.argmax(st[1:, cv2.CC_STAT_AREA]))
x0, y0, w0, h0 = st[big, cv2.CC_STAT_LEFT], st[big, cv2.CC_STAT_TOP], st[big, cv2.CC_STAT_WIDTH], st[big, cv2.CC_STAT_HEIGHT]
corners = [rectify(x0, y0), rectify(x0 + w0, y0), rectify(x0 + w0, y0 + h0), rectify(x0, y0 + h0)]

json.dump({
    "source": src,
    "photo": [W, H],
    "matrix": {"origin_px": [round(float(origin[0]), 2), round(float(origin[1]), 2)], "col_px": [round(float(M[0, 0]), 3), round(float(M[1, 0]), 3)],
               "row_px": [round(float(M[0, 1]), 3), round(float(M[1, 1]), 3)], "col_pitch_px": round(col_pitch, 3), "row_pitch_px": round(row_pitch, 3),
               "row_over_col": round(ratio, 4), "fit_residual_px": {"max": round(float(resid.max()), 2), "mean": round(float(resid.mean()), 2)},
               "roll_degrees": round(float(np.degrees(np.arctan2(M[1, 0], M[0, 0]))), 2)},
    "plate_px": [int(x0), int(y0), int(x0 + w0), int(y0 + h0)],
    "plate_u": [[round(a, 3), round(b, 3)] for a, b in corners],
    "unit": "one matrix column pitch; X right from matrix column 1, Y down from matrix row 9",
    "found": rows,
}, open(out, "w"), indent=1)
print(f"{len(keys)} keys, {len(knobs)} knobs, {len(dots)} led-sized dots; matrix residual max {resid.max():.2f} px; row/col {ratio:.4f}; plate {x0},{y0} {w0}x{h0}", file=sys.stderr)
