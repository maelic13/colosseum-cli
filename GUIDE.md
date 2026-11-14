# Colosseum — development guide

The short operational view: where the work stands and what to do next.
Rationale, specifications, success criteria and the open items' full form
live in [`PLAN.md`](PLAN.md); the evidence of completed phases lives in
[`docs/architecture/`](docs/architecture/README.md).

**This file and `PLAN.md` are the maintainer-facing pair.** `README.md` is the
user-facing front door; `docs/cli/` is the user-facing CLI detail. Neither may carry phase numbers,
internal naming or method argumentation.

## Current checkpoint

| | |
|---|---|
| Branch / versions | `master` holds **CLI 0.5.0** (2026-10-29), the latest release; releases are plain `vX.Y.Z` tags (`v0.1.0`–`v0.5.0` re-tagged from `cli-v`). This repository is CLI-only since the split, 2026-11-14: `maelic13/colosseum-cli`, with the desktop application in `maelic13/colosseum`. Work lands on `master` as one squash pull request per piece; `dev` was the last long-lived branch |
| What exists | Phases 0–10 complete and published (10.9m rejected; 10.9h, 10.9p, 10.9t deferred). Phase 12.1–12.4 published as CLI 0.2.0; CLI 0.3.0 (engines kept per slot in tournaments), 0.4.0 and 0.5.0 (run, tournament and SPSA reporting) from adoption. GUI v2 decided 2026-09-27 ([ADR-0011](docs/architecture/adr/0011-desktop-application-as-separate-cli-client.md)); 11.1, 11.2 and 11.2.1 done; the swap and this repository's merge done at the split instead of at the end ([ADR-0012](docs/architecture/adr/0012-names-and-repositories-after-the-split.md), *Revision 2026-11-14*) |
| What is missing | **Phase 12(d)**, the engine projects' requests (12.7–12.19 from Rarog's list of 2026-11-13, 12.10.1 from Basilisk), released as **CLI 0.6.0** and later minors — 0.6.0 the first release through pull-request delivery. Then **Phase 11** (11.3 gap analysis and its decisions 11.3.1–11.3.5, the 11.4 protocol specification), **Phase 13** (the protocol, one CLI minor release) and **14.2** (the release pair); 12.5. In the desktop repository: GUI Phases 1–5 |
| Validation engines | Two independently developed UCI engines in different languages and build systems. Any two UCI engines would serve; nothing depends on which |
| Platform status | Windows/Linux/macOS ☑ required debug and optimized CI on every pull request into `master` · Windows x86-64/ARM64, Linux x86-64 and macOS ARM64 release archives ☑ built, packaged and smoked as a candidate on every pull request, exactly as a tag builds them |
| Next step | **12.7** — `winc` and `binc` from each side's own control — `I1` (Claude Sonnet 5 — Medium), then 12.8–12.10, the other small corrections; then 12.10.1, Basilisk's `spsa read`; then 12.11 and 12.12, Rarog's priorities; 12.10.1, 12.11 and 12.12 each its own pull request; the 0.6.0 release pull request when the maintainer closes its scope. In the desktop repository: 1.2.0 from `legacy/`, then **1.1.1–1.1.4** and 1.2. 11.3 waits for GUI 1.2 |
| Confirmed decisions | The 2026-09-22 set, confirmed by the maintainer and not reopened, recorded with the release preparation in [`phase-10-record.md`](docs/architecture/phase-10-record.md): exclusion for unspawnable engines, the xtask surface, versions kept in artifact names, the throughput margin not chased. The 2026-09-27 set for GUI v2, in [ADR-0011](docs/architecture/adr/0011-desktop-application-as-separate-cli-client.md) and PLAN §Phase 11: design first; the desktop application a separate repository over a CLI protocol; this repository the CLI alone; 1.x tournaments not migrated; no CLI runs in the GUI; device theme with light and dark; no served dashboard, broadcast and remote control kept possible; front-end technology provisional (TypeScript + React) until measured. The same day's review set, in PLAN §S8 maintainer requirements and §S5.15: placement stays a CLI feature and desktop tournaments run with it off, several at a time; 64 participants the design point; lifecycle and close behaviour analysed at implementation (13.4 / GUI 4.2) with *stop now on close* the starting preference; changed engines and amendments (add, remove, length) are CLI mechanisms decided at 11.3; configuration by run file, a rebuildable index, positions as FEN, two event classes, emission off the game path, a handshake version, short-lived queries, synthetic streams; the shell a measured choice at GUI 2.3; the 1.x engine library imported. The 2026-09-28 naming set, in [ADR-0012](docs/architecture/adr/0012-names-and-repositories-after-the-split.md): Colosseum and `colosseum-cli` kept; the naming risk accepted, both products renamed together if a conflict ever arises; the new application's data directories apart from 1.x's. The 2026-11-14 split set, in ADR-0012's revision: the swap and this repository's merge at the split; plain `v` tags with the `cli-v` releases re-tagged; no bridge release; the 1.x releases deleted here once desktop 1.2.0 is out; the prototype and x64 branches dropped; Rarog's delivery adopted — short-lived branches, one squash pull request each with a short title and description, CI and the release candidate on every pull request; the Rust toolchain pinned (1.100.0), each update its own tested pull request |

