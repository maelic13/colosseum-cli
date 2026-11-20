# Architecture and evidence

Maintainer-facing. The binding specifications are in [`PLAN.md`](../../PLAN.md),
the requests plan, with its tracker [`GUIDE.md`](../../GUIDE.md); the GUI
programme is [`PLAN_GUI.md`](../../PLAN_GUI.md) with
[`GUIDE_GUI.md`](../../GUIDE_GUI.md). This directory holds what they point
at: the decisions, the definitions and the evidence that each step met its
exit criterion. The analyses of the repository before the split
(`current-state.md`, `dependency-inventory.md`, `target-architecture.md`,
`release-architecture.md`, `naming-decision.md`) and the GUI v2 research
(`gui-v2-research.md`) were removed on 2026-11-20; they are in the
repository's history at `56cbd00`, and their decisions are in the ADRs.

## Architecture

| Document | What |
|---|---|
| [`fastchess-mechanics.md`](fastchess-mechanics.md) | Source study of fastchess's process, clock and pipe mechanics, with adopt/decline verdicts |
| [`spsa-read-definitions.md`](spsa-read-definitions.md) | The definitions `spsa read` computes, pinned against the engine projects' reference reader |
| [`adr/`](adr/README.md) | Architecture decision records |

## Records and acceptance evidence

| Document | What |
|---|---|
| [`phases-0-9-record.md`](phases-0-9-record.md) | The PLAN text of Phases 0–9 as implemented: progress, accepted results, decisions |
| [`phase-10-record.md`](phase-10-record.md) | Every Phase 10 item with rationale and implementation evidence, including the maintainer decisions |
| [`phase-10-qualification.md`](phase-10-qualification.md) | Real-engine qualification: scale, verdict, ratings, tuning recovery and symmetry figures |
| [`requests-record.md`](requests-record.md) | The engine projects' first requests, 12.1–12.11, with their evidence |
| `cli-0.3.0-record.md` … `cli-0.5.0-record.md` | What each CLI release changed and why |
| [`phase-0-review.md`](phase-0-review.md) | Phase 0 review of the architecture and release design |
| `phase-2-exit.md` … `phase-8-exit.md` | Exit evidence per phase |
| [`phase-8-parity.md`](phase-8-parity.md), [`phase-8-gap-decisions.md`](phase-8-gap-decisions.md) | Parity against external runners; feature-gap decisions |
| [`phase-9.4-candidate.md`](phase-9.4-candidate.md), [`phase-9.5-coverage.md`](phase-9.5-coverage.md), [`phase-9.6-usability.md`](phase-9.6-usability.md), [`phase-9.7-release-acceptance.md`](phase-9.7-release-acceptance.md) | Candidate identity, coverage acceptance, usability exercise, release acceptance |

The recorded parity commands and per-phase acceptance manifests are in
[`../fixtures/`](../fixtures/); the vendored statistics fixtures and their
generator are in `tests/fixtures/`.
