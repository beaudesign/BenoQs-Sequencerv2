# WENGE: every verb an agent needs. See agents/CLAUDE.md "Commands".
#
# `just verify:<zone>` from the docs is not literal: `just` recipe names cannot contain
# a `:`, so zone gates are `verify-<zone>` here.
#
# The gates live in `xtask/` (run as `cargo xtask`, see harness/README.md). `verify`
# runs all of them, writes harness/report/latest.json and exits non-zero if any gate
# failed or any gate in harness/required-gates.txt did not pass. A gate that is not
# built reports `not_implemented`; it is never a silent pass.

# Where cargo puts its output: the checkout's target/, or CARGO_TARGET_DIR if it is set.
target := env_var_or_default("CARGO_TARGET_DIR", "target")

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

# The octoweb module for the web app, into apps/web/dist: the shipped build (no allocation counter,
# no pattern runner) as octoweb.wasm, and the build with both, for the tests and spike S2, as
# octoweb-spike.wasm. Needs `rustup target add wasm32-unknown-unknown`. See apps/web/engine/ABI.md.
web-wasm:
	mkdir -p apps/web/dist
	cargo build -q --release -p octoweb --target wasm32-unknown-unknown --no-default-features
	cp {{target}}/wasm32-unknown-unknown/release/octoweb.wasm apps/web/dist/octoweb.wasm
	cargo build -q --release -p octoweb --target wasm32-unknown-unknown
	cp {{target}}/wasm32-unknown-unknown/release/octoweb.wasm apps/web/dist/octoweb-spike.wasm

# The web app's type-check and its Node tests (the real worklet file in a stand-in scope, the ABI
# wrapper, the MIDI scheduler and the Stop property). Needs Node 22.18 or newer.
web-test: web-wasm
	cd apps/web && npm ci --no-audit --no-fund && npm run typecheck && npm test

# Chromium: the real host, worklet and scheduler with a recording MIDI output (apps/web/test/browser).
# Needs `npx playwright-core install chromium` once, or a Chromium on PLAYWRIGHT_BROWSERS_PATH.
web-browser-test: web-wasm
	cd apps/web && npm ci --no-audit --no-fund && npm run test:browser

# Spike S2: the engine in an AudioWorklet, hashes against native and the cost of render. Add `--write` to save the
# record under handoffs/evidence/. See specs/SPEC-0002/p3-plan.md section 5.
spike-s2 *args: web-wasm
	cd apps/web && npm ci --no-audit --no-fund && npm run spike:s2 -- {{args}}

# Spike S1, the half that runs here: scheduling margin with a stand-in MIDI output. `--hidden` also runs it with the
# tab in the background (needs xvfb or a display). The real-port half is apps/web/spikes/s1/page.html, for the owner.
spike-s1 *args: web-wasm
	cd apps/web && npm ci --no-audit --no-fund && npm run build && node spikes/s1/run.ts {{args}}

# Record new tests and fixtures as the ratchet floor. Dropping one needs --remove <kind> <id> --adr ADR-NNNN.
baseline *args:
	cargo xtask baseline {{args}}

# Fast check for the sequencer core only (seconds). Not a gate.
verify-octocore:
	cargo test -p octocore

# Per-zone gate verbs below. Owners are listed in xtask/src/gates.rs.

verify-timing:
	@cargo xtask gate timing
verify-conformance:
	@cargo xtask gate conformance
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
verify-scope:
	@cargo xtask gate scope

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
