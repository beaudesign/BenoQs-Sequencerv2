# WENGE: every verb an agent needs. See agents/CLAUDE.md "Commands".
#
# `just verify:<zone>` from the docs is not literal: `just` recipe names cannot contain
# a `:`, so zone gates are `verify-<zone>` here.
#
# The gates live in `xtask/` (run as `cargo xtask`, see harness/README.md). `verify`
# runs all of them, writes harness/report/latest.json and exits non-zero if any gate
# failed or any gate in harness/required-gates.txt did not pass. A gate that is not
# built reports `not_implemented`; it is never a silent pass.

# Run every gate. Extra args go to xtask, for example `just verify --base origin/main`.
verify *args:
	cargo xtask verify {{args}}

# One gate. Exit 42 means "not implemented yet".
gate name *args:
	cargo xtask gate {{name}} {{args}}

# Run a pattern headless: `just run examples/hello`. Writes target/octorun/NAME.ndjson and
# NAME.mid and prints the SHA-256 of each. The same pattern gives the same bytes every time.
run pattern *args:
	cargo run -q --release -p octorun -- {{pattern}} {{args}}

# Rewrite the golden hashes in examples/golden/ after an INTENDED change to what the engine
# plays. Review the diff and say why in the commit. `cargo test -p octorun` fails until then.
golden:
	cargo build -q --release -p octorun
	for p in examples/*.pattern; do n=$(basename $p .pattern); ./target/release/octorun $p --golden > examples/golden/$n.sha256; done

# WebAssembly smoke test: builds octoffi for wasm32, plays the example patterns in Node and
# checks the event log against the native golden hashes. Needs `rustup target add
# wasm32-unknown-unknown` and Node 18 or newer.
wasm-smoke:
	cargo build -q --release -p octoffi --target wasm32-unknown-unknown
	node harness/wasm/smoke.mjs

# Record new tests and fixtures as the ratchet floor. Dropping one needs --remove <kind> <id> --adr ADR-NNNN.
baseline *args:
	cargo xtask baseline {{args}}

# Fast check for the sequencer core only (seconds). Not a gate.
verify-octocore:
	cargo test -p octocore

# Per-zone gate verbs below. Owners are listed in xtask/src/gates.rs.

verify-geometry:
	@cargo xtask gate geometry
verify-color:
	@cargo xtask gate color
verify-frames:
	@cargo xtask gate frames
verify-motion:
	@cargo xtask gate motion
verify-timing:
	@cargo xtask gate timing
verify-conformance:
	@cargo xtask gate conformance
verify-acoustics:
	@cargo xtask gate acoustics
verify-a11y:
	@cargo xtask gate a11y
verify-arch:
	@cargo xtask gate arch
verify-slop:
	@cargo xtask gate slop
verify-tokens:
	@cargo xtask gate tokens
verify-regressions:
	@cargo xtask gate regressions
verify-determinism:
	@cargo xtask gate determinism
verify-persistence:
	@cargo xtask gate persistence

# Deterministic offline capture of a registered scene. Not wired yet — no
# renderer exists. See harness/capture/ and docs/07-verification.md §2.
capture scene:
	@echo "capture '{{scene}}': not yet implemented — harness/capture/ has no renderer to drive yet."
	@exit 1

# The shared world model, rendered. Falls back to STATE.md until the real
# report exists.
report:
	#!/usr/bin/env bash
	set -euo pipefail
	if [ -f harness/report/latest.json ]; then
		cat harness/report/latest.json
	else
		echo "harness/report not built yet — current world model is journal/STATE.md"
		echo "---"
		cat journal/STATE.md
	fi

# Grep the Octopus reference manual for a topic (matches reference/manual/INDEX.md
# row labels) and print the matched pages' plain text.
manual topic:
	#!/usr/bin/env bash
	set -euo pipefail
	idx="reference/manual/INDEX.md"
	if [ ! -f "$idx" ]; then
		echo "reference/manual/ is not populated yet — nothing to grep. See reference/NOTES.md."
		exit 0
	fi
	row=$(grep -i "| {{topic}}" "$idx" || true)
	if [ -z "$row" ]; then
		echo "no INDEX.md row matches topic '{{topic}}'. Rows are:"
		grep -E '^\| [a-z-]+ \|' "$idx" | awk -F'|' '{gsub(/^ +| +$/, "", $2); print $2}'
		exit 1
	fi
	echo "$row"
	echo "---"
	pages=$(echo "$row" | awk -F'|' '{print $3}')
	for range in $(echo "$pages" | tr ',' ' '); do
		start=$(echo "$range" | cut -d- -f1 | tr -d ' ')
		end=$(echo "$range" | cut -d- -f2 | tr -d ' ')
		[ -z "$end" ] && end=$start
		for p in $(seq "$start" "$end"); do
			f=$(printf "reference/manual/pages/p%03d.txt" "$p")
			[ -f "$f" ] && { echo "=== p.$p ==="; cat "$f"; }
		done
	done

# Open (creating if needed) today's journal entry for a role.
journal role:
	#!/usr/bin/env bash
	set -euo pipefail
	day=$(date +%F)
	dir="journal/{{role}}"
	file="$dir/$day.md"
	mkdir -p "$dir"
	if [ ! -f "$file" ]; then
		{
			echo "## $day"
			echo
			echo "**Intend to change:** "
			echo "**How I'll know it worked:** "
		} > "$file"
	fi
	echo "$file"
