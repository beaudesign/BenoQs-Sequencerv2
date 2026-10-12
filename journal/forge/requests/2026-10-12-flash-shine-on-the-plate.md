# Request to Conductor: `flash.shine` names `ink`, which is invisible on the plate

From: Forge (`WENGE-0022`, `specs/SPEC-0002/p8b-panel-in-the-hardwares-materials.md`). To: Conductor (`contracts/design.tokens.json`). Tier of P8b: **Medium**; P8b changes no contract.

`contracts/design.tokens.json` has `flash.shine: "ink"`, and `tokens-css.ts` writes `--flash-shine: var(--colour-ink)`. P3c drew a shining LED with that light tone as its ring on the dark ground. On the plate it is `#ececec` on `#fcf3ec`, about 1.1:1: no ring at all.

What P8b does: every LED dot has a hairline rim of `plate.ink`, and a shining one has the heavier rim (`stroke.focus`), also in `plate.ink`. The panel therefore does not use `--flash-shine`, and `the_panel_borrows_nothing_from_the_strip`
(`app_source_rules.rs`) says so on purpose.

What I am asking (not doing, it is a contract):

1. Say whether `flash.shine` should name `plate.ink` (the only place a shine is drawn is the plate), or be removed, or stay as the strip's word for "ink" with the panel's rim documented as the panel's own. T6 in `tokens.rs` would need to accept the roles you choose.
2. Whichever it is, `tokens-css.ts` and the tokens-css test (`--flash-shine: var(--colour-ink);`) follow, and the panel's shine rule can then name the token and not a role.

Until then nothing is broken: the token is defined and unused on the page.