## Current model mapping

PLAN §S8 records each step's stable capability class; this table maps the
classes to the current Claude model and thinking mode and is the only place
model names appear. Edit it when model generations change; do not rewrite
the roadmap. These are maintainer judgements, not measured rankings.

| Class | Capability | Claude |
|---|---|---|
| `R3` | Frontier causal/architecture research | Claude Fable 5.1 — High |
| `R2` | Bounded correctness-sensitive reasoning | Claude Opus 5 — High |
| `I2` | Difficult implementation | Claude Opus 5 — High |
| `I1` | Well-specified implementation | Claude Sonnet 5 — Medium |
| `M` | Mechanical/docs/provenance | Claude Sonnet 5 — Medium |
| `V` | Verification/measurement | Claude Sonnet 5 — High |

## Completed phases

One line each; the numbered steps, their model labels and their evidence are
in [`docs/architecture/phases-0-9-record.md`](docs/architecture/phases-0-9-record.md)
and [`docs/architecture/phase-10-record.md`](docs/architecture/phase-10-record.md).
Step identifiers are never reused.

- ☑ **Phase 0** (0.1–0.8) — Current-state audit, Clean Architecture target, ADRs 0001–0008, independent product releases in one repository, the Colosseum naming decision.
- ☑ **Phase 1** (1.1–1.9) — Pentanomial statistics and normalized Elo in `colosseum-core`, the analytic and external oracle fixture corpus, the hermetic required suite.
- ☑ **Phase 2** (2.1–2.10) — Boundary migration, the independently versioned `colosseum-cli` package, run files, run records, run directories, `self-test`, `status`.
- ☑ **Phase 3** (3.1–3.8) — OS topology adapters, deterministic placement, hard affinity where the OS allows it, `capabilities`.
- ☑ **Phase 4A** (4A.1–4A.8) — Fixed-match runner with explicit clock accounting, typed engine faults, books and durability.
- ☑ **Phase 4B** (4B.1–4B.6) — Pair-atomic capped SPRT, oracle replay and live parity against fastchess and cutechess.
- ☑ **Phase 4C** (4C.1–4C.3) — Optional identical-binary calibration.
- ☑ **Phase 5** (5.1–5.10) — SPSA kernel and driver, `spsa plan`, `spsa status`, `sprt --apply`.
- ☑ **Phase 6** (6.1–6.9) — `nps` and scaling sweeps, `book` tools, `stats` replay and planning, `suite`.
- ☑ **Phase 7** (7.1–7.3) — Round-robin and gauntlet tournaments with joint or anchored ML ratings, matching the GUI on a frozen fixture.
- ☑ **Phase 8** (8.1–8.3) — Parity repeated on the candidate, ponder adopted, remaining gaps decided.
- ☑ **Phase 9** (9.0–9.7) — Naming retained (ADR-0009), versioned `docs/cli/` with a generated reference (ADR-0010), README, candidate bundle, coverage and usability acceptance, release acceptance of the step 9.7 candidate.
- ☑ **Phase 10** (10.1–10.10) — Adjudication off by default, class-aware placement, per-move PGN annotations, final-centre SPSA estimator, graceful stop, fixed rating field, book-range refusal, command-module split, shared game slots, pair identity in PGN, review defects, unscorable tagging, progress reports, commit path off the game loop, writer hardening, latency instrument, one slot per game, fault allowance, SPSA occupancy and scale, persistent engines per slot, real-engine qualification and symmetry, adoption-audit corrections, a game whose engine could not start excluded from scoring, the release versions and tag contract fixed, one build entry point for both products, user documentation reconciled with them, and the acceptance repeat passed. 10.9m rejected (the forfeits were an engine defect); 10.9h, 10.9p, 10.9t deferred behind the release; 10.9g's two maintainer probes still owed and not blocking.

