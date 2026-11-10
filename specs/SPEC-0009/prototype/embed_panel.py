#!/usr/bin/env python3
"""Writes the panel's measured layout into shell.html, between the PANEL markers.

The prototype draws a simplified stand-in of the panel from apps/web/layout/panel.layout.json, so the
instrument in it sits where the real one does. Run from the repository root after the layout changes:

    python3 specs/SPEC-0009/prototype/embed_panel.py
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
LAYOUT = ROOT / "apps/web/layout/panel.layout.json"
PAGE = Path(__file__).with_name("shell.html")

layout = json.loads(LAYOUT.read_text())


def entry(c):
    # x, y in column pitches; knob (no LED key in the file); LED side; printed name; which side the name sits.
    return [c["x"], c["y"], 0 if "led" in c else 1, c.get("led", ""), c.get("text", ""), c.get("label", "")]


data = {
    "plate": [layout["plate"]["w"], layout["plate"]["h"]],
    "matrix": [layout["matrix"]["x"], layout["matrix"]["y"], layout["matrix"]["row_pitch"]],
    "ball": layout["knob"]["ball"],
    "led": layout["led"]["distance"],
    "controls": [entry(c) for c in layout["controls"] + layout["keys"]],
    "legends": [[l["text"], l["x"], l["y"], l["kind"]] for l in layout["legends"]],
}

# Escaped, so the two clef glyphs survive a server that sends no charset.
blob = json.dumps(data, separators=(",", ":"))
page = PAGE.read_text()
page, n = re.subn(r"/\*PANEL\*/.*?/\*END\*/", lambda _: "/*PANEL*/" + blob + "/*END*/", page, count=1, flags=re.S)
if n != 1:
    raise SystemExit("shell.html has no PANEL marker")
PAGE.write_text(page)
print(f"embedded {len(data['controls'])} controls and {len(data['legends'])} legends ({len(blob)} bytes)")
