# Colosseum agent instructions

Read this file, [`CLAUDE.md`](CLAUDE.md), [`PLAN.md`](PLAN.md), and
[`GUIDE.md`](GUIDE.md) before implementation work. `CLAUDE.md` describes the
existing application and conventions; `PLAN.md`, the requests plan, is the
binding CLI specification and the source of every numbered step; `GUIDE.md`
is its ordered tracker, its board generated from `PLAN.md`. The desktop
application's programme is the separate pair [`PLAN_GUI.md`](PLAN_GUI.md)
and [`GUIDE_GUI.md`](GUIDE_GUI.md), with lettered steps; read them for work
on it.

## Scope and architecture

- Implement steps in their plan's order of work (the board's Now first).
  Do not skip a step's exit criterion.
- The evidence of completed work lives under `docs/architecture/` (exit
  documents, ADRs, the Phase 0–9, Phase 10 and requests records). Do not
  re-derive or re-litigate what they record; extend them when a step adds
  evidence.
- The CLI measures and reports; a project applies its own thresholds
  (PLAN §S3, *Numbers, not verdicts*). Do not add a threshold or verdict
  that belongs to one project; offer the figures it needs.
- Dependencies point inward: domain → nothing outward; application use cases →
  domain and ports; adapters/drivers implement ports. The CLI is the one
  composition root; the desktop application lives in its own repository and
  would reach the CLI only through the process protocol of `PLAN_GUI.md`
  (ADR-0011).
- Preserve working UCI, runner, persistence, and compatibility behaviour.
  Prefer the smallest boundary refactor that satisfies the target architecture.
- The CLI accepts ordinary UCI executables. Do not add engine manifests, custom
  build/bench requirements, compiler inspection, or engine-specific logic.
- Treat existing engine crashes and protocol quirks as real supported input
  conditions. Never hide them by weakening diagnostics or fault classification.
- Tooling stays platform-independent although development is on Windows:
  builds, packaging and release checks are `cargo xtask`; any other new tool
  or script is Python 3 on the standard library, run as `python <path>`.
  Add no new PowerShell or shell scripts; the existing two are ported to
  Python when they next need a change.

## Step and commit discipline

One step of a plan is the normal unit of work.

Agents work locally only: create the required commits, but never push, fetch,
create tags, releases or pull requests, or perform other remote operations.
The maintainer owns every remote operation.

### Branches and pull requests

- `master` is the trunk; nothing else lives long, and nothing is pushed to it
  directly. Every coherent piece of work — a feature, a fix, a release, or
  one tooling or documentation change — lives on its own short-lived branch
  from `master` and lands as ONE squash commit through a pull request: no
  merge commits, no rebase merges. The full procedure, the required checks
  and the release steps are in [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)
  *Delivery* and *Releases*.
- On the branch, commit in small verified steps: one numbered step or one
  coherent change per commit, as below. These commits are working records;
  GitHub keeps them under the pull request (`refs/pull/<n>/head`) after the
  squash, and their subjects may name the step.
- Before the pull request opens, merge `master` into the branch if `master`
  moved, so the pull request shows only the branch's own change.
- The squash message — the pull request's title and description, which is
  all that reaches `master` — is short and simple: an imperative title that
  says what changed ("Report the time to an SPSA stop"; a release is
  "Release 0.6.0"), with no phase, step or plan references and no
  pull-request number (GitHub appends " (#N)" to a squash subject: remove
  it, and give `gh pr merge --squash` an explicit `--subject` whenever a
  merge is not done by hand); then a few plain lines on what and why,
  wrapped at 72 columns; then, after an empty line, the verification, one
  line each ("Tests: cargo test --workspace --all-targets, 1,204 passed").
- The Rust toolchain is pinned in `rust-toolchain.toml`. Never change it as
  part of other work: a toolchain update is its own pull request, "Rust
  X.Y.Z", with only the bump and the fixes its new lints need
  (`docs/DEVELOPMENT.md`, *The toolchain*).
- A user-visible change adds its entry under `CHANGELOG.md`'s
  `[Unreleased]` in the commit that makes it, written for users. A release
  only dates that section.

