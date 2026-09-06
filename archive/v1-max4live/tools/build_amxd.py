#!/usr/bin/env python3
"""
Build Octopus.amxd by replacing the embedded Max patch (ptch chunk) inside
Ableton's stock "Max MIDI Effect.amxd" container.

Requires: Ableton Live installed at the default path (edit LIVE_APP if needed).
Output: ../Octopus.amxd next to this repo's .js files (same folder as octopus_ui_main.maxpat).
"""

from __future__ import annotations

import json
import shutil
import struct
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
LIVE_APP = Path(
    "/Applications/Ableton Live 12 Standard.app/Contents/App-Resources/Misc/Max Devices/Max MIDI Effect.amxd"
)
TEMPLATE_MAXPAT = REPO_ROOT / "octopus_ui_main.maxpat"
OUT_AMXD = REPO_ROOT / "Octopus.amxd"


def read_amxd_chunks(path: Path) -> tuple[bytes, bytes]:
    """Return (prefix_before_ptch_json, suffix_after_ptch_json) for rewriting ptch."""
    data = path.read_bytes()
    if data[:4] != b"ampf":
        raise ValueError("Not an amxd/ampf file")
    # After ampf + u32 version: mmmm + meta + u32 size + meta payload
    off = 8
    if data[off : off + 4] != b"mmmm":
        raise ValueError("Expected mmmm chunk")
    off += 4
    if data[off : off + 4] != b"meta":
        raise ValueError("Expected meta chunk name")
    off += 4
    meta_size = struct.unpack_from("<I", data, off)[0]
    off += 4
    off += meta_size
    if data[off : off + 4] != b"ptch":
        raise ValueError("Expected ptch chunk")
    off += 4
    ptch_size = struct.unpack_from("<I", data, off)[0]
    off += 4
    ptch_start = off
    ptch_end = ptch_start + ptch_size
    prefix = data[:ptch_start]
    suffix = data[ptch_end:]
    return prefix, suffix


def load_json_maxpat(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def merge_m4l_shell(ours: dict, template_path: Path) -> dict:
    """Keep our patcher as primary; ensure midiin/midiout + M4L-ish keys exist."""
    tdata = template_path.read_bytes()
    off = 24
    psz = struct.unpack_from("<I", tdata, off + 4)[0]
    chunk = tdata[off + 8 : off + 8 + psz].rstrip(b"\x00")
    template = json.loads(chunk.decode("utf-8"))
    t_patch = template["patcher"]

    p = ours["patcher"]
    # Carry over flags Ableton's empty device uses
    if "modernui" not in p:
        p["modernui"] = t_patch.get("modernui", 1)
    if "classnamespace" not in p:
        p["classnamespace"] = t_patch.get("classnamespace", "box")

    # Append template I/O if missing
    have_midiin = any(
        b.get("box", b).get("text") == "midiin" for b in p.get("boxes", [])
    )
    have_midiout = any(
        b.get("box", b).get("text") == "midiout" for b in p.get("boxes", [])
    )
    if not have_midiin:
        for b in t_patch.get("boxes", []):
            box = b.get("box", b)
            if box.get("text") == "midiin":
                p.setdefault("boxes", []).append(b)
                break
    if not have_midiout:
        for b in t_patch.get("boxes", []):
            box = b.get("box", b)
            if box.get("text") == "midiout":
                p.setdefault("boxes", []).append(b)
                break
    # Ableton template passthrough (midiin -> midiout); safe default for a MIDI effect shell
    thru = {"patchline": {"source": ["obj-1", 0], "destination": ["obj-2", 0]}}
    lines = p.setdefault("lines", [])
    if thru not in lines:
        lines.append(thru)

    return ours


def patcher_to_ptch_bytes(patcher_dict: dict) -> bytes:
    """Serialize patcher JSON the way Max expects inside ptch (trailing NUL)."""
    wrapper = {"patcher": patcher_dict}
    # Tabs match typical Max exports; separators keep floats as 800.0 style.
    text = json.dumps(wrapper, indent="\t", ensure_ascii=False)
    # Max often terminates the JSON with a NUL inside the chunk
    return text.encode("utf-8") + b"\x00"


def main() -> int:
    if not LIVE_APP.is_file():
        print(f"Missing Ableton template: {LIVE_APP}", file=sys.stderr)
        print("Edit LIVE_APP in tools/build_amxd.py to your Live .app path.", file=sys.stderr)
        return 1
    if not TEMPLATE_MAXPAT.is_file():
        print(f"Missing {TEMPLATE_MAXPAT}", file=sys.stderr)
        return 1

    ours = load_json_maxpat(TEMPLATE_MAXPAT)
    merged = merge_m4l_shell(ours, LIVE_APP)
    ptch = patcher_to_ptch_bytes(merged["patcher"])

    prefix, suffix = read_amxd_chunks(LIVE_APP)
    # prefix ends at start of JSON; we must insert correct ptch size before JSON
    # prefix = ... + b"ptch" + old_size — need to rebuild tail of prefix with new size
    if prefix[-8:-4] != b"ptch":
        raise ValueError("Prefix parsing mismatch: expected ptch before JSON")
    head = prefix[:-4]  # drop old 4-byte size (keep through "ptch")
    new_prefix = head + struct.pack("<I", len(ptch))

    out = new_prefix + ptch + suffix
    shutil.copyfile(LIVE_APP, OUT_AMXD.with_suffix(".amxd.bak-template"))
    OUT_AMXD.write_bytes(out)
    print(f"Wrote {OUT_AMXD} ({len(out)} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
