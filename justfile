# WENGE — every verb an agent needs. See agents/CLAUDE.md "Commands".
#
# `just verify:<zone>` from CLAUDE.md isn't literal — `just` recipe names can't
# contain `:` — so zone gates are `verify-<zone>` here. `verify` is the
# umbrella: it runs every gate, reports which are wired vs. not yet
# implemented, and only fails the build on a wired gate that actually failed.

zones := "geometry color frames motion timing conformance acoustics a11y arch slop tokens regressions determinism"

# Run every gate; fail only on a wired gate that fails.
verify:
	#!/usr/bin/env bash
	set -uo pipefail
	fail=0
	for z in {{zones}}; do
		echo "== verify:$z =="
		just "verify-$z"
		code=$?
		if [ $code -eq 0 ]; then
			echo "-> pass"
		elif [ $code -eq 42 ]; then
			echo "-> not yet implemented (see docs/07-verification.md)"
		else
			echo "-> FAIL"
			fail=1
		fi
	done
	if [ -f crates/octocore/Cargo.toml ]; then
		echo "== verify:octocore =="
		just verify-octocore || fail=1
	fi
	exit $fail

# octocore is the one zone with real code once crates/octocore exists.
verify-octocore:
	#!/usr/bin/env bash
	set -euo pipefail
	if [ -f crates/octocore/Cargo.toml ]; then
		cd crates/octocore && cargo test
	else
		echo "octocore: not yet implemented"
		exit 42
	fi

# Every other zone: honest "not built yet" until its owning role lands it.
_stub name role:
	@echo "{{name}}: not yet implemented — owned by {{role}}, see docs/07-verification.md"
	@exit 42

verify-geometry:
	@just _stub geometry panelwright
verify-color:
	@just _stub color forge
verify-frames:
	@just _stub frames referee
verify-motion:
	@just _stub motion forge
verify-timing:
	@just _stub timing metronome
# octocore's `tests/conformance.rs` runs every tests/conformance/**/*.fixture
# (excluding pending/); wired here once octocore exists so this gate stops
# being a stub even though jitter/host-loopback timing (verify-timing) isn't.
verify-conformance:
	#!/usr/bin/env bash
	set -euo pipefail
	if [ -f crates/octocore/Cargo.toml ]; then
		cd crates/octocore && cargo test --test conformance
	else
		echo "conformance: not yet implemented — owned by metronome, see docs/07-verification.md"
		exit 42
	fi
verify-acoustics:
	@just _stub acoustics sceneshaper
verify-a11y:
	@just _stub a11y loom
verify-arch:
	@just _stub arch conductor
verify-slop:
	@just _stub slop curator
verify-tokens:
	@just _stub tokens curator
verify-regressions:
	@just _stub regressions referee
verify-determinism:
	@just _stub determinism referee

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

# Grep the Octopus reference manual for a topic. Not populated yet.
manual topic:
	@echo "reference/manual/ is not populated yet — nothing to grep. See reference/NOTES.md."

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