## Forward tracker

<!-- TRACKER FORMATTING RULES — follow them, they get broken often:
     1. ONE step per bullet. Never join two steps on one line.
     2. Use renderer-independent Unicode markers; not every Markdown
        renderer implements GitHub task-list `[ ]` / `[x]` syntax:
            - ☐ **1.2** — todo
            - ◐ **1.2 — IN PROGRESS** — genuinely in flight
            - ☑ **1.2 — DONE** — resolved
     3. Resolved outcome labels are DONE · REJECTED · DEFERRED → <item> ·
        PARKED · FIXED. Anything resolved uses ☑, never ◐.
     4. Continuation lines indent 2 spaces. Sub-items indent 2 more spaces,
        use a normal `-` bullet, and indent their continuations another 2.
     5. Once implementation starts, NEVER renumber existing items — commits
        reference them. To insert before an item use a dotted suffix on the
        previous one (10.9w.1 follows 10.9w).
     6. Always mark a completed step here in the same commit. Add status or
        evidence to PLAN.md when it improves the durable specification; do not
        duplicate routine tracker detail there.
     7. Blank line AFTER the `###` heading, then NO blank lines between
        bullets: one continuous list per phase.
     8. ONLY NUMBERED STEPS live here. Recurring procedures go in their own
        section and never get a status marker.
     9. Every numbered step includes its PLAN §S8 model assignment. Keep the
        label and the routing table synchronized.
    10. When a phase completes, collapse its steps to one line under
        "Completed phases" and move its PLAN text to a record file under
        docs/architecture/. -->

Each phase ends with a verifiable exit criterion — see PLAN §S8. Nothing is
"done" because it compiles; it is done when its criterion is demonstrated.
When reporting the next step, always report its class and the model the
mapping above gives it.

### The GUI v2 programme — Phases 11, 13 and 14 here; GUI Phases 1–5 in the GUI repository

Both repositories advance together and release together; the joint
milestones M0–M9 are in PLAN §S8. A step here that names a GUI step waits
for it, and the GUI tracker says the same the other way.

#### Phase 11 — Separation

- ☑ **11.1 — DONE** — `R2` — Naming of both products, executables and
  repositories after a dated collision screen: Colosseum and
  `colosseum-cli` kept, the repository swap before both releases
  (revised 2026-09-29), plain `v` tags;
  [ADR-0012](docs/architecture/adr/0012-names-and-repositories-after-the-split.md)
  — PLAN §Phase 11 (M0)
- ☑ **11.2 — DONE** — `M` — The GUI repository seed finalised
  under the 11.1 names with ADR-0012's obligations and handed over;
  `maelic13/colosseum-gui` (now `maelic13/colosseum`) published with it as
  its root commit (`22d0be1`, identical to the seed at `54b9c47`);
  `docs/gui-v2/` reduced to a pointer (removed at the split); `README.md`
  names the new application — PLAN §Phase 11 (M1)
- ☑ **11.2.1 — DONE** — `I1` — After GUI 1.1, on `dev` only: the egui
  application, `engine::scheduler`, the SQLite store, the runtime adapter
  and the `gui-v` lane and packaging removed; `docs/design/` archived;
  documents describe a CLI-only repository; ADR-0006 superseded; `main`
  keeps 1.x until 14.1. Evidence 2026-09-29: `cargo fmt --check`, `check`,
  `clippy -D warnings`, 543 tests in 18 suites and the command-reference
  check green; only the architecture tests' forbidden-dependency guards
  still name the GUI stack — PLAN §Phase 11
- ☐ **11.3** — `R2` — After GUI 1.2: the CLI against every
  inventory and requirement item — supported / CLI extension / GUI-owned /
  not feasible; the changed-engine behaviour and the amendment semantics
  decided with the maintainer; `docs/architecture/gui-gap-analysis.md`;
  13.5–13.7 confirmed or re-cut; the inventory's CLI column taken in —
  PLAN §Phase 11 (M2)
