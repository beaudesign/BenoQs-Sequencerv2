# Request to Referee: enforce the expiry of the two tinted colour roles

From: Conductor (`WENGE-0021`, P8a, ADR-0010 decision 5). To: Referee (`xtask/`, `harness/`, the gates). Tier of P8a: **High**; this request changes nothing in P8a.

ADR-0010 loosens ADR-0008 rule 7 for two roles, `plate` and `frame`, and the loosening **expires on 2027-01-31**. In `contracts/design.tokens.json` each carries `exemption: {rule: "T7", why, expires: "2027-01-31"}`. `tests/tokens.rs` checks that the exemption exists, names T7, says why, and has a date in `YYYY-MM-DD`
form. It does not compare the date to anything: a test that reads the clock is not deterministic, and `determinism` is a required gate.

What is asked (the Referee's to design):

1. A check, in the `tokens` gate (not implemented yet; the Curator's) or `regressions`, that fails when **the date of the checked-out commit** (`git log -1 --format=%cI HEAD`, which is the same every time the same commit is checked out) is on or after the earliest `expires` of any
   `exemption` in the tokens file. After that date `plate` and `frame` must be pure grey (T7's NEUTRAL rule) unless the owner has renewed the exemption by merging an amendment to ADR-0010 with a new date.
2. Red first: a fixture with a past date fails, a future date passes, and a commit date is injected, not read from the clock.
3. Decide what a pull request's "HEAD" is on CI (the PR's last commit, or the merge commit). A branch committed before the date and merged after it passes under the first and fails under the second; the second is the stricter.

Until this exists the expiry is a date in a file and the owner's memory. It is the only part of ADR-0010 that nothing enforces.
