# Colosseum — GUI guide

<!-- board: generated from PLAN_GUI.md by `python tools/guide_board.py`; edit PLAN_GUI.md, never this block -->
**Now: A** (`R2`): Revise and decide the GUI programme. Waits for the maintainer's revision of the programme.

## History — CLOSED

Naming (ADR-0012), the GUI repository's seed, the egui application retired
from this repository (2026-09-29), and the swap and merge done at the split
(2026-11-14), as the steps of the old Phases 11 and 14 they were.

## Decision

- [ ] **A** Revise and decide the GUI programme — **R2**

## Separation

- [ ] **B** The CLI against the requirements — **R2**
    - [ ] **B.1** Decide engine option resolution — **R2**
    - [ ] **B.2** Decide ponder — **R2**
    - [ ] **B.3** Decide adjudication — **R2**
    - [ ] **B.4** Decide faults in tournaments — **R2**
    - [ ] **B.5** Decide scheduling — **R2**
- [ ] **C** Protocol specification — **R2**, EXIT

## Protocol

- [ ] **D** Session — **I2**
- [ ] **E** Synthetic streams — **I1**
- [ ] **F** Live events — **I2**
- [ ] **G** Control and lifecycle — **R2, then I2**
- [ ] **H** Amendments — **I2**
- [ ] **I** Changed engines — **I1**
- [ ] **J** Queries — **I1**
    - [ ] **J.1** Tournament statistics — **I1**
    - [ ] **J.2** Tournament options for the application — **I1**
- [ ] **K** Conformance and candidate — **V**, EXIT

## Release

- [ ] **L** Releases — **V**, EXIT
<!-- end of generated board -->

The short operational view of the GUI programme: where it stands and what to
do next. Its requirements, protocol and steps in full are in
[`PLAN_GUI.md`](PLAN_GUI.md). The CLI's general work is the requests plan,
[`PLAN.md`](PLAN.md), with its board in [`GUIDE.md`](GUIDE.md); it never
waits for this one.

## Reading the board

The board above is generated from `PLAN_GUI.md`'s step heads by
`python tools/guide_board.py`, which writes `GUIDE.md`'s board too; never
edit it by hand. Steps are lettered, A onward; a sub-step is a letter and a
number (B.1). `python tools/guide_board.py --check` fails on a stale board,
and CI runs it. A step's class maps to a model in `GUIDE.md`, *Current model
mapping*.

## Current checkpoint

| | |
|---|---|
| Status | **Under revision** (2026-11-20): the maintainer is deciding whether, and in what form, the programme continues — step A |
| What exists | The decision of 2026-09-27 ([ADR-0011](docs/architecture/adr/0011-desktop-application-as-separate-cli-client.md)); the names ([ADR-0012](docs/architecture/adr/0012-names-and-repositories-after-the-split.md)); the GUI repository, [`maelic13/colosseum`](https://github.com/maelic13/colosseum), seeded; the egui application retired from this repository; the swap and merge done at the split (2026-11-14). No protocol code exists |
| What is missing | A's decision; then, if the programme continues, the gap analysis and its decisions (B, B.1–B.5), the protocol specification (C), the protocol (D–K) and the release pair (L). In the desktop repository: GUI Phases 1–5 |
| Kept in the CLI for it | The runner's live-game state (`engine::live`), written on every search and read by nothing today, and `performance_rating`, unused: F and J.1 would use them. If A stops the programme, a requests-plan step removes them |
| Next step | **A** — revise and decide the programme (`R2`) |

## What to do now

1. **A** — the maintainer decides: continue as specified, continue narrowed
   to what any client could use, or stop. The decision goes in an ADR and
   `PLAN_GUI.md` is re-cut to it.
2. If it continues: the desktop repository's 1.2.0 from `legacy/`, then GUI
   1.1.1–1.1.4 and 1.2; B waits for GUI 1.2 (requirements signed off); C
   follows and unblocks the GUI screen specifications (GUI 3.x).
3. Then D–K in order. E unblocks the GUI performance proof (GUI 2.3); F and
   G unblock the GUI live view (4.5); G's lifecycle decision is taken
   together with GUI 4.2; H–J unblock GUI 4.6. K produces the CLI release
   candidate the GUI accepts against (GUI 4.8).
4. L: the CLI release, then the GUI's 2.0.0 release pinned to it.

Each design step (GUI 1.1–1.4, 3.1–3.3) ends in a maintainer sign-off,
reviewed in pairs; no GUI screen is implemented before its specification is
signed off.

## Decision rules

| Situation | Action |
|---|---|
| A GUI behaviour differs from the CLI's for the same mechanism | The CLI's is the specification (`PLAN.md` §S5); never add a second implementation |
| The GUI needs something the CLI does not provide | It is a CLI extension through the G2 protocol, recorded in B's gap analysis — never game-playing, rating or chess logic in the desktop application |
| Tempted to coordinate CPU placement across desktop tournaments | Placement is a CLI feature for gates and tunes; desktop tournaments run with it off |
| A protocol event could block on a slow reader | Not allowed: bounded queue, state dropped first, a `gap` fact instead of blocking (G2) |
| Tempted to decide the close / detach / re-attach behaviour early | Analysed and decided at G with GUI 4.2; until then every option must leave a resumable run |
| Tempted to implement a GUI screen before its specification is signed off | Not allowed: design first (`PLAN_GUI.md` §G1) |
| A protocol change in a CLI release | The CLI release is tagged first; the desktop release that needs it follows, bundling and pinning that CLI version. A protocol minor version never ships in a CLI patch release |