- ☐ **11.3.1** — `R2` — Decide engine option resolution: threads and
  hash mapped and clamped, compatibility notes, tablebase settings and
  switch — in the CLI or composed by the application (inventory PLY-11–13)
  — PLAN §Phase 11
- ☐ **11.3.2** — `R2` — Decide ponder: `Ponder=false` when off; ponder
  without a clock or separate cores (PLY-14) — PLAN §Phase 11
- ☐ **11.3.3** — `R2` — Decide adjudication: defaults offered,
  one-sided resign, book and scoreless plies in the windows, the start
  position in threefold (PLY-15, PLY-35) — PLAN §Phase 11
- ☐ **11.3.4** — `R2` — Decide faults: an engine that cannot start,
  the fault allowance, a fixed-search hang (PLY-17, PLY-18, PLY-35) — PLAN
  §Phase 11
- ☐ **11.3.5** — `R2` — Decide scheduling: parallel games changed
  during a run; colours across cycles at one game per pair (PLY-26,
  DEF-14) — PLAN §Phase 11
- ☐ **11.4 — EXIT** — `R2` — Protocol specification: PLAN
  §S5.15 message catalogue, run-file hand-off, amendment and changed-engine
  semantics, versioning, JSON Schema from Rust types, conformance fixture
  format, synthetic-stream surface; `docs/cli/protocol.md` — PLAN §Phase 11
  (M3)

#### Phase 13 — The front-end protocol (one CLI minor release)

- ☐ **13.1** — `I2` — Session: flag, framing, handshake,
  bounded event pipeline with state coalescing, fact sequence numbers and
  `gap`, snapshot request, error responses, schema validation in tests; run
  state only — PLAN §Phase 13
- ☐ **13.2** — `I1` — Synthetic streams: a synthetic tournament
  at chosen scale and rate, and a replay of a finished run directory; both
  schema-valid without engines; recorded as fixtures — PLAN §Phase 13 (M4)
- ☐ **13.3** — `I2` — Live events from `tournament run`: game
  start, every move with SAN, clocks, FEN and the opening name, search state per side, game end,
  standings and rating snapshot per scored game; artifacts identical with
  and without the protocol (stub engines) — PLAN §Phase 13
- ☐ **13.4** — `R2`, then `I2` — Control and lifecycle: *stop* and *stop
  now* as protocol commands on every platform; the end-of-input, detach,
  re-attach and close-behaviour analysis done with GUI 4.2 and decided by
  ADR; clean-stop suite extended — PLAN §Phase 13 (M5)
- ☐ **13.5** — `I2` — Amendments: remove a participant, add a
  participant, change the length of a tournament in progress; append-only
  journal, stable game identity, amendment facts and record entries; kill /
  amend / resume tests; `stats` replays an amended run — PLAN §Phase 13
- ☐ **13.6** — `I1` — Changed engines: the 11.3 policy for
  an executable that changed since the run started, never silent, always
  recorded; a test per branch — PLAN §Phase 13
- ☐ **13.7** — `I1` — Queries: run summary, paged games with
  FEN per ply, standings and crosstable as read-only `--json` invocations
  within the CLI-side budgets on the largest target — PLAN §Phase 13 (M6)
- ☐ **13.7.1** — `I1` — Tournament statistics 1.x showed: SB and
  shared places, gauntlet score share, Perf, nps, depth, time per move,
  forfeits split, terminations, decisive/drawn; equal to 1.x on a frozen
  fixture — PLAN §Phase 13
- ☐ **13.7.2** — `I1` — Tournament options for the application:
  opening count cap, tournament name and date in the PGN, "name version"
  labels, and what 11.3.1–11.3.5 put in the CLI — PLAN §Phase 13
- ☐ **13.8 — EXIT** — `V` — Conformance suite green; schema
  and fixtures one release asset; `docs/cli/protocol.md` final; the CLI
  release candidate for GUI 4.8 (M7) — PLAN §Phase 13

#### Phase 14 — Swap, merge and release

- ☑ **14.1 — DONE** — `I1` — Moved to the split (desktop ADR-0001) and
  done 2026-11-14: the maintainer swapped the repositories; on `dev`,
  `master` merged in (CLI 0.3.0–0.5.0), the release lane on plain `v` tags
  on `master` claiming Latest, CI on pull requests into `master`, every
  old-address link moved, `CHANGELOG.md` the CLI's changelog, the archived
  1.x documents removed; then `dev` into `master` by one pull request. The
  maintainer re-tags the `cli-v` releases and deletes the 1.x releases after
  desktop 1.2.0 (M8) — PLAN §Phase 14
