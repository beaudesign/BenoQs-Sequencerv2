# ADR 0012: The number keys enter the inventory, and the ring keys wait for their names

## Status

Proposed on 2026-10-13 by the Conductor. It takes effect when the owner merges the pull request that carries it, as ADR-0007 and ADR-0010 did. If the owner closes it, `contracts/controls.json` stays as it is (247 controls) and the ten number keys stay inert on the page.

It is S0 of SPEC-0003 (`WENGE-0028`, draft PR #59). The owner answered that spec on Tue 2026-10-13, 13:41 London, in chat: "approve, all defaults". **This ADR departs from one of those defaults, D5, and says so in decision 2.** The owner's merge is the answer to the departure; the rest of D5, asking the owner once to read the printed names of the ring, stands.

This ADR changes no file other than itself. The contract change it authorises is a second pull request, after this one has merged (decision 5).

## Context

**What SPEC-0003 asks for.** The circle is where a pattern is given its meaning, and the page binds five of its 42 controls. The first thing a user does there is key in a number (the tempo, a velocity, a certainty, a program), and the manual's rule for that is one sentence: "a double-click will set a 10's value and a single-click will set a 1's value" (p17, for the matrix keys; p52 and p57 say the circle's keys work the same way). The ten keys that do it are drawn on the page today and do nothing, because no control in the inventory is theirs.

**What the inventory says about them.** `controls.json` has one `open` group, `circle.numbers`, that holds the "number keys", the "scale keys (Maj, Min and others)" and the "chain selectors XXIX to XXXI", with question Q32 and the note that "their count and positions are not stated". An `open` group "becomes controls by amendment when a hardware check settles it" (ADR-0007 decision 1).

