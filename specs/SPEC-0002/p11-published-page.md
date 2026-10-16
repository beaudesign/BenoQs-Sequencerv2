# SPEC-0002 P11: a page you can open at an address

| Field | Value |
|---|---|
| Task | `WENGE-0029` (the owner, Wed 2026-10-14 18:45 London: "Can;t se the sequencer"; Fri 2026-10-16 13:19: "Let's get it running") |
| Spec | SPEC-0002 P3c (the web app) and P6 (it runs when opened) |
| Plan revision | r0, 2026-10-16 |
| Tier | **Medium**, Forge and Referee hats: `apps/web/` and `.github/workflows/` (CI is medium tier, `AGENTS.md`). No contract, threshold, golden file or engine code is touched |
| Stacked on | Nothing. It branches from `main` at `72ad32c` |

## 1. What the owner asked, and how it is read

The sequencer has only ever run for the owner when an agent started it: on a local server, or as a private preview on claude.ai that needs a sign-in. The owner could not see it. "Let's get it running" is read as: **a link that opens it, on any device, with nothing to install.**

The repository is public, and the page is a static build (a WebAssembly module, JavaScript, two stylesheets, a font and three JSON files), so a static host is enough. The one the repository already has is GitHub Pages. The page needs an address that is a secure context (HTTPS) for the audio worklet and Web MIDI, which a Pages address is.

## 2. The finding that makes this more than a workflow

A project site is served under a prefix, `https://<owner>.github.io/<repo>/`. The app loaded every file by an absolute path (`/dist/octoweb.wasm`, `/contracts/controls.json`, ...), which is the root of the address, so under a prefix it found none of them and drew nothing. The change to the app is small (the page and its script find their files from where they are) and the test that shows it is the first thing in the pull request.

## 3. What changes, with the observable for each (tests written first)

| # | Change | Observable |
|---|---|---|
| P11-1 | `scripts/site.ts` assembles the folder a static host serves: the page and exactly the files it loads, in the layout it expects, an index, and `.nojekyll` | `published.browser.test.ts` |
| P11-2 | The page runs from under an address prefix with no cross-origin-isolation headers: it draws all 252 controls. Every file it loads comes from inside the prefix and none fails. The site holds the font's licence and no spike, test or TypeScript source, and the module without the spike exports | `published.browser.test.ts` |
| P11-3 | The site's own address opens the app | `published.browser.test.ts` |
| P11-4 | One press of Play on the site plays, and it is heard | `published.browser.test.ts` |
| P11-5 | No page error and no console error from the first load to a second of playing | `published.browser.test.ts` |
| P11-6 | The workflow starts only on a push to `main` or by hand; the workflow may only read and the deploy job alone may publish; it builds what the tests serve and uploads that folder and nothing else; no secret; every action official and at a version tag | `pages-workflow.test.ts` |

Red first: on `be4be6c` five of the six Chromium tests fail (0 of 252 controls drawn); the workflow test fails with a missing file. Six single-line mutants of the app and the assembler, and eight of the workflow, are each caught.

## 4. Decisions the owner can change, with the default taken

| # | Decision | Default and why |
|---|---|---|
| D-P11-1 | The host is GitHub Pages, deployed by a workflow | The repository is public and already on GitHub; HTTPS; no new account, no new secret, nothing to pay |
| D-P11-2 | The workflow runs on every push to `main`, and by hand | The owner reviews by looking at the running page, so the page should be `main`. The alternative, a deploy only on a tag, would show a page that is behind what was merged |
| D-P11-3 | What is published is the page and the files it loads, and nothing else of the repository: not the manual, the specs, the journals, the spikes or the tests | The site is the instrument, not the repository. The manual is in the repository, which is public |
| D-P11-4 | Every file the page loads is found from the page, with no setting for a prefix | A setting would be a second way to be wrong. One rule (relative to the page) works at a root, under a prefix and from a folder |
| D-P11-5 | The link is not added to the README here | The address exists only after the first deploy. A later Scribe change adds it from the run's output, so the README never names a link that is not live |
| D-P11-6 | The spike and test pages keep their root-relative paths | They are served only by `scripts/serve.ts`, which is the root of an address, and they are not published |

## 5. What the owner does, and when

1. Read this plan and the pull request.
2. **Before merging**, switch Pages on: repository **Settings, Pages, Build and deployment, Source: GitHub Actions**. Only the owner can (the agent's access does not reach it, and publishing is the owner's to start).
3. Merge. The merge's push to `main` starts the workflow. If Pages was not on yet, the deploy job fails and nothing else does: switch it on and re-run that job, or use **Actions, pages, Run workflow**.
4. The run prints the address, and it is `https://beaudesign.github.io/BenoQs-Sequencerv2/` if the repository keeps its name. Open it in **Chrome or Edge** for MIDI.

## 6. Rollback

Delete `.github/workflows/pages.yml`, or set the Pages source to none in Settings: the site is taken down and nothing else changes. The app's relative paths are harmless at a root.

## 7. What is not done

- The link in the README (D-P11-5), and a screenshot of the published page there.
- A custom domain.
- Any caching or version marking of the site. Every push replaces it.
- Safari and Firefox have no Web MIDI; the page says so and plays through its built-in sound, as it does anywhere.