- ☐ **14.2 — EXIT** — `V` — The CLI release that carries the
  protocol, from `maelic13/colosseum-cli`, then desktop 2.0.0 pinned to it,
  with no bridge release (M9) — PLAN §Phase 14

### Phase 12 — CLI maintenance from adoption (beside Phases 11, 13 and 14)

- ☑ **12.1** — `I1` — Tooling notes from the first
  adoption: a regression test that a resumed tune's `spsa status` has a
  finite ETA (the ETA itself was fixed by carrying the run's elapsed time in
  the checkpoint); the resume note names completed and remaining units
  ("resuming: N of M games complete, K to play") instead of the
  post-checkpoint replay count, and `run.log` records the stop and the
  resume events its documentation promises; under `--json` the note stays
  on stderr and the JSON value also carries the resume facts; one
  regression test per item; `CHANGELOG-CLI.md` Unreleased — PLAN §Phase 12(a)
- ☑ **12.2** — `I1` — `spsa history`: the centre vector after
  every completed iteration, rebuilt from the journal, as a table, `--json`
  or `--csv` — PLAN §Phase 12(b)
- ☑ **12.3** — `I1` — `spsa --seed-from <run dir>`: a fresh
  tune starting from a finished tune's rounded final centres, same surface
  unless `--tune` is given — PLAN §Phase 12(b)
- ☑ **12.4** — `I1` — A warning, kept in the run record,
  whenever a command-line option list replaces a run file's list and drops
  values it named — PLAN §Phase 12(b)
- ☐ **12.5** — `I1` — An unparsable start position is refused
  instead of silently replaced by the standard one (from the 1.x
  inventory) — PLAN §Phase 12(c)
- ☑ **12.6 — DONE** — `I1` — The time to an SPSA stop point: with
  `--stop-after-iteration` below the total, progress reports, `run.log` and
  `spsa status` give the time left to the stop point beside the time left
  to the end; first of the CLI 0.6.0 requests. Evidence 2026-11-14: a
  synthetic tune stopped at 1, resumed to 3, then to the end asserts the
  `time to stop` line in the block and `run.log` and `eta.stop_point` in
  `spsa status`, and its absence without a stop — PLAN §Phase 12(d)
- ☐ **12.7** — `I1` — `winc` and `binc` from each side's own control, not
  the mover's increment for both (Rarog request 11) — PLAN §Phase 12(d)
- ☐ **12.8** — `I1` — A fixed match's pentanomial in the run record and its
  pentanomial, Elo and nElo in `results.txt` (Rarog request 12) — PLAN §Phase 12(d)
- ☐ **12.9** — `I1` — An EPD opening keeps its halfmove and fullmove counters
  instead of `0 1` (Rarog request 2) — PLAN §Phase 12(d)
- ☐ **12.10** — `I1` — Node-budget compliance per side in `results.txt` and the
  telemetry: the largest count and the moves over budget (Rarog request 13) — PLAN §Phase 12(d)
- ☐ **12.10.1** — `R2`, then `I2` — `spsa read`: per-coordinate gradient
  `z`, the familywise line, the cross-fitted gain read with its
  sign-randomised null, rails, clamps and an integrity check, measurement
  only (Basilisk request) — PLAN §Phase 12(d)
- ☐ **12.11** — `I2` — Moves-per-period time controls with `movestogo`, clock
  refills and the standard `TimeControl` tag (Rarog request 4) — PLAN §Phase 12(d)
- ☐ **12.12** — `R2`, then `I2` — SPSA without an iteration barrier, with exact
  replay, resume and the synchronous mode kept; reopens 10.9t (Rarog
  request 1) — PLAN §Phase 12(d)
- ☐ **12.13** — `I2` — Automatic placement gives a slot with a multi-thread
  engine one logical CPU per physical core (Rarog request 5) — PLAN §Phase 12(d)
- ☐ **12.14** — `I2` — Host load on the game CPUs sampled during the run,
  recorded and flagged (Rarog request 6) — PLAN §Phase 12(d)
- ☐ **12.15** — `I1` — Wait for an idle host before the first game, with a
  window, threshold and deadline (Rarog request 10) — PLAN §Phase 12(d)
