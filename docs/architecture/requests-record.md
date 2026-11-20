# Requests record: 12.1–12.11

The engine projects' first requests, numbered 12.1–12.11 as Phase 12 of the
plan before it was split (2026-11-20), moved here verbatim when they were
finished. The requests plan, [`PLAN.md`](../../PLAN.md) §S8, numbers its
steps afresh from 1; its step 1 is the first after these.

## Phase 12 — CLI maintenance from adoption

Corrections and small additions reported by projects using the released CLI.
This phase runs beside Phases 11, 13 and 14, not after them: a step here is small, changes
no statistic and breaks no run-directory or run-file format, lands on
`master` by its own pull request, ships in the next CLI release, and never
waits for GUI work. Each report becomes
one numbered step with its regression tests. The version follows the
changelog section the step lands in: fixes only are a patch (`cli-v0.1.x`);
a new command or option is a minor release (`cli-v0.2.0` carries 12.1–12.4,
because 12.2 and 12.3 add `spsa history` and `--seed-from`); anything that
would change a reported statistic or break the run-file schema or the
run-directory layout is a minor release with its own step, said in the
changelog, and never lands as a patch. Added fields — a checkpoint's
`run_elapsed_seconds`, a JSON value's `resume`, a run record's
`seeded_from` — are compatible in both directions and do not count as
format changes.

- **(a) Tooling notes from the first adoption** (GUIDE 12.1, `I1`), reported
  2026-09-22; nothing blocks.
  - **12.1 Tooling notes from the first adoption — `I1`, DONE.** The
    three items below, shipped in CLI 0.2.0.
  - **`spsa status` ETA after a resume — resolved 2026-09-25, before this
    step.** Every checkpoint now records the run's total active time across
    invocations (`run_elapsed_seconds`), progress blocks and the SPRT total
    show it, and `spsa status` computes its whole-run ETA from it, so a
    resumed tune has an ETA without a separate since-resume basis. Rates and
    `time remaining` in progress blocks still use the current invocation
    only. What remains for this step is a regression test that a stopped
    and resumed synthetic tune shows a finite ETA in `spsa status`.
  - **The resume note and the log.** On resume, `match` (and every other
    durable command with the same note) prints "resuming 0 durable game(s)"
    because it reads the units replayed from the journal after the last
    checkpoint, not the units the run has completed. The note names the
    completed units and the remaining ones: "resuming: 1,240 of 2,000 games
    complete, 760 to play". `run.log` gains the two events its
    documentation already promises in `run-directories.md` — one line at a
    clean stop (units complete, exit code) and one at resume (units
    complete, units to play) — written through the existing human-events
    path. A test stops and resumes a stub run and asserts both the note and
    the two log lines.
  - **`--json` carries the resume facts.** The human resume note stays on
    stderr in `--json` mode — kept at the user's request (2026-09-25),
    which the maintainer agreed to — and the JSON value also
    carries the facts as a `resume` object (`unit`, `completed_units`,
    `remaining_units`), so a wrapper need not parse stderr. The
    JSON-contract tests assert that stdout is exactly one JSON value.
  - `CHANGELOG-CLI.md` under Unreleased, then `[0.1.1]` when cut; the
    command reference regenerated if any help text changed; the run-file
    schema, run-directory layout and every statistic unchanged.

**Exit criterion:** the three regression tests pass; `colosseum-docs --check`
and the JSON-contract tests pass; a stopped and resumed synthetic tune shows
a finite ETA in `spsa status`.

- **(b) Notes from a production tune and gate** (GUIDE 12.2–12.4, `I1`),
  reported 2026-09-25 against `cli-v0.1.0`; the runner's SPSA and
  SPRT were validated at production scale (a 150,000-game tune, +13 Elo, no
  faults). The resume items of that report are (a) above.
  - **12.2 `spsa history` — `I1`, DONE.** A read-only command that prints the centre
    vector after every completed iteration, rebuilt from the journal
    exactly as a resume replays it, so it works while a tune is running
    and a trajectory read at a fixed iteration comes from an artifact.
    Columns are the tune's knob names; text table by default, one JSON
    value with `--json`, CSV with `--csv`; `--every N` thins the rows,
    always keeping the last. `result.json` is unchanged. A test tunes a
    synthetic run and asserts the rows equal the centres the driver
    committed, in knob order.
  - **12.3 `spsa --seed-from <run dir>` — `I1`, DONE.** Starts a new tune from a finished
    tune's final centres, rounded as the engine received them, as the new
    initial values, keeping the previous tune's surface (names, bounds,
    steps) unless `--tune` is given, in which case the knob names must
    match exactly. The schedule, seed and horizon are the new command's,
    so it is a fresh tune, recorded with the source run's result identity.
    Only a completed run seeds: a stopped, cancelled or invalid one is
    refused with the reason. Tests: a seeded tune starts from the source's
    rounded centres, and each refusal.
  - **12.4 A command-line option list that replaces a run file's — `I1`,
    DONE.** The
    documented rule stays: an explicit command-line option replaces the
    run file's value, arrays included. When that drops values the run
    file named — a side's `--option` list losing `Hash` and `Threads` —
    the resolver warns on stderr naming the option and each dropped value,
    and the run record keeps the warning, so the replacement is never
    silent. No warning when nothing was dropped. A test resolves a run
    file with three options against a one-option command line and asserts
    the warning and the recorded note.