**What the plate says.** `apps/web/layout/panel.layout.json` (measured from the owner's photograph by `handoffs/evidence/panel-layout/`, committed as numbers; no photograph is in any context) places **thirty-seven keys that no inventory names**. Ten of them are printed `1` to `10` along the upper-left arc of the outer ring (`hw.num.1` to `.10`, basis `photo`: the label was read). Thirteen more are on the ring and their printed names are about four pixels high and cannot be read (`hw.ring.1` to `.13`). The rest are the MIX TARGET row, SEL, MAP and a left transport key, and are not this ADR's.

**The rules this has to fit.**

| Rule | Says | Bears on |
|---|---|---|
| ADR-0007 decision 1 | The contract "holds what the manual says and nothing else"; every control cites a manual page; a design choice does not belong in a contract; adding a control is an ADR | The count of ten comes from the plate, not the manual |
| ADR-0007 decision 1 | `n` is "assigned once, never reused, never renumbered"; "a rename is an ADR" | A placeholder id that is later renamed or withdrawn costs an ADR each time |
| ADR-0007 decision 2 | `ControlId` means `n` | A key with no `n` cannot be pressed, so it cannot be bound |
| SPEC-0003 D5 (approved) | The thirteen unnamed ring keys get ids by position (`circle.ring.1` to `.13`), flagged; functions keep their own ids and are bound to a key when the key is identified | See decision 2 |
| `CLAUDE.md` rule 7 | No reference photography in context | The ADR works from the numbers in the layout file |

**Precedent, and why it does not settle the ring keys.** The twelve inner-circle note keys are in the contract by position (`circle.note.k1` to `.k12`, `unnamed`, `pending` Q33). They are there because the manual states that the inner circle has note keys and what they do; Q33 asks only which note is which. The ring keys are different in one respect that matters: **the manual states no function for any one of them**. It names functions (Chr., Follow, the presets, the scales "Maj, Min etc") and never says which key carries which.

## Decision

### 1. Ten number keys become controls

`circle.num.1` to `circle.num.10`, `n` **295 to 304** (free; zone `circle_outer`; the zone's last number today is 294), kind `button`, `label` the digit, `at.index` the digit, `name` "number key N", `cite` `["p017", "p052", "p057"]`: p17 for the single and double click, p52 for "values may also be entered using the keys of the circle fields", p57 for "the buttons in the numeric quadrant".

**No `led`.** The schema says `led` is `true` "when the manual gives the control an LED". The manual gives the numeric quadrant a display (`circle.numeric`, n 294, unplaced) and says that "an Orange dot at the 1 position" can be lit (p54); it does not say that the dot beside each number key is the key's own lamp. The plate draws a dot beside each key (`led: "n"` in the layout, basis `photo`). That is the case the repo has already met with Stop, Play and Pause: the photograph has a dot, the manual gives no LED, the contract has no `led`, and the page draws the dot dark (`layout.test.ts`). The ten keys are treated the same way. If Q43 settles that the quadrant is these dots, `led: true` is added under an Amendments line of this ADR, and the layout needs no change (it already has the side). S1's asserted observables are on the display layer, not on the lamps (SPEC-0003 `tech.md` sections 2 and 3: the meaning layer is asserted, the rendering is provisional), so S1 does not wait for that.

**Why the plate may settle the count.** That there are ten, and that they are labelled 1 to 10, is a fact the manual does not state and the plate states legibly. ADR-0007 asks for "a hardware check" before an `open` group becomes controls. The owner named the genoQs Octopus article, the page the photograph is on, "the original spec to vuild this" (2026-10-07, 18:03; ADR-0010 records it). ADR-0010 and P7 already take the plate's geometry and colours from the photograph as numbers, and the ten labels are the only plate fact in this ADR that was *read* rather than measured. This ADR treats them as the check. It is the first time a plate fact enters the contract, so it is said here and in the `open` group's note (below), and it stops at facts the plate states legibly.

**The `circle.numbers` group** keeps its other two names (scale keys, chain selectors), keeps Q32, and its note is rewritten: the number keys are controls as of this ADR, ten, from the plate; what each of the thirteen remaining ring keys is, is not stated.

### 2. The thirteen ring keys do not enter the contract (D5's default is not taken)

SPEC-0003 D5 approved "ids by position (`circle.ring.1` to `.13`), flagged". This ADR proposes not to, for three reasons, and the owner's merge decides:

1. **A positional id buys nothing.** No function can be bound to a key whose function nobody has read. Between now and the owner reading the ring, the controller would hold thirteen controls that it ignores.
2. **It costs two ADRs later, not one.** `circle.chr` and `circle.follow` already exist in the contract (n 290, 291), unplaced because the plate's readable keys do not include them. When the owner says "ring key 7 is Chr.", the key is `circle.chr`, and `circle.ring.7` is a duplicate of it that must be withdrawn: a removal is an ADR, and `n` 7's number is never reused. The same applies to each of the five other named functions and to every scale key.
3. **The route that costs nothing exists.** When a ring key is read, the layout file's `hw.ring.k` is renamed to the contract id that already exists (`circle.chr`, `circle.follow`) and the controller can bind it: no contract change. For a key the manual does not already have an id for (a scale name, a preset), it is **one amendment of ADR-0007, adding that one control** with its real name and a cite.

**What this changes in SPEC-0003**, and nothing else: S0 promotes ten keys, not twenty-three; the drawn count rises by ten, not twenty-three. Every observable in `plan.md` S1 to S8 that uses a ring key's name is already `pending` (K-12, H-13) or does not use the ring (S1 uses the number keys; S3 composes a scale from the note keys and Select; S5 builds a chain with X). **No slice loses an asserted observable.** What waits is the same thing that waited before: the names. The owner is still asked once to read them.

### 3. The numbers that move

| What | Before | After | Where it is asserted (all under `apps/web/test/`) |
|---|---|---|---|
| Controls in `controls.json` | 247 | **257** | `layout.test.ts:45` (inventory length) |
| Controls drawn, each once | 242 | **252** | `layout.test.ts:51`; `browser/panel.browser.test.ts:80`, and the test's name at `:77`; `browser/app.browser.test.ts:413` |
| Not drawn, with a reason | 5 | 5 | `layout.test.ts:35` (`UNPLACED`, unchanged) |
| Plate keys in no inventory, drawn and inert | 37 | **27** | `layout.test.ts:56`; `browser/panel.browser.test.ts:92` |
| Tab stops | 242 | **252** | `layout.test.ts:382` |

Every number rises or stays, except the inert count, which falls because ten of its keys have become controls: the page draws more controls, makes them tab stops with the manual's names, and has fewer inert keys. **No threshold is loosened.** These are exact-equality assertions about the inventory, and they are changed in the same pull request as the contract, so the head of that pull request is green. The only test whose *name* changes is the browser test at `:77`, which carries "242" in its title; it is a Node test, and no `harness/baseline.txt` line names it (the baseline holds Rust `test` and `fixture` ids), so nothing leaves the baseline (`CLAUDE.md` rule 3).

The controller changes nothing: "a number the controller does not use is ignored and is not an error" (`ABI.md`). Pressing a number key does nothing until S1.

### 4. The policy, so it does not grow

A control may enter the contract on a plate fact only when **(a)** the manual states what the control is for and cites a page, **(b)** the plate states its count or label *legibly*, as a number in `panel.layout.json` with basis `photo`, and **(c)** the ADR that adds it says so. A key the plate shows and the manual gives no function for stays in the layout as `hw.*`. This is a rule for ADRs, not a new schema field.

### 5. Who does what, in what order

| Step | By | What | Tier |
|---|---|---|---|
| 1 | Owner | Merges this ADR | |
| 2 | Conductor, then Forge, one pull request, two commits, one hat each | Commit one (Conductor): the ten controls and the note in `contracts/controls.json`; an ADR-0007 Amendments line. Commit two (Forge): `hw.num.N` becomes `circle.num.N` in `panel.layout.json`, and the count assertions above | High |
| 3 | Panelwright | Files Q43 to Q56 in `QUESTIONS.md` (Low; its own pull request) | Low |

One pull request for step 2 because the layout test crosswalks the contract and the layout: either file changed alone is red. The layout test is red between the two commits and green at the head, which is what CI runs and what the owner merges. The two hats stay in two commits because `.github/CODEOWNERS` groups `contracts/` under the Conductor; if the owner would rather every commit were green, the alternative is one commit declared under both hats, and the owner can say so in review.

## Alternatives considered

- **Take D5 as approved: twenty-three positional ids now.** Rejected, decision 2.
- **Wait for the owner to read all the ring before S0.** Rejected: ten of the keys are legible, S1 needs them, and nothing in S1 needs the other thirteen.
- **A `source: plate` field in the schema.** Rejected for now: it adds machinery for ten keys, and the rule of decision 4 is enough. If a second plate fact comes along that does not fit the rule, revisit.
- **Leave the number keys in the layout and let the page send plate positions to the controller.** Rejected: `ControlId` means `n` (ADR-0007 decision 2); a second numbering would make every fixture say which one it means.
- **Put the number keys in the contract as `pending` Q32 only.** Rejected: Q32 is open for the *scale keys and chain selectors*; the number keys' count is settled by the plate.

## Not decided here

- Which of the thirteen ring keys is Chr., Follow, a preset or a scale name (Q32). The owner reads them; the first amendment follows.
- How the numeric quadrant draws a number, and whether its digits are the number keys' dots or other lamps (Q43). Until that is settled the dots are drawn dark, as Stop, Play and Pause are, and the controller claims no ring key.
- The sixth MODE key (`hw.mode.map`, Q23) and the left transport key (`hw.transport.left`, Q21). They stay inert.
- Any change to the engine, the ABI, the tokens or the look.

## Consequences

- S1 (the number and the tempo) can start as soon as step 2 has merged: the keys it needs have numbers.
- The contract grows by ten entries that each cite three manual pages; this ADR, not the contract, says where the count came from. They claim no LED the manual does not give.
- The twenty-seven keys still inert on the plate are all ones whose function the manual does not state per key. That is the honest list of what the manual cannot tell us.
- SPEC-0003's D5 reads differently after this merges. An Amendments line in its README says how (it is in the draft pull request for the spec, and in force when the owner merges this).
- Revert: remove the ten entries, restore the `open` group's note and the five counts. Nothing reads the new entries yet except the layout test, so there is no data to migrate.

## Amendments

None yet.