Each step's head in its plan records its capability class (`R3`, `R2`,
`I2`, `I1`, `M`, `V`), which the generated board shows; `GUIDE.md`'s
"Current model mapping" table is the single maintainer-edited source that
maps a class to the current Claude model and thinking mode. Whenever
reporting the next step, name it with its class and recommend the mapped
model and mode as `Claude: <model> — <mode>` with a brief task-specific
reason; do not substitute newer models, and never change the active model
automatically. If an `I1`, `M` or `V` step exposes a material design choice
not settled by the plan, stop and continue it as `R2` rather than inventing
the contract; never silently downgrade a recorded class.

1. Start from a clean worktree, or identify and preserve pre-existing user
   changes. Never stage unrelated files.
2. Implement the step, its tests, documentation, migrations, and generated
   files needed for that step.
3. Run the verification required by the step and the proportionate workspace
   checks from `CLAUDE.md`.
4. Demonstrate the exit criterion. A compiling change is not automatically
   complete.
5. In the same change, mark the finished step's head `DONE <date>` (or
   its other outcome), remove its row from its plan's *Order of work*, add
   the evidence to the step's text, and regenerate the boards with
   `python tools/guide_board.py` (*Documentation ownership* below).
6. Commit the completed step before starting another numbered step.

Use a short imperative commit subject that names the outcome, preferably with
the step identifier, for example:

```text
Send movestogo for moves-per-period controls (step 4)
```

Do not:

- combine independently completable steps in one commit;
- mark or commit an incomplete step as done;
- amend, rewrite, reset, or discard user commits/changes unless explicitly
  requested;
- add `Co-authored-by`, assistant attribution, or other authorship trailers;
- begin the next numbered step until the current step's commit succeeds.

If a step is blocked, leave it unchecked, document the blocker, and do not
create a misleading completion commit. Small corrective commits discovered
during verification are allowed, but each must have a precise single-purpose
subject.

## Verification baseline

Unless a step specifies more, use what CI enforces, in this order:

```text
cargo fmt --all --check
cargo check --workspace --tests
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

`cargo fmt --all --check` is not optional and is not cosmetic: `ci.yml` fails
the pull request on it, and a formatting failure is indistinguishable from a
real one in a release gate. The same applies to `--all-targets -- -D
warnings`, which lints the test code the shorter form skips.

Run debug and release/platform-specific checks where PLAN or GUIDE requires
them. Documentation-only changes require at least `git diff --check` and
consistency validation; they do not require rerunning unchanged Rust tests.

## Documentation ownership

- `PLAN.md` and `GUIDE.md`: the requests plan — the binding
  specification and the numbered steps — and its tracker. `PLAN_GUI.md`
  and `GUIDE_GUI.md`: the desktop application's programme and its tracker,
  with lettered steps. All four are maintainer-facing. A plan carries its
  specification and open work; finished work is summarised in a `— CLOSED`
  group, which the board shows, and recorded under `docs/architecture/`.
- Each guide's board is generated from its plan by `python
  tools/guide_board.py`; never edit it by hand, and run `python
  tools/guide_board.py --check`, which CI runs too, rather than reading
  it. A plan's steps live in the level-2 section that holds its `### Order
  of work`; every other level-3 heading there is a group, and a group
  whose heading ends `— CLOSED` shows its first paragraph instead of steps.
  A step head is a list item whose bold run reads
  ``**<ID> <title> — <field>.**``, wrapped if need be. `PLAN.md` numbers its
  steps (`4`, a sub-step `4.1`) and `PLAN_GUI.md` letters them (`C`, a
  sub-step `C.1`); a step keeps its ID once given, done or refused, and a
  new request takes the next number. The title is what the board shows; the
  field names the classes in backticks ("`R2`, then `I2`" for a two-part
  step) and may say EXIT, IN PROGRESS, or an outcome — DONE, REJECTED,
  DEFERRED, PARKED, FIXED or CLOSED — with an optional date. *Order of
  work* lists the open steps in the order they are taken, one `| <ID> |
  <waits for> |` row each, and its first row is the board's Now. Requests
  that overlap are one step, with their open questions settled with the
  maintainer before its code; a request for one project's threshold or
  verdict is refused as a step that keeps its number.
- `README.md`, `packaging/cli/README.md` (staged as the CLI archive's `README.md`) and
  `docs/cli/`: user-facing; no step numbers or internal method
  argumentation.
- `docs/DEVELOPMENT.md`: implemented build/test/release facts; update it when
  the workspace or release process actually changes, not merely when planned.
- `CHANGELOG.md`: user-visible changes, `[Unreleased]` first; each release's
  section is its release notes.
