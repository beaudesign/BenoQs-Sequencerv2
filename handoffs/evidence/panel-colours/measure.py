"""Measures the colours of the Octopus in reference/plates/web-frontal-01.jpg, as numbers.

Usage:  python3 measure.py <plate.jpg> <../panel-layout/measured.json> <out.json>     (needs numpy, opencv-python)

Positions come from handoffs/evidence/panel-layout/measured.json (keys, knobs, LED dots, plate outline), so this
reads colour at the places that file already says things are. Nothing here is a judgement of taste: every output is a
median or a percentile of the pixels at those places, in sRGB, with the count of pixels behind it.

Caveat, stated in the output: the photograph is a retailer's product shot with no colour chart and an unknown white
balance (reference/plates/NOTES.md). The numbers say what the photograph shows, not what the hardware is.
"""
import json, sys
import cv2
import numpy as np

img_path, meas_path, out_path = sys.argv[1:4]
bgr = cv2.imread(img_path)
assert bgr is not None, img_path
rgb = cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB)
H, W = rgb.shape[:2]
m = json.load(open(meas_path))
found = m["found"]


def hexs(c):
    return "#%02x%02x%02x" % tuple(int(round(v)) for v in c)


def lab(c):
    p = np.uint8([[list(map(int, c))]])
    L, a, b = (float(v) for v in cv2.cvtColor(p, cv2.COLOR_RGB2LAB)[0, 0])
    return [round(L * 100 / 255, 1), round(a - 128, 1), round(b - 128, 1)]


def chroma(c):
    _, a, b = lab(c)
    return round(float(np.hypot(a, b)), 1)


def disk(cx, cy, r):
    ys, xs = np.ogrid[:H, :W]
    return (xs - cx) ** 2 + (ys - cy) ** 2 <= r * r


def summarise(px):
    px = np.asarray(px, dtype=float).reshape(-1, 3)
    med = np.median(px, axis=0)
    return {"hex": hexs(med), "lab": lab(med), "chroma": chroma(med), "pixels": int(len(px)),
            "p10": hexs(np.percentile(px, 10, axis=0)), "p90": hexs(np.percentile(px, 90, axis=0))}


out = {"source": m["source"], "caveat": "retailer product photograph, no colour chart, unknown white balance; the numbers are what the photograph shows"}

# --- the plate: the cream region inside the outline, away from every key, knob and dot
x0, y0, w0, h0 = m["plate_px"]
inside = np.zeros((H, W), bool)
inside[y0 + 2:y0 + h0 - 2, x0 + 2:x0 + w0 - 2] = True
busy = np.zeros((H, W), bool)
for f in found:
    cx, cy = f["px"]
    busy |= disk(cx, cy, 14 if f["kind"] == "knob" else 11)
    if f.get("led"):
        busy |= disk(cx + f["led"]["dx_px"], cy + f["led"]["dy_px"], 6)
cream = (rgb.min(axis=2) > 200) & (rgb[..., 0] >= rgb[..., 2])
plate_mask = inside & ~busy & cream
out["plate"] = summarise(rgb[plate_mask])
out["plate"]["note"] = "median of the light, warm pixels inside the outline and clear of every key, knob and LED; p10 and p90 show the lighting across the plate"

# --- the frame: the wood outside the outline (left and right cheeks, and the top and bottom if the photograph has them)
frame_px = []
bands = {"left": (0, max(x0 - 3, 0)), "right": (min(x0 + w0 + 3, W), W)}
for name, (a, c) in bands.items():
    patch = rgb[y0:y0 + h0, a:c]
    if patch.size:
        px = patch.reshape(-1, 3)
        dark = px[px.max(axis=1) < 140]
        out.setdefault("frame_bands", {})[name] = {"width_px": int(c - a), **(summarise(dark) if len(dark) else {"hex": None})}
        if len(dark):
            frame_px.append(dark)
if frame_px:
    out["frame"] = summarise(np.vstack(frame_px))
    out["frame"]["note"] = "median of the dark pixels in the strips left and right of the plate outline (the wenge cheeks)"

# --- the matrix keys: face of the chrome ball
keys = [f for f in found if f["kind"] == "key"]
face = [rgb[disk(f["px"][0], f["px"][1], 3)] for f in keys]
out["key_face"] = summarise(np.vstack(face))
out["key_face"]["note"] = "median over all keys of the central 3 px radius; a chrome ball has a highlight, so p10 to p90 is wide"
out["keys_counted"] = len(keys)

# --- the knobs: hex nut and ball
knobs = [f for f in found if f["kind"] == "knob"]
nut, ball = [], []
for f in knobs:
    cx, cy = f["px"]
    ball.append(rgb[disk(cx, cy, 3)])
    nut.append(rgb[disk(cx, cy, 10) & ~disk(cx, cy, 6)])
out["knob_ball"] = summarise(np.vstack(ball))
out["knob_nut"] = summarise(np.vstack(nut))
out["knobs_counted"] = len(knobs)

# --- the LED dots: unlit ones are the common case; lit ones have a hue
dots = []
for f in keys:
    if f.get("led"):
        cx = f["px"][0] + f["led"]["dx_px"]
        cy = f["px"][1] + f["led"]["dy_px"]
        dots.append(rgb[disk(cx, cy, 1.5)].reshape(-1, 3))
dots = [d for d in dots if len(d)]
meds = np.array([np.median(d, axis=0) for d in dots])
sat = meds.max(axis=1) - meds.min(axis=1)
unlit = meds[sat < 40]
lit = meds[sat >= 40]
out["led_dots"] = {"found": int(len(meds)), "without_hue": int(len(unlit))}
if len(unlit):
    out["led_off"] = summarise(unlit)
    out["led_off"]["note"] = "median of the dots with no hue; the photograph's dots are about 3 px across, so this is a blend with the plate round them"

# --- the printed ink: the darkest pixels on the plate that are not a key, knob or dot
ink_mask = inside & ~busy
dark = rgb[ink_mask]
lum = dark.astype(float) @ np.array([0.2126, 0.7152, 0.0722])
cut = np.percentile(lum, 1.0)
out["ink"] = summarise(dark[lum <= cut])
out["ink"]["note"] = "the darkest 1 percent of plate pixels clear of keys; printed names are about 4 px high so this is a blur of the ink with the plate and is lighter than the ink"

json.dump(out, open(out_path, "w"), indent=1)
for k in ["plate", "frame", "key_face", "knob_ball", "knob_nut", "led_off", "ink"]:
    if k in out:
        print(f"{k:10s} {out[k]['hex']}  lab {out[k]['lab']}  chroma {out[k]['chroma']}  n={out[k]['pixels']}")
print("led dots", out["led_dots"], "keys", out["keys_counted"], "knobs", out["knobs_counted"], "frame bands", out.get("frame_bands"))