- ☐ **12.16** — `I1` — The clock left on every move in the PGN comments (Rarog
  request 7) — PLAN §Phase 12(d)
- ☐ **12.17** — `I1` — The conversion read per engine in `results.txt`, equal to
  Rarog's `conversion_audit.py` (Rarog request 8) — PLAN §Phase 12(d)
- ☐ **12.18** — `R2`, then `I1` — The SPSA checkpoint verdict and
  self-stopping registered checkpoints on 12.10.1's reads; first the
  maintainer's decision whether that policy belongs in the CLI (Rarog
  request 9) — PLAN §Phase 12(d)
- ☐ **12.19** — `M` — Release archives for Linux ARM64 and Intel macOS, if
  wanted (Rarog request 3) — PLAN §Phase 12(d)

### Deferred and post-release (not steps until reopened)

- **10.9h** — Placement per platform research note (Linux isolated cores and
  IRQ affinity, macOS advisory contract, WSL excluded). Reopens on a Linux
  or macOS placement report, or before CLI 1.0.0 — [record](docs/architecture/phase-10-record.md), item (r)
- **10.9t** — Overlapped SPSA iterations, `--overlap 1`, off by default.
  Reopened as **12.12** on Rarog's RAR-M88 evidence (2026-11-14); taken
  before 10.9p — [record](docs/architecture/phase-10-record.md), item (ad)
- **10.9p** — Two games per physical core for `spsa` and `match` only, as a
  measured experiment. Reopens only if 10.9t leaves a margin worth it —
  [record](docs/architecture/phase-10-record.md), item (z)
- **10.9g probes** — the 2,000-game 3+0.03 scramble probe and the 100 ms,
  14-slot, 50,000-move outlier probe, maintainer-run, informative, recorded
  in `docs/architecture/phase-10-record.md` when done
- **Asynchronous SPSA** — research, needs its own evidence — PLAN §S8
  Post-release research

## Recurring procedures

Not steps — they are never "done".

### Declaring a platform supported

- Full test suite green there, debug **and** optimized
  (`cargo test --workspace --all-targets --profile ci-release`).
- Affinity, process, timer and filesystem capabilities or fallbacks implemented,
  tested and documented there.
- The exact released CLI artifact passes `--version`, `--help`, headless
  `self-test` and one deterministic JSON-mode workflow.

### After changing anything that runs games

- Re-run the Phase 4B oracle replay and controlled live parity on compatible
  shared fields; repeat the release-candidate matrix at Phase 8.1.
- Consider a real-machine calibration after material clock, scheduling or
  affinity changes; it is evidence, not a release or usage prerequisite.
- Bump the harness version in run records, and `stats_version` if any reported
  statistic changed definition — with a changelog entry.

### After completing a generic workflow

- Migrate the corresponding validation-engine harness workflows immediately.
- Compare old/new resolved inputs, schedule, durable artifacts and statistics.
- Archive the old generic implementation only after parity; retain declarative
  configs and thin project-policy/CI invocation.
- Record an exception as either a CLI mechanism gap or intentional
  engine-specific policy.

### Cutting a release (maintainer)

The procedure is [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) *Releases*;
in short:

- The release commit — the version in `crates/colosseum-cli/Cargo.toml`,
  `Cargo.lock`, the changelog's `[Unreleased]` dated — goes on a branch from
  `master`, and `cargo xtask release-check vX.Y.Z` passes there. Its pull
  request is titled "Release X.Y.Z".
- That pull request is the rehearsal: `CI complete` and `Verify the
  complete matrix` green, the candidate including the release check.
  Squash-merge it.
- Tag the merged `master` with an annotated `vX.Y.Z` and push only the tag;
  watch the release workflow; check the release page. Wrong release: delete
  release and tag, fix `master` by pull request, tag again.
- The next pull request that changes the product reopens it at the next
  version with `-dev`.
- A patch while `master` holds unreleased work is tagged on a branch from
  the last release tag, after the fix has landed on `master`.
- The CLI release that carries a protocol change is tagged first; the
  desktop release that needs it follows, bundling and pinning that CLI
  version. A protocol minor version never ships in a CLI patch release.

## What to do now

**The split is done; this repository is the CLI alone.** In order:

1. **CLI 0.6.0** — the engine projects' requests (Phase 12(d)): 12.6 (done),
   then 12.7–12.19 in order, 12.10.1 after 12.10. Small steps share a
   branch and one pull request, one commit each (12.6–12.10 on
   `engine-requests`); 12.10.1, 12.11, 12.12 and 12.14 have their own.
   Then the release pull request "Release 0.6.0" and the `v0.6.0` tag, over
   the steps the maintainer puts in it. It is also the first full run of
   the delivery path.
2. The desktop repository, [`maelic13/colosseum`](https://github.com/maelic13/colosseum),
   tracks its own steps in its `GUIDE.md`: 1.2.0 from `legacy/`, then GUI
   1.1.1–1.1.4 and 1.2. Once 1.2.0 is published, the maintainer deletes the
   1.x releases and tags here.
3. Here: 11.3 waits for GUI 1.2 (requirements signed off); 11.4 follows and
   unblocks the GUI screen specifications (GUI 3.x).
4. Here: Phase 13 in order. 13.2 unblocks the GUI performance proof (GUI
   2.3); 13.3 and 13.4 unblock the GUI live view (4.5); 13.4's lifecycle
   decision is taken together with GUI 4.2; 13.5–13.7 unblock GUI 4.6.
5. 13.8 produces the CLI release candidate, from a branch, that the GUI
   accepts against (GUI 4.8).
6. 14.2: the CLI release, then the GUI's 2.0.0 release pinned to it.

Each design step (GUI 1.1–1.4, 3.1–3.3) ends in a maintainer sign-off,
reviewed in pairs; no GUI screen is implemented before its specification is
signed off. Phase 12 steps are taken beside all of it as reports arrive.

```
git diff --check
```

## Working rhythm

```text
Pick the next ☐ step  ->  implement + test  ->  demonstrate its exit
criterion  ->  mark it ☑ here  ->  update PLAN.md when useful  ->  commit before
the next step.
```

Long game jobs (optional calibrations, parity runs, real gates) run on a real
machine and are pasted back; everything else is verified locally by tests.
Use a focused imperative commit subject, stage only the step's files and never
add co-author or assistant-attribution trailers; `AGENTS.md` is the binding
repository workflow.

## Decision rules

| Situation | Action |
|---|---|
| A phase "works" but its exit criterion is not demonstrated | Not done. Do not proceed |
| An inner layer needs a GUI/SQLite/Tokio/OS type | Stop and introduce or correct an application port/adapter; dependencies point inward |
| Our statistic disagrees with a compatible external oracle | Root-cause before shipping — one of them is wrong and it may be ours |
| External tools disagree or do not expose the same model | Record the matrix limitation; prefer analytic fixtures; never average or compare unsupported fields |
| OS cannot support a capability (e.g. hard macOS affinity) | Record the advisory/off fallback in the run record and PLAN; fail only if the capability was explicitly requested |
| Tempted to make one of our defaults mandatory | It belongs in PLAN §S3 Tier B with a reason, or Tier C as advice — Tier A needs a silent-wrong-number failure mode |
| Feature exists in an external runner but not here | Phase 8.2 decided the first set; "does a general engine developer need it?" is the tie-breaker for the next |
| Tempted to add engine-specific logic | It belongs in the engine's own tooling, not here |
| Engine project still needs scheduling/statistics/tuning/recovery code | Generic mechanism gap: add or explicitly decline it |
| Engine project keeps a run file or thin CI command | Expected project policy, not a CLI gap |
| Diagnostic heuristic looks stable | Report the observation; do not call it convergence or causation |
| A GUI behaviour differs from the CLI's for the same mechanism | The CLI's is the specification (PLAN §S5); Phase 11 removes the second implementation, so do not add a third |
| GUI v2 needs something the CLI does not provide | It is a CLI extension through the S5.15 protocol, recorded in the 11.3 gap analysis — never game-playing, rating or chess logic in the desktop application |
| Tempted to coordinate CPU placement across desktop tournaments | Placement is a CLI feature for gates and tunes; desktop tournaments run with it off |
| A protocol event could block on a slow reader | Not allowed: bounded queue, state dropped first, a `gap` fact instead of blocking (S5.15) |
| Tempted to decide the close / detach / re-attach behaviour early | Analysed and decided at 13.4 with GUI 4.2; until then every option must leave a resumable run |
| Tempted to implement a GUI v2 screen before its specification is signed off | Not allowed: design first (PLAN §Phase 11) |
