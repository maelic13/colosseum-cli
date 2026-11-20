# Colosseum CLI — development guide

<!-- board: generated from PLAN.md by `python tools/guide_board.py`; edit PLAN.md, never this block -->
**Now: 2** (`R2`, then `I2`): Refactor and simplify `colosseum-cli`.

## History — CLOSED

Phases 0–10 built, accepted and corrected the CLI and published CLI 0.1.0;
the engine projects' requests 12.1–12.11 followed, published as CLI
0.2.0–0.5.0 and, from 12.5, on `master` for 0.6.0. Evidence:
[Phases 0–9](docs/architecture/phases-0-9-record.md),
[Phase 10](docs/architecture/phase-10-record.md),
[requests 12.1–12.11](docs/architecture/requests-record.md), the phase exit
documents and the [ADRs](docs/architecture/adr/README.md).

## Requests

- [x] **1** Keep the CLI general — DONE 2026-11-20
- [ ] **2** Refactor and simplify `colosseum-cli` — **R2, then I2**
- [ ] **3** Moves-per-period time controls — **I2**
- [ ] **4** SPSA without an iteration barrier — **R2, then I2**
- [ ] **5** Placement for engines with more than one thread — **I2**
- [ ] **6** Host load, measured and reported — **R2, then I2**
- [ ] **7** The clock left on every move — **I1**
- [ ] **8** Release builds for Linux ARM64 and Intel macOS — **M**
- [ ] **9** Release CLI 0.6.0 — **M**

## Refused

- [x] **10** The conversion read in `results.txt` — REJECTED 2026-11-20
- [x] **11** SPSA stop verdicts and a checkpoint grid — REJECTED 2026-11-14
- [x] **12** Waiting for an idle host — REJECTED 2026-11-20
<!-- end of generated board -->

The short operational view of the requests plan: where the work stands and
what to do next. Rationale, specifications, success criteria and every step
in full live in [`PLAN.md`](PLAN.md); the evidence of completed work lives in
[`docs/architecture/`](docs/architecture/README.md). The desktop
application's programme is the separate pair [`PLAN_GUI.md`](PLAN_GUI.md)
and [`GUIDE_GUI.md`](GUIDE_GUI.md), under revision; nothing here waits for
it.

**This file and `PLAN.md` are the maintainer-facing pair.** `README.md` is the
user-facing front door; `docs/cli/` is the user-facing CLI detail. Neither may
carry step numbers, internal naming or method argumentation.

## Reading the board

The board above is generated from `PLAN.md`'s step heads by
`python tools/guide_board.py`, which writes `GUIDE_GUI.md`'s board too; never
edit it by hand. A step's title, class and status are written only in its
PLAN head, and the order of work in PLAN §S8 *Order of work*;
`python tools/guide_board.py --check` fails on a stale board, and CI runs it.
The grammar is in `AGENTS.md` *Documentation ownership*.

Requests are numbered 1, 2, 3, … in the order they are recorded and keep
their number, done or refused; the GUI plan's steps are lettered. The work
before this numbering — Phases 0–10 and the requests 12.1–12.11 — is the
board's *History*. A step is done when its exit criterion is demonstrated,
not because it compiles. When reporting the next step, always report its
class and the model *Current model mapping* below gives it.

## Current checkpoint

