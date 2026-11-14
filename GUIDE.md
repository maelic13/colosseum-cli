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
| What is missing | **Phase 12(d)**, the engine projects' requests (12.7–12.18, from Rarog's list of 2026-11-13 and Basilisk's SPSA read request), released as **CLI 0.6.0** and later minors — 0.6.0 the first release through pull-request delivery. Then **Phase 11** (11.3 gap analysis and its decisions 11.3.1–11.3.5, the 11.4 protocol specification), **Phase 13** (the protocol, one CLI minor release) and **14.2** (the release pair); 12.5. In the desktop repository: GUI Phases 1–5 |
| Validation engines | Two independently developed UCI engines in different languages and build systems. Any two UCI engines would serve; nothing depends on which |
| Platform status | Windows/Linux/macOS ☑ required debug and optimized CI on every pull request into `master` · Windows x86-64/ARM64, Linux x86-64 and macOS ARM64 release archives ☑ built, packaged and smoked as a candidate on every pull request, exactly as a tag builds them |
| Next step | The board's **Now** — 12.7, `winc` and `binc` from each side's own control — `I1` (Claude Sonnet 5 — Medium); then 12.8–12.10, the other small corrections; then 12.11, the SPSA reads Basilisk and Rarog share, once its open questions are settled with the maintainer; then 12.12 and 12.13, Rarog's priorities; the 0.6.0 release pull request when the maintainer closes its scope. In the desktop repository: 1.2.0 from `legacy/`, then **1.1.1–1.1.4** and 1.2. 11.3 waits for GUI 1.2 |
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

The board below is generated from `PLAN.md`'s step heads by
`python tools/guide_board.py`; never edit it by hand. A step's title, class
and status are written only in its PLAN head, and the order of work in PLAN
§S8 *Order of work*; `python tools/guide_board.py --check` fails on a stale
board, and CI runs it. The grammar is in `AGENTS.md` *Documentation ownership*.

Each phase ends with a verifiable exit criterion — see PLAN §S8. Nothing is
"done" because it compiles; it is done when its criterion is demonstrated.
When reporting the next step, always report its class and the model the
mapping above gives it.

Phases 11, 13 and 14 are the GUI v2 programme; the GUI repository's Phases
1–5 are its other half. Both repositories advance together and release
together; the joint milestones M0–M9 are in PLAN §S8. A step here that names
a GUI step waits for it, and the GUI tracker says the same the other way.
Phase 12 runs beside them.

<!-- board: generated from PLAN.md by `python tools/guide_board.py`; edit PLAN, never this block -->
**Now: 12.7** (`I1`): `winc` and `binc` from each side's own control.

**Then:** 12.8, 12.9, 12.10.

### Phase 11 — Separation (this repository)

- ☑ **11.1** Naming — `R2`, DONE
- ☑ **11.2** The GUI repository seed — `M`, DONE
  - ☑ **11.2.1** Retire the egui application from `dev` — `I1`, DONE 2026-09-29
- ☐ **11.3** The CLI against the requirements — `R2`
  - ☐ **11.3.1** Decide engine option resolution — `R2`
  - ☐ **11.3.2** Decide ponder — `R2`
  - ☐ **11.3.3** Decide adjudication — `R2`
  - ☐ **11.3.4** Decide faults in tournaments — `R2`
  - ☐ **11.3.5** Decide scheduling — `R2`
- ☐ **11.4** Protocol specification — `R2`, EXIT

### Phase 13 — The front-end protocol (this repository, one CLI minor release)

- ☐ **13.1** Session — `I2`
- ☐ **13.2** Synthetic streams — `I1`
- ☐ **13.3** Live events — `I2`
- ☐ **13.4** Control and lifecycle — `R2`, then `I2`
- ☐ **13.5** Amendments — `I2`
- ☐ **13.6** Changed engines — `I1`
- ☐ **13.7** Queries — `I1`
  - ☐ **13.7.1** Tournament statistics — `I1`
  - ☐ **13.7.2** Tournament options for the application — `I1`
- ☐ **13.8** Conformance and candidate — `V`, EXIT

### Phase 14 — Swap, merge and release (both repositories)

- ☑ **14.1** Swap and merge — `I1`, DONE 2026-11-14
- ☐ **14.2** Releases — `V`, EXIT

### Phase 12 — CLI maintenance from adoption

- ☑ **12.1** Tooling notes from the first adoption — `I1`, DONE
- ☑ **12.2** `spsa history` — `I1`, DONE
- ☑ **12.3** `spsa --seed-from <run dir>` — `I1`, DONE
- ☑ **12.4** A command-line option list that replaces a run file's — `I1`, DONE
- ☐ **12.5** An unparsable start position is refused — `I1`
- ☑ **12.6** The time to an SPSA stop point — `I1`, DONE 2026-11-14
- ☐ **12.7** `winc` and `binc` from each side's own control — `I1`
- ☐ **12.8** A fixed match's score in `results.txt` and the run record — `I1`
- ☐ **12.9** An EPD opening keeps its move counters — `I1`
- ☐ **12.10** Node-budget compliance in the summary — `I1`
- ☐ **12.11** SPSA gradient and gain reads, and the checkpoint verdict — `R2`, then `I2`
- ☐ **12.12** Moves-per-period time controls — `I2`
- ☐ **12.13** SPSA without an iteration barrier — `R2`, then `I2`
- ☐ **12.14** Placement for engines with more than one thread — `I2`
- ☐ **12.15** Host load: recorded during the run, and an idle host before it starts — `R2`, then `I2`
- ☐ **12.16** The clock left on every move — `I1`
- ☐ **12.17** The conversion read in `results.txt` — `I1`
- ☐ **12.18** Release builds for Linux ARM64 and Intel macOS — `M`
<!-- end of generated board -->

### Deferred and post-release (not steps until reopened)

- **10.9h** — Placement per platform research note (Linux isolated cores and
  IRQ affinity, macOS advisory contract, WSL excluded). Reopens on a Linux
  or macOS placement report, or before CLI 1.0.0 — [record](docs/architecture/phase-10-record.md), item (r)
- **10.9t** — Overlapped SPSA iterations, `--overlap 1`, off by default.
  Reopened as **12.13** on Rarog's RAR-M88 evidence (2026-11-14); taken
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
   then 12.7–12.18 in the order of work. Small steps share a branch and one
   pull request, one commit each (12.6–12.10 on `engine-requests`); 12.11,
   12.12, 12.13 and 12.15 have their own, and 12.11 and 12.15 start with
   the maintainer's decisions on their open questions. Then the release
   pull request "Release 0.6.0" and the `v0.6.0` tag, over the steps the
   maintainer puts in it. It is also the first full run of the delivery
   path.
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

## Working rhythm

```text
Take the board's Now  ->  implement + test  ->  demonstrate its exit
criterion  ->  mark its PLAN head DONE <date> and drop its Order of work row
->  python tools/guide_board.py  ->  commit before the next step.
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