- **(c) Findings from the 1.x inventory** (GUIDE 12.5, `I1`), GUI step 1.1,
  2026-09-29; latent, not reported by an adopter.
  - **12.5 An unparsable start position is refused — `I1`, DONE
    2026-11-18.** The runner falls back to the standard start position when
    a game's start FEN does not parse, while the engines are still sent that
    FEN. Books are validated at load, so no run reaches it today; the
    fallback becomes an error naming the game and the FEN. A test hands the
    runner an invalid FEN. Taken before 12.9, which changes how a start
    position is read, so a mistake there is an error, not a game from the
    standard position. Done: the runner reads the start position before it
    starts or reuses an engine; a FEN that does not parse, or parses to an
    illegal position, ends the game unscorable with the infrastructure fault
    `start-position` ("game N: the start position `FEN` is not a legal
    position: …"), so the run stops as an infrastructure error, and kept
    engines are quit. The engine crate's `start_position` test hands the
    runner both kinds with absent engine executables and finds that fault,
    not `engine-spawn`, as the old fallback gave. Nothing that plays a
    parsable position changed, so no replay or parity check applies.
- **(d) Requests from the maintainer's engine projects, for CLI 0.6.0**
  (12.6 onward), 2026-11-14. The maintainer takes these before the GUI
  programme continues and releases them as 0.6.0 and the minor releases
  after it; 0.6.0 is the first release through pull-request delivery and
  plain `v` tags, so the whole path — pull request, candidate, tag,
  publication — is exercised on real work. The intake is Rarog's request
  list, `analysis/colosseum_requests_2026-11-13.md` in the Rarog
  repository (thirteen requests, each proved from 0.5.0's artifacts), and
  Basilisk's SPSA read request of 2026-11-14. Requests that overlap are one
  step (Basilisk's read and Rarog's request 9; Rarog's requests 6 and 10),
  so the fourteen requests are the twelve steps 12.7–12.18, numbered in the
  order of work; each names its requests, whose evidence stays in the
  requesting repository. The order: four small corrections first, which
  are also all Rarog's next oracle read (RAR-O09) needs from the CLI; then
  the SPSA reads Basilisk and Rarog share; then Rarog's two priorities,
  moves-per-period clocks and barrier-free SPSA; then the measurement
  guards; then the reporting conveniences; the release platforms last. A
  merged step's open questions are settled with the maintainer before its
  code. A step that changes what an engine receives or how a game is
  played (12.7, 12.9, 12.12, 12.13, 12.14) repeats the oracle replay and
  parity checks and says so in the changelog. Which steps ship in 0.6.0 is
  the maintainer's choice; small steps may share one branch and one pull
  request, a large one (12.11, 12.12, 12.13, 12.15) has its own.
  - **12.6 The time to an SPSA stop point — `I1`, DONE 2026-11-14.** When
    a tune runs with `--stop-after-iteration N` below its total (stop at
    2,000 of 6,000 iterations), every progress report — the console block
    and its `run.log` line — and `spsa status` give the time left to the
    stop point as well as the time left to the end of the tune, both from
    the same observed rate. Without a stop point, or once the stop point is
    the end, the report is unchanged. A test runs a synthetic tune with a
    stop point and asserts both figures in the progress output, `run.log`
    and `spsa status`, and that a resumed run with a new stop point reports
    the new one. Done: the block's `time to stop` line sits just above
    `time remaining` and reads `<time>, at iteration N`; `spsa status`
    takes the stop from the latest invocation's run record and reports it
    as `eta.stop_point`. Evidence: a synthetic tune stopped at 1, resumed
    to 3, then run to the end asserts the line in the block and `run.log`
    and `eta.stop_point` in `spsa status`, and their absence without a
    stop.
  - **12.7 `winc` and `binc` from each side's own control — `I1`, DONE
    2026-11-14.** Rarog request 11. The runner's `move_limits` sent the
    mover's own increment as both `winc` and `binc`, so with a clock per
    side the engine to move is told its opponent has its own increment; the
    remaining times are right. Send `winc` from White's control and `binc`
    from Black's. Equal clocks are unchanged. A test plays 10000+100 against
    11270+113 and asserts `winc 100 binc 113` on every `go`, reversed when
    the colours are. Done: `move_limits` takes both sides' controls, the
    mover's own deciding the kind of limit; the runner's unit test and the
    CLI test `clock_increments` (the fixture's command log, both colours,
    failing on the old code with `winc 100 binc 100`) cover it. Equal clocks
    send what they did. The parity matrix plays depth 1, whose `go` is
    unchanged, and the oracle replay is the required suite's statistics
    fixtures, which pass.
  - **12.8 A fixed match's score in `results.txt` and the run record — `I1`,
    DONE 2026-11-14.** Rarog request 12. `match` writes
    `official_sample.pentanomial` as five zeros although its checkpoint
    holds the counts, and `results.txt` has no pentanomial, Elo or nElo
    line. Fill the run record's counts from the paired sample and add the
    pentanomial, Elo and nElo with their 95% intervals to `results.txt` as
    `sprt` reports them. Nothing changes definition. A test asserts the run
    record's counts equal a recount of the PGN by pair and that
    `results.txt` carries the figures the progress output printed. Done:
    `results.txt` and the run record's `results` gain a paired score
    (`results.paired`) recounted from the journal by pair as `stats` pairs
    games, so `sprt` and `calibrate` carry it too; `match` fills
    `official_sample` (pentanomial, complete pairs, unpaired games) from the
    paired sample its final report prints. The CLI test plays a scripted
    three-opening match whose pairs score 0, 1 and 2 and compares the run
    record with a PGN recount and `results.txt` with the recorded progress
    block; `stats_replay` checks the recount leaves out post-terminal and
    unscorable games.
  - **12.9 An EPD opening keeps its move counters — `I1`, DONE 2026-11-18.**
    Rarog request 2. `epd_fen_into` keeps an EPD line's four position fields
    and always appends `0 1`, which shortens a ply-aware engine's thinking
    on move 1 and moves the 50-move rule. Two trailing integers after the
    four fields and before any opcode are the halfmove clock and the
    fullmove number; the `hmvc` and `fmvn` opcodes are taken when present;
    otherwise `0 1`, as now. The resolved configuration records where the
    counters came from. This changes what every run from such a book plays.
    A test asserts `… w KQkq - 2 4` reaches the engine as `position fen … 2
    4`, and each source and the default. Done: `epd_fen_into` takes two
    integers after the four fields as the counters, else the `hmvc`/`fmvn`
    operations (a `;` inside a quoted operand ends none; either alone
    defaults the other), else `0 1`, keeping the counters as written; the
    loaded list counts each source, and `OpeningPolicyReport::Book` records
    them as `move_counters` (EPD only), so the resolved configuration of
    every EPD run changes and an earlier version's interrupted EPD run is
    refused on resume. `book slice` now writes whole FENs, since a shard of
    four fields lost the counters this step keeps. The CLI test plays a
    three-line book (`… w KQkq - 2 4`, opcodes with a quoted `hmvc`,
    neither) and reads `position fen … 2 4`, `… 2 3` and `… 0 1` from both
    engines' command logs and the counts from `resolved-config.json`; a
    second slices the book and finds the counters; the engine unit test
    covers each source, defaults and chunked parsing. On the old loader the
    CLI test reads `0 1` for all three. The parity matrix plays no book, and
    the oracle replay is the required suite's statistics fixtures.
  - **12.10 Node-budget compliance in the summary — `I1`, DONE 2026-11-18.**
    Rarog request 13. With a node budget, `results.txt` and the telemetry
    give each side's largest reported node count and the number of moves
    that reported more than that side's budget, so an engine that overshoots
    or ignores its budget is visible. Tests: a fixture engine that ignores
    its budget is named with its count of moves over; a compliant run
    reports none. Done: the runner writes `WhiteNodeBudget` /
    `BlackNodeBudget` tags for a side whose control is a node count, and the
    telemetry holds each post-opening move's `n=` against its own game's
    budget: `node_budget` (budget, moves under it, moves that reported
    nodes, largest count, moves over), printed by `results.txt` and `stats`.
    The fixture's `--reported-nodes` makes a side ignore its budget; the CLI
    tests play 1,000-node matches where one side reports 5,000 (8 of 8 moves
    over, named in `results.txt`, the run record and `stats`) and where both
    comply (none over), and fail on the old code with no `node_budget`; a
    unit test covers a budget per game, book moves and moves without `n=`.
    Nothing a game plays changed, so no replay or parity check applies.
  - **12.11 SPSA gradient and gain reads — `R2`, then `I2`, DONE
    2026-11-18.** Basilisk's request and the measurement half of Rarog
    request 9. Both projects judge whether a tune still has gradient with
    their shared `tools/spsa_read.py`, which regenerates the runner's
    ChaCha12 sign stream and re-derives every move from the schedule, so it
    depends on the CLI's internals and would break silently if either
    changed. The runner owns the signs, the schedule and the centres, so it
    computes the reads. Basilisk asks for the numbers only; Rarog also asks
    for a verdict and for tunes that stop themselves.

    Basilisk's request, the measurement: a read-only command, safe while a
    tune runs as `spsa history` is, `spsa read <run-dir> [--from K0]
    [--to K1] [--json] [--null-reps N] [--null-seed S] [--gain-segment G]
    [--forecast-span F]`; the window defaults to the last 1,000 completed
    iterations (all, if fewer), then 40, 1, 500 and 2,000. With
    `delta[k][i]` the sign drawn for coordinate `i` at iteration `k` and
    `D[k]` the plus side's wins minus losses:
    - per coordinate `z[i] = Σ delta[k][i]·D[k] / sqrt(Σ D[k]²)` (mean 0,
      variance exactly 1 without gradient, because the runner drew the
      signs), and the Elo across one perturbation, `695 / (2 ×
      games_per_iteration)` per unit of mean `D`, with its standard error;
    - the familywise line for `n` coordinates at family alpha 0.05,
      `Φ⁻¹(1 − (1 − (1 − α)^(1/n)) / 2)`, the largest `|z|` and the
      coordinates over it; the excess `Σ z² − n` in units of its null SD
      `sqrt(2n)`;
    - the cross-fitted gain read: in each segment of `G` iterations, the
      gradient estimated on the even iterations times each coordinate's
      actual move summed over the odd ones, and the reverse, summed over
      coordinates, in Elo, since iteration 0 and in the window, each with
      the SD of `N` seeded re-draws with every sign randomised;
    - an optional forecast, labelled an upper bound: the next `F`
      iterations' gain from the window's excess held constant through the
      remaining schedule (`ratio_k × c_k²` against the window's mean
      `c_k²`), with its null SD;
    - rails: a centre within one current perturbation of a bound while its
      `z` points outward, with the distance, the `z` and the share of
      iterations on which either arm was clamped; the run's own fault
      counters;
    - integrity: every move re-derived as `r_k · c_k · D · delta` (clamped
      where it was) against the journal, the worst deviation reported; a
      mismatch is an error, not a warning.

    Text is a summary block and one row per coordinate (name, `z`, Elo
    across a perturbation, centre, start, rounded value, clamp share, rail
    flag); `--json` is the same under schema `colosseum-spsa-read-v1` with
    stable field names; exit 0 for a read, non-zero for an unreadable or
    inconsistent run. No change to `spsa` runs, `spsa history` or the
    run-directory format; the user documentation and the changelog state
    the sign-stream and schedule definitions the read relies on, so a
    change to either becomes a versioned break.

    Rarog's request 9, the policy: `spsa status` prints the checkpoint
    verdict of Rarog's per-coordinate gradient test — quiet, live, a centre
    pushed outward at a bound, or faults — and a run file registers
    checkpoints (iteration 2,000, then every 1,000), so a tune stops itself
    at a quiet one and continues on a live one. Basilisk asks for the
    opposite: no checkpoint grid, stop verdict or project threshold in the
    CLI, the caller reading `spsa read --json` and pausing with
    `--stop-after-iteration`.

    Decided with the maintainer, 2026-11-14:
    - *What is general goes in the CLI; what is not stays in the projects.*
      The window's live/quiet test is general: a test of "no gradient in
      the window" with a known null, like LOS. Live means some `|z|` reaches
      the familywise line or the excess reaches its line; both lines are
      stated parameters (family alpha 0.05, excess 2 null SD by default).
      It is reported with what quiet rules out (no coordinate worth more
      than the line times its standard error, and the forecast's upper
      reading). A stop verdict is not general. Read at a checkpoint grid,
      it becomes a sequential decision whose error rates depend on the grid
      and are computed by nobody. The ceiling, the acceptance of faults, and
      what a rail or a stop means (writing theta, registering a range again)
      are project process. `spsa status` already promises never to advise
      continuing or abandoning a tune. So the CLI has no checkpoint grid,
      CONTINUE/STOP verdict or self-stopping run file. Rarog keeps its rule
      7c verdict in its wrapper, reading `spsa read --json` and pausing
      with `--stop-after-iteration`. Request 9's policy half is declined
      here, and its measurement half is this step.
    - *A new command.* `spsa read`, with its own versioned schema;
      `spsa status` stays the lifecycle view.
    - *The reference.* `spsa_read.py` at a pinned revision, with its
      `test_spsa_read.py` cases and the runs Basilisk exports committed as
      fixtures, reproduced to the digit. The `R2` part pins each definition
      against them before any code.

    Pinned 2026-11-18, the `R2` part, in
    [`spsa-read-definitions.md`](docs/architecture/spsa-read-definitions.md):
    each definition against Basilisk's `spsa_read.py` at `5f61c94`, whose
    29 tests pass there, the five recorded blocks among them. Decided with
    the maintainer the same day:
    - *The null draws are the CLI's.* A new named stream, `spsa-read-null`,
      from `--null-seed`, not Python's `random`: every other figure agrees
      with the reference to the digit, the null SD within sampling error,
      and the fixtures pin the CLI's own null values beside the
      reference's.
    - *No rail candidate.* The reference's outward `z` of 2.0 under the
      line is a project threshold; the CLI reports each coordinate's
      outward `z` and distance to its bound, and the rail at the line.

    Settled by the definitions, as the request and the decisions of
    2026-11-14 imply: the window may be shorter than 1,000 when fewer have
    completed; `Φ⁻¹` is Wichura's AS 241, as Python's `NormalDist` (the
    core's bisection is off by about 5e-5 at these tails);
    `--family-alpha` and `--excess-sd` state the two lines; the JSON is
    unrounded; the integrity check compares the replay with the centres
    the checkpoint and a finished run's `result.json` keep; exit 0, 2
    (usage), 1 (unreadable) and 3 (inconsistent).

    Tests: golden vectors on small synthetic runs, a planted gradient
    reading live and over the line and none reading quiet, with `z`
    variance about 1 over many seeds; the gain read's mean about 0 over
    sign-randomised runs; a tampered journal or a wrong seed failing
    loudly; the agreement fixtures. 12.13 keeps the reads working on a
    barrier-free tune.

    Done: `spsa read` (schema `colosseum-spsa-read-v1`) computes the read in
    `colosseum_core::spsa_read` from the run's own replay, signs and
    coefficients, with `Φ⁻¹` by AS 241 (`inverse_normal_cdf`) and the
    `spsa-read-null` stream, after comparing the replay with the
    checkpoint's and `result.json`'s centres. `tools/spsa_read_fixtures.py`
    exports Basilisk's five recorded tunes (b33, b46, b54, b23 blocks 1 and
    2) with the reference's figures at two windows each; the agreement test
    replays every centre exactly and matches every deterministic figure at
    the reference's rounding, and the null SDs within sampling error (the
    widest gap is the reference's own draw: its 200-draw estimates bracket
    the CLI's) and to committed values. Core tests: AS 241 against Python's
    `NormalDist`, the familywise lines, the forecast floor table, the
    gradient with zero scores, a planted gradient live and over the line, no
    gradient quiet, a rail at a bound, `z` variance 1 and the gain's mean 0
    over 120 seeds, the window and the null seed. CLI tests read a synthetic
    tune finished, mid-run and over a window, and refuse altered centres
    (exit 3), a changed journal or seed, and a window past the run (exit 2).
    The check found that an unoptimised build computed `c.powi(2)` a bit
    differently from the release runner; `SpsaSchedule::coefficients` and
    the schedule derivation now write `c * c`, which a release build already
    computed, so a release runner's moves are unchanged: the five tunes
    replay bit for bit in both builds. Read on Basilisk's b54 run directory,
    in 0.9 s, the command reproduces the reference's figures and leaves the
    directory untouched.