| | |
|---|---|
| Branch / versions | `master` holds **CLI 0.5.0** (2026-10-29), the latest release, and the unreleased 0.6.0 work; releases are plain `vX.Y.Z` tags (`v0.1.0`–`v0.5.0` re-tagged from `cli-v`). This repository is CLI-only since the split, 2026-11-14: `maelic13/colosseum-cli`, with the desktop application in `maelic13/colosseum`. Work lands on `master` as one squash pull request per piece |
| What exists | Phases 0–10 complete and published; requests 12.1–12.4 as CLI 0.2.0, then 0.3.0–0.5.0 from adoption; 12.5–12.11 and step 1 on `master` for 0.6.0: an unparsable start position refused, the time to an SPSA stop point, each side's own increment, a fixed match's paired score, EPD move counters, node-budget figures, `spsa read`, and the CLI kept general — figures, not verdicts |
| What is missing | Step 2, the refactor; Rarog's remaining requests, steps 3–8; then step 9, the 0.6.0 release, with every request in it |
| Validation engines | Two independently developed UCI engines in different languages and build systems. Any two UCI engines would serve; nothing depends on which |
| Platform status | Windows/Linux/macOS ☑ required debug and optimized CI on every pull request into `master` · Windows x86-64/ARM64, Linux x86-64 and macOS ARM64 release archives ☑ built, packaged and smoked as a candidate on every pull request, exactly as a tag builds them |
| Next step | The board's **Now** — step 2, the refactor (`R2`, then `I2`); then steps 3–8; the 0.6.0 release, step 9, last |
| Confirmed decisions | The 2026-09-22 set, recorded with the release preparation in [`phase-10-record.md`](docs/architecture/phase-10-record.md): exclusion for unspawnable engines, the xtask surface, versions kept in artifact names, the throughput margin not chased. The 2026-09-28 naming set and the 2026-11-14 split set, in [ADR-0012](docs/architecture/adr/0012-names-and-repositories-after-the-split.md): Colosseum and `colosseum-cli` kept; plain `v` tags with the `cli-v` releases re-tagged; Rarog's delivery adopted — short-lived branches, one squash pull request each with a short title and description, CI and the release candidate on every pull request; the Rust toolchain pinned (1.100.0), each update its own tested pull request. The 2026-11-14 SPSA read decisions (record 12.11): measurement in the CLI, stop verdicts in the projects. The 2026-11-20 set: *Numbers, not verdicts* (PLAN §S3); the plans split into requests and GUI and renumbered; the GUI programme under revision (`PLAN_GUI.md` step A) |

## Current model mapping

PLAN §S8 records each step's stable capability class; this table maps the
classes to the current Claude model and thinking mode and is the only place
model names appear. `PLAN_GUI.md` uses the same classes. Edit it when model
generations change; do not rewrite the roadmap. These are maintainer
judgements, not measured rankings.

| Class | Capability | Claude |
|---|---|---|
| `R3` | Frontier causal/architecture research | Claude Fable 5.1 — High |
| `R2` | Bounded correctness-sensitive reasoning | Claude Opus 5 — High |
| `I2` | Difficult implementation | Claude Opus 5 — High |
| `I1` | Well-specified implementation | Claude Sonnet 5 — Medium |
| `M` | Mechanical/docs/provenance | Claude Sonnet 5 — Medium |
| `V` | Verification/measurement | Claude Sonnet 5 — High |

Deferred items (10.9h, 10.9p, the 10.9g probes, asynchronous SPSA) are not
steps until reopened; they are listed in PLAN §S8 *Deferred*.

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

## What to do now

1. **Step 2, the refactor** — its `R2` part measures and proposes slices for
   the maintainer to accept; then the accepted slices, one commit each, with
   nothing observable changing.
2. **Steps 3–8**, Rarog's remaining requests, in the order of work. Small
   steps may share a branch and one pull request; 3, 4 and 6 have their own.
3. **Step 9, CLI 0.6.0** — once every request is in: the release pull
   request "Release 0.6.0" and the `v0.6.0` tag, the first full run of the
   delivery path. Until then, finished work lands on `master` by pull
   request and waits there for the release.
4. The GUI programme is decided apart, at `PLAN_GUI.md` step A.

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
| A step "works" but its exit criterion is not demonstrated | Not done. Do not proceed |
| An inner layer needs a GUI/SQLite/Tokio/OS type | Stop and introduce or correct an application port/adapter; dependencies point inward |
| Our statistic disagrees with a compatible external oracle | Root-cause before shipping — one of them is wrong and it may be ours |
| External tools disagree or do not expose the same model | Record the matrix limitation; prefer analytic fixtures; never average or compare unsupported fields |
| OS cannot support a capability (e.g. hard macOS affinity) | Record the advisory/off fallback in the run record and PLAN; fail only if the capability was explicitly requested |
| Tempted to make one of our defaults mandatory | It belongs in PLAN §S3 Tier B with a reason, or Tier C as advice — Tier A needs a silent-wrong-number failure mode |
| A request asks for a project's threshold or verdict | Refuse it as a step, and offer the figures it needs instead (PLAN §S3, *Numbers, not verdicts*) |
| Feature exists in an external runner but not here | "Does a general engine developer need it?" is the tie-breaker |
| Tempted to add engine-specific logic | It belongs in the engine's own tooling, not here |
| Engine project still needs scheduling/statistics/tuning/recovery code | Generic mechanism gap: add or explicitly decline it |
| Engine project keeps a run file or thin CI command | Expected project policy, not a CLI gap |
| A figure looks stable | Report it; do not call it convergence or causation |
| A request concerns the desktop application | It belongs to `PLAN_GUI.md`; its decision rules are in `GUIDE_GUI.md` |
