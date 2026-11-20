# Colosseum — GUI plan

The desktop application, Colosseum, is planned as a separate client of
`colosseum-cli` in its own repository,
[`maelic13/colosseum`](https://github.com/maelic13/colosseum), driving the
CLI as a chess GUI drives a UCI engine
([ADR-0011](docs/architecture/adr/0011-desktop-application-as-separate-cli-client.md)).
This plan holds what that programme needs from the CLI: its requirements and
joint milestones (G1), the front-end protocol (G2) and the CLI's steps (G3).
The GUI's own steps are tracked in the desktop repository. The board is
[`GUIDE_GUI.md`](GUIDE_GUI.md), generated from this file by
`python tools/guide_board.py`.

**Status: under revision (2026-11-20).** The maintainer is deciding whether,
and in what form, the programme continues. Step A decides it, and no step
after it starts first. The CLI's general work is the requests plan,
[`PLAN.md`](PLAN.md), and never waits for this one.

**Numbering.** Steps are lettered, A onward, so they cannot be mistaken for
the requests plan's numbered steps. Before the plans were split
(2026-11-20) these were Phases 11, 13 and 14 of one plan: B was 11.3, B.1–B.5
were 11.3.1–11.3.5, C was 11.4, D–K were 13.1–13.8 (J.1 and J.2 were 13.7.1
and 13.7.2), and L was 14.2. Records and commits before that date use the
old numbers.

**Document audiences.** This file and `GUIDE_GUI.md` are maintainer-facing,
as `PLAN.md` and `GUIDE.md` are.

---

## G1. The programme

The desktop application is re-designed and re-implemented from scratch as a
separate product in its own repository, and this repository becomes the CLI
alone. The decision, the ownership split and the provisional technology are
[ADR-0011](docs/architecture/adr/0011-desktop-application-as-separate-cli-client.md);
the research behind it is
`docs/architecture/gui-v2-research.md` (in the repository's history at
`56cbd00`). This replaces the earlier Phase 11 (a `colosseum-harness` library crate with the egui
application moved onto it), which was never started: the duplicate
game-playing mechanism is removed by retiring the egui application, not by
extracting a library for it. The first cut of this programme (11A–11E,
2026-09-27) was reviewed the same day and re-cut into the phases below; no
step of it had started.

The work is split across two repositories that advance together and release
together:

| Steps | Where | What |
|---|---|---|
| A | this repository | the programme revised and decided |
| B–C — Separation | this repository | the CLI gap analysis and its decisions, the protocol specification (naming, the GUI repository seed and the swap are done) |
| D–K — Protocol | this repository | the protocol implemented in the CLI; one CLI minor release |
| L — Release | this repository | the CLI release that carries the protocol |
| Requests | this repository | the requests plan, `PLAN.md`, beside everything |
| GUI 1 — Design foundations | GUI repository | inventory of 1.x, requirements, layout concepts, design system |
| GUI 2 — Repository and technology proof | GUI repository | skeleton, component gallery, performance proof |
| GUI 3 — Screen specifications | GUI repository | every screen against the protocol and the gallery |
| GUI 4 — Implementation | GUI repository | the application, packaging, acceptance |
| GUI 5 — Release | GUI repository | the first release, pinned to the CLI release of L |

The GUI repository, [`maelic13/colosseum`](https://github.com/maelic13/colosseum) (created as `maelic13/colosseum-gui`), was seeded
with its `AGENTS.md`, `PLAN.md` and `GUIDE.md` from this repository at step
11.2 and is the tracker for its steps. This section keeps the CLI steps
in full and the joint milestones, so the programme reads in one place.

**Joint milestones** — the order both trackers follow:

| | Milestone | This repository | GUI repository |
|---|---|---|---|
| M0 | Names decided | 11.1 | — |
| M1 | GUI repository exists | 11.2 hands over the seed | 1.1 and 2.1 start |
| M2 | Requirements signed off | B gap analysis starts | 1.2 |
| M3 | Protocol specified | C | 3.1–3.3 reference it |
| M4 | Synthetic streams available | E | 2.3 performance proof |
| M5 | Live events and control | F, G | 4.5 live view on the real CLI |
| M6 | Amendments, changed engines, queries | H–J | 4.6 results, history, amendments |
| M7 | CLI release candidate | K, a candidate build | 4.8 acceptance against the candidate |
| M8 | Swap and merges (done 2026-11-14, at the split) | 14.1: the repository swap, then `dev` into `master` | took `maelic13/colosseum`; 1.x in `legacy/` as 1.2.0 |
| M9 | Releases in quick succession | L: the CLI release | 5.1, pinned to it |

**Maintainer requirements (2026-09-27, revised after the same day's
review), binding for every step in both repositories:**

- **Ratings (2026-09-29).** The CLI computes every tournament's ratings —
  the joint ML fit, error bars and performance ratings — from the starting
  ratings and pins the run file carries (`--rating`, `--fixed`, `--anchor`),
  and publishes a rating snapshot after every scored game. The desktop
  application owns the engine library and its ratings: it passes each
  engine's library rating as the starting rating, pins the engines a
  tournament must not move (its *Chosen* mode pins the rest), and writes the
  CLI's number into the library after each scored game. It never computes a
  rating.

- **Design first.** Inventory, requirements, layout, design system and
  screen specifications are finished and signed off before a screen is
  implemented. The design is written without depending on the technology;
  the component catalogue is validated in a gallery on the real stack (GUI
  2.2) before screens are specified against it.
- **Mature technology agents work in reliably.** Any language is acceptable.
  The front end is TypeScript + React with shadcn/ui on Radix and Tailwind
  as the provisional component system. The **shell is a measured choice at
  GUI 2.3**: the same front end is built into Electron and into Tauri 2 and
  both are measured on the three required platforms; Electron is the default
  expectation, because the GUI links no Rust and one Chromium on every
  platform removes the WebKitGTK risk on Linux, and Tauri is chosen only if
  it meets the budgets everywhere and its smaller bundle is judged worth it.
  Qt/QML and Flutter are considered only if the web front end itself fails
  the budgets.
- **Responsiveness and performance come first.** No freeze and no long
  loading, including for large tournaments: loading them, creating them and
  starting them. The numbers are set at GUI 1.2 and are exit criteria at GUI
  2.3 and 4.8, not aspirations. The rules that make them hold: the UI thread
  only renders; anything that can take longer than a frame runs elsewhere
  (in the CLI process or off the UI thread); every long list is virtualised;
  live updates are coalesced to at most one render per frame; the window is
  usable before data has arrived and fills in as it does.
- **One set of standard components.** Every screen is composed from the
  design system's component catalogue (GUI 1.4). A screen that needs
  something new extends the catalogue first, where it is reviewed in both
  themes, and only then uses it. 1.x has no such catalogue and it shows.
- **Dependencies where they pay.** A dependency is accepted when it is
  mature, widely used, maintained, GPL-3.0-compatible and saves real work —
  the industry-standard choice, not a reinvented wheel — and is rejected when
  it overlaps one already taken or adds weight for little. The GUI repository
  keeps a dependency list with one line of reason per entry.
- **Platforms.** Required: macOS arm64, Linux x64, Windows x64. Wanted where
  cheap: macOS x64, Linux arm64, Windows arm64.
- **Theme.** Follows the device by default; light and dark can be chosen
  explicitly; both are designed and tested equally.
- **Scope.** Tournaments (round-robin, gauntlet), the engine library, live
  viewing, results, history and export; several tournaments may run at once,
  as in 1.x. CLI runs (match, SPRT, SPSA, tunes, suites) are not shown in the
  desktop application. The 1.x tournaments are not migrated; the 1.x engine
  library is imported, because it is the user's own data and cheap to keep.
  No served dashboard now; playing on one machine and watching or
  controlling from another is a later feature the protocol keeps possible.
- **Placement.** CPU placement is a CLI feature for gates and tunes, where
  it matters. The desktop application runs tournaments without it and offers
  no placement control, so several tournaments can run at once without
  coordination; placement must always be possible to turn off.
- **Scale.** Tournaments of **64 participants** are the design point, not
  the ceiling: creation, start, standings and crosstable are designed and
  measured at that size. The other targets are proposed in the GUI plan and
  confirmed at GUI 1.2.
- **Lifecycle and stopping.** Two user actions — *finish the games in
  progress and stop*, and *stop now* — both leaving a tournament that
  resumes. What closing the application does, whether a tournament can keep
  running without it, and whether it can be re-attached are analysed and
  chosen at implementation (G here, GUI 4.2), not now. The maintainer's
  starting preference: closing the application stops now; keeping a
  tournament running without it is worth evaluating; remote play and watch
  is the later feature above.
- **Changed engines and amendments.** Decided at B, specified at C,
  implemented in H and I: what happens when an engine executable
  changed since a tournament started; removing a participant from an active
  tournament; adding a participant to a running one; changing an active
  tournament's length. All are CLI mechanisms the desktop application only
  presents.
- **Configuration and data.** The desktop application writes a run file per
  tournament and starts the CLI on it; its tournament list is a rebuildable
  index over run directories; it holds no chess logic, because positions
  arrive as FEN; it never re-implements a rating or statistic.
- **Sign-offs.** Each design step ends in a maintainer sign-off; consecutive
  document steps are reviewed in pairs (inventory with requirements,
  concepts with design system) so the design converges in a few sessions.

**Where the work happens.** Steps A–L are in this repository. GUI Phases
1–5 are in the GUI repository from step 11.2 on, tracked there. The egui
application is maintained in the desktop repository's `legacy/` until 2.0.0;
nothing of it is fixed here. The requests plan's steps are taken beside all
of it as requests arrive.

**Programme exit:** one game-playing implementation, in the CLI; the desktop
application released from its own repository over the protocol; this
repository building and releasing the CLI alone.

---

## G2. The front-end protocol

**Status:** principles binding if the programme continues (step A); the
message catalogue, the JSON Schema and the conformance fixtures are written
at step C from the gap analysis (B) and the GUI requirements, and
implemented in steps D–K.

A front end — the desktop application first, any other client later — drives
the CLI as a child process, as a chess GUI drives a UCI engine. The CLI owns
everything that plays or scores; the front end presents.

- **Framing.** One JSON object per line on standard output for events and
  responses; one JSON object per line on standard input for commands. Human
  output stays on standard error. It is an explicit mode selected by a flag
  (named at C), never inferred.
- **Handshake and versioning.** The first message of a session is the CLI's
  `hello`: the protocol version as major and minor, the CLI version, the
  schema identifier and the capabilities the session offers. Versions travel
  in the handshake, not in every message. Added fields and message types are
  a minor change a client must ignore when unknown; anything else is a major
  change. The JSON Schema is generated from the Rust types, versioned with
  the CLI and published together with the conformance fixtures as one
  release asset; the fixtures live in `tests/fixtures/`.
- **Configuration by run file.** A front end writes the tournament's run
  file into the run directory and starts the CLI on `--run-file` and
  `--dir`, so every tournament a front end plays is reproducible from a
  shell with the same file. Argument vectors are for people.
- **Two classes of event.** *Facts* — run state, game start, move, game end,
  amendment, and the standings and rating snapshot after every scored game —
  carry a sequence number and are never silently dropped. *State* — the
  latest search line, clocks between moves — is coalescable, latest wins,
  rate-limited.
- **Emission never touches the game path.** Events leave through a bounded
  queue on its own task. A slow or stalled reader costs the front end
  fidelity, never an engine its clock: state is dropped first; when a
  bounded fact queue would overflow, the CLI emits one `gap` fact naming the
  missing sequence range and continues. The journal remains the record of
  every game.
- **Snapshot and resync.** A client may request a full snapshot of the run
  at any time and then apply the facts above the snapshot's sequence number.
  This is how a client recovers from a gap, and it is the primitive a later
  re-attach or remote transport needs.
- **Positions travel as FEN.** Every move fact and every game a query returns
  carries the position after the move as FEN beside the move in UCI and SAN
  notation, so a front end draws boards without implementing chess.
- **Control.** *Stop* finishes the games in progress, starts no new one and
  ends the run cancelled and resumable, with no grace timeout; *stop now*
  abandons the games in progress, which replay on resume. Both arrive as
  protocol commands and behave identically on every platform, including
  Windows, where a console interrupt cannot be sent to one child process
  (S5.11). What end of input means — stop, stop now, or keep playing
  detached — and whether a run can be re-attached are analysed and decided
  at step G; every option must leave a resumable run, never an orphan.
- **Amendments.** A tournament in progress can be amended: a participant
  removed, a participant added, the length changed. The journal stays
  append-only and every played game stays in it; game identity is stable
  across amendments; each amendment is a fact in the stream and an entry in
  the run record and the configuration origins; resume, `status` and `stats`
  understand an amended run; ratings are recomputed jointly over every
  scored game, as always. The exact semantics — what a removed participant's
  played games count for, how a schedule grows — are decided at B and
  specified at C.
- **Changed engines.** A resume whose engine executables differ from the
  stored ones is never silent. Whether it is refused (S5.11 today), accepted
  on an explicit request that records the change as an anomaly, or handled
  by the front end as an amendment is decided at B and specified at C.
- **Queries are short invocations.** Read-only questions over a run
  directory — a summary, a page of games with positions, standings and
  crosstable — are separate `--json` invocations in the pattern of `status`,
  not session commands. They answer within the front end's budgets without
  parsing every game: the journal already records each game's byte range in
  `games.pgn`. They never mutate a run.
- **Synthetic streams.** The CLI can emit a schema-valid session without
  engines: a synthetic tournament at a chosen scale and update rate, and a
  replay of a finished run directory's journal as a stream. Front-end
  performance proofs and end-to-end tests run against these, so both
  repositories test the same messages.
- **Placement is not the protocol's concern.** CPU placement stays a CLI
  feature for gates and tunes; the desktop application starts tournaments
  with placement off, several at a time if the user wishes, and nothing in
  the protocol assumes pinned cores.
- **Transport independence.** Messages are defined apart from stdio, so the
  same catalogue can later travel over a local socket (re-attaching to a
  running tournament) or a WebSocket (playing on one machine, watching or
  controlling from another). Neither transport is scheduled; neither is
  closed off.
- **The protocol observes and requests; it never changes the result.** A
  run driven through the protocol writes the same run directory, PGN,
  statistics and run record as the same run without it, apart from timing.
  This is demonstrated with the deterministic stub engines of the clean-stop
  suite, because real engines never reproduce a run.

---

## G3. Steps

Each step carries a capability class from the requests plan's table
(`PLAN.md` §S8), with the same meaning and the same model mapping
(`GUIDE.md`).

### Order of work

The open steps in the order they are taken. `GUIDE_GUI.md`'s board reads
this table, and its **Now** is the first row; a finished step leaves the
table in the commit that finishes it.

| Step | Waits for |
|---|---|
| A | the maintainer's revision of the programme |
| B | A, and GUI 1.2 signed off |

### History — CLOSED

Naming (ADR-0012), the GUI repository's seed, the egui application retired
from this repository (2026-09-29), and the swap and merge done at the split
(2026-11-14), as the steps of the old Phases 11 and 14 they were.

- **11.1 Naming — `R2`, DONE.** The names of both products, their executables
  and the two repositories, after a dated collision screen as in Phase 0.6.
  The recommendation going in: the CLI keeps `colosseum-cli`, because its
  value to scripted users is stability and a rename would land on exactly
  the projects that adopted it; the desktop application takes the new name,
  or keeps the Colosseum brand its installers own; this repository keeps its
  history and becomes the CLI repository. The maintainer decides.
  **Exit:** an ADR recording both product names, both executables and both
  repository names (superseding ADR-0009 if anything changes).
  **Done 2026-09-28:** [ADR-0012](docs/architecture/adr/0012-names-and-repositories-after-the-split.md).
  The desktop application stays **Colosseum** and the CLI stays
  `colosseum-cli`. The maintainer lifted the adoption premise, since Rarog is
  the only adopter, but the bundled CLI, the brand and the `cutechess-cli`
  pattern decide for the existing name. The desktop repository starts as
  `maelic13/colosseum-gui`. At 14.1 the maintainer renames this repository
  to `maelic13/colosseum-cli` and the desktop repository to
  `maelic13/colosseum`, before both `dev` branches are merged and before
  both releases (revised 2026-09-29). Both use plain `v` tags; the desktop
  application's first release also carries one `gui-v2.0.0` bridge release
  for the 1.1.0 updater. The 1.x releases and their tags are deleted from
  this repository at L. The new application must not share 1.x's data
  directories.
- **11.2 The GUI repository seed — `M`, DONE.** `docs/gui-v2/`
  finalised under the 11.1 names — `AGENTS.md`, `PLAN.md`, `GUIDE.md` — and
  handed to the maintainer, who creates the remote repository
  (`maelic13/colosseum-gui`) and commits the seed as its first commit. The
  seed carries ADR-0012's obligations for that repository: plain `v` tags,
  first release 2.0.0 with its `gui-v2.0.0` bridge release, the CLI fetched
  from `maelic13/colosseum-cli` after the swap, and data directories apart
  from 1.x's with the installer relationship to 1.x decided at GUI 4.7.
  `docs/gui-v2/` is then reduced here to a pointer, and `README.md` names
  the new application. The inventory of 1.x (GUI 1.1) reads this
  repository's code at the `gui-v1.1.0` tag; the seed names its commit
  (`ba30829`) as well, because L deletes the tag.
  **Exit:** the GUI repository exists with the seed as its root; this
  repository points to it. (M1)
  **Done 2026-09-29:** [`maelic13/colosseum-gui`](https://github.com/maelic13/colosseum-gui) published
  by the maintainer; its root commit `22d0be1` holds `AGENTS.md`,
  `PLAN.md` and `GUIDE.md` byte-identical to the seed at `54b9c47`, with
  the licence and a `.gitattributes`.
- **11.2.1 Retire the egui application from `dev` — `I1`, DONE
  2026-09-29.** After GUI 1.1
  has read the 1.x code, and on `dev` only: `main` keeps 1.x and CLI 0.2.0
  until 14.1 (ADR-0012, *Branches and order*). Remove `colosseum-gui`,
  `engine::scheduler`, the SQLite store, the runtime adapter, and the `gui-v`
  release lane and its packaging. Archive `docs/design/`. S4, `CLAUDE.md`,
  `AGENTS.md`, `README.md` and `docs/DEVELOPMENT.md` describe a CLI-only
  repository that points to the new application. ADR-0006 is superseded. The
  1.x source stays in history at `ba30829` and on `main`.
  **Exit:** the workspace builds and tests without the GUI; the required CLI
  suite and the parity checks are unchanged; nothing on `dev` names a GUI
  1.x module.
  **Done 2026-09-29:** removed `crates/colosseum-gui`; from
  `colosseum-engine` the `tournament` feature (`scheduler`, `store`,
  `detect`, `error`) and its `scheduler_smoke` target, `rusqlite`,
  `crossbeam-channel`, `anyhow`, `uuid` and `serde_json`; from
  `colosseum-core` the types only they used (`TournamentConfig`,
  `CommonEngineOptions`, `RatingWriteback`, `StartPosition`,
  `TournamentEvent`, `EngineConfig`, `EngineMeta`, `TimeUnit`,
  `TournamentId`, `generate_schedule`); the `gui` product from `xtask` and
  the release tool, `release-gui.yml`, the GUI smoke script, the Linux GTK
  steps in CI and the desktop packaging files. `docs/design/`, the GUI
  changelog, screenshot and macOS signing notes moved to
  `docs/archive/gui-1.x/`. The CLI's code, its tests and the Phase 7 GUI
  parity fixture are untouched; the architecture tests keep their
  forbidden-dependency lists as guards.
The egui application left `dev` at 11.2.1. The swap and the merge were
moved to the split by the desktop repository's ADR-0001 and done there
(ADR-0012, *Revision 2026-11-14*); what is left is the release pair.

- **14.1 Swap and merge — `I1`, DONE 2026-11-14.** Done at the split, not
  after GUI 4.8. The maintainer swapped the repositories. On `dev`, before
  its merge into `master`: `master` merged in (CLI 0.3.0–0.5.0), the CLI
  release lane moved to plain `v<semver>` tags on `master` that claim
  Latest (`release.yml`, `cargo xtask release-check`, the architecture
  tests), CI gating pull requests into `master`, every link to this
  repository's old address moved to `maelic13/colosseum-cli`,
  `CHANGELOG-CLI.md` renamed `CHANGELOG.md`, the archived 1.x documents and
  the GUI seed pointer removed (the desktop repository holds them). The
  maintainer re-tags `cli-v0.1.0`–`cli-v0.5.0` as `v0.1.0`–`v0.5.0` and,
  after desktop 1.2.0 is out, deletes the 1.x releases and tags here. (M8)

### Decision

- **A Revise and decide the GUI programme — `R2`.** The maintainer is
  reconsidering the desktop application (2026-11-20). The programme was cut
  on 2026-09-27 for one client; this step decides whether it continues as
  specified, continues narrowed to what any client of the CLI could use — an
  event stream of game facts with a snapshot (F's facts), a stop command
  that behaves the same on every platform (G) and read-only queries (J) — or
  stops. If it stops: B–L are refused; the CLI's leftovers of the 1.x live
  view — the runner's live-game state (`engine::live` and the writes the
  runner makes to it on every search) and `performance_rating`, kept until
  now because F and J.1 would use them — are removed by a requests-plan
  step; the GUI risks leave this plan; and ADR-0011 is superseded by a new
  ADR. If it continues, the requirements in G1 are reviewed against the
  revision before B starts.
  **Exit:** the decision in an ADR; this plan re-cut to it.

### Separation

- **B The CLI against the requirements — `R2`.** After GUI 1.2 is
  signed off: a head-to-head evaluation of what `colosseum-cli` does today
  against every *keep* and *change* item of the inventory and every
  requirement, the later ones included (detach, remote watch and control).
  Each item is classified *supported as-is* (naming the command and output),
  *CLI extension needed* (small or large, with a sketch), *owned by the GUI*
  (per ADR-0011), or *not feasible as specified* (reason and alternatives).
  Known candidates: the live event stream; the control channel; amendments;
  the changed-engine policy; paged queries with positions; the synthetic
  streams; anything the engine library needs beyond `engine inspect --json`
  and `engine check --json`; and the CLI column of the GUI repository's `docs/design/inventory.md` §8, drafted
  2026-09-29 — tournament statistics (J.1), tournament options
  (J.2), SAN and opening names in the live events (F). The two
  decisions the maintainer asked for are taken here with him: the
  changed-engine behaviour and the amendment semantics. The inventory's
  other CLI questions are B.1–B.5. Two are settled: the time
  tolerance past the clock is already `--margin-ms`, any value, so the
  application chooses its default (GUI 1.1.2); ratings follow the
  2026-09-29 requirement above.
  **Exit:** `docs/architecture/gui-gap-analysis.md`; every requirement
  classified; *not feasible* items resolved with the maintainer; H–J
  confirmed or re-cut from it. (M2)
- **B.1 Decide engine option resolution — `R2`.** Inventory PLY-11,
  PLY-12, PLY-13: mapping the requested threads and hash to what each
  engine calls them, clamped to the declared range (the exact-name
  allowlist, never substrings); the compatibility notes before a start;
  the global tablebase settings and the per-tournament switch that
  withholds every tablebase option. Either the CLI resolves them against
  the engines' declared options, or the application composes exact
  per-engine options into the run file from its library.
  **Exit:** the choice in the gap analysis, with the implementing step
  named (J.2 or a GUI step).
- **B.2 Decide ponder — `R2`.** Inventory PLY-14: whether a tournament
  with ponder off sends `Ponder=false` to every engine, as 1.x did, and
  whether ponder is allowed without a clock control or without separate
  cores for each engine, which the CLI refuses today.
  **Exit:** the choice in the gap analysis; a CLI change, if any, named.
- **B.3 Decide adjudication — `R2`.** Inventory PLY-15, PLY-35: the
  defaults the application offers (1.x: draw from ply 40, 8 moves, 8 cp;
  resign 4 moves, 800 cp; max moves 300 — against the CLI's 40 moves,
  8 moves, 10 cp; 3 moves, 600 cp); whether one-sided resign is offered;
  and two shared runner behaviours that change results — book and
  scoreless plies counting in the adjudication windows, and the start
  position not counting toward a threefold repetition.
  **Exit:** the choice in the gap analysis; a result-changing CLI fix is
  a minor-release step of its own, never a patch.
- **B.4 Decide faults in tournaments — `R2`.** Inventory PLY-17,
  PLY-18, PLY-35: a game whose engine cannot start — the CLI ends the run
  as an infrastructure error; 1.x skipped the game and replayed it at the
  next resume; the fault allowance that marks a run invalid; and whether a
  hang in a fixed-nodes or fixed-depth search is a time loss or a crash.
  **Exit:** the choice in the gap analysis; a CLI change, if any, named.
- **B.5 Decide scheduling — `R2`.** Inventory PLY-26 and DEF-14:
  changing the number of parallel games while a tournament runs (1.x yes,
  CLI fixed per run); and colours across cycles when a pair plays one game
  per cycle — the shared pairing gives the same engine White every cycle.
  A pairing change alters the schedule of a resumable run, so the fix
  names its resume compatibility.
  **Exit:** the choice in the gap analysis; a CLI change, if any, named.
- **C Protocol specification — `R2`, EXIT.** G2 filled in: the
  message catalogue (handshake, facts, state, commands, responses, errors,
  gap and snapshot), the run-file hand-off, the amendment and changed-engine
  semantics from B, the versioning rules, the JSON Schema generated from
  Rust types (the types may land before their behaviour), the conformance
  fixture format, and the surface of the synthetic-stream commands. Written
  so GUI 3.1–3.3 can name the message each screen reads.
  **Exit:** `docs/cli/protocol.md` (user-facing: the contract) and the
  schema in the repository; the GUI plan references them; the G2
  principles unchanged, or amended by ADR. (M3)

### Protocol

- **D Session — `I2`.** The protocol mode: the flag, framing, the
  handshake, the event pipeline (a bounded queue on its own task, state
  coalescing, fact sequence numbers, `gap`), the snapshot request, error
  responses, and a test harness that validates every emitted message
  against the schema. Run state only; no game events yet.
  **Exit:** a protocol session over `tournament run` with stub engines emits
  a valid handshake, run state and terminal state; the clean-stop suite is
  unchanged.
- **E Synthetic streams — `I1`.** The two generators: a synthetic
  tournament at a chosen participant count, game count, concurrency and
  update rate, and a replay of a finished run directory at real or
  accelerated speed; both schema-valid, neither needs an engine.
  **Exit:** GUI 2.3 can run against them; their output recorded as
  conformance fixtures. (M4)
- **F Live events — `I2`.** `tournament run` publishes game start,
  every move with its SAN, clocks and FEN and the opening's ECO code and
  name while it is known (the 1.x `eco` module and its lichess CC0 data,
  restored from `ba30829`), the latest search information per side as
  state, game end with termination and sample class, and the standings and
  rating snapshot after every scored game — fanned out from the runner's
  live state, which exists (`engine::live`), through a richer tournament
  observer.
  **Exit:** a real two-engine tournament observed through the protocol; the
  durable artifacts identical to the same run without the protocol, shown
  with stub engines.
- **G Control and lifecycle — `R2`, then `I2`.** *Stop* and *stop now* as
  protocol commands, identical on every platform; resume reports what it
  resumed. The analysis the maintainer deferred is done here, together with
  GUI 4.2: what end of input means, whether a run can keep playing detached
  and be re-attached, what the desktop application does on close; the
  choice is recorded in G2 and an ADR.
  **Exit:** the clean-stop suite extended with both protocol commands and
  the chosen end-of-input behaviour for `tournament run`. (M5, with F)
- **H Amendments — `I2`.** Remove a participant, add a participant,
  change the length of a tournament in progress, per the B and C
  semantics: append-only journal, stable game identity, amendment facts and
  run-record entries, resume, `status` and `stats` on amended runs.
  **Exit:** kill, amend and resume tests reach the statistics of an
  uninterrupted amended run; `stats` replays an amended directory.
- **I Changed engines — `I1`.** The policy chosen at B for an
  engine executable that changed since the run started: never silent,
  always recorded.
  **Exit:** a test per branch of the policy; the refusal message names the
  alternative.
- **J Queries — `I1`.** The read-only invocations: run summary,
  paged games with FEN per ply, standings and crosstable, on a run directory
  of the GUI 1.2 largest size within the CLI-side budgets, using the
  journal's PGN byte ranges.
  **Exit:** budgets met on a synthetic directory at the largest target;
  nothing mutated. (M6, with H and I)
- **J.1 Tournament statistics — `I1`.** What the 1.x standings showed
  and the CLI does not compute (the GUI repository's `docs/design/inventory.md`, ARN-11, ARN-12, ARN-18, PLY-21,
  PLY-22, PLY-24): Sonneborn–Berger with shared places, the one-seed
  gauntlet's score-share ranking, the performance rating, each engine's
  average nps (derived from nodes and time when an engine reports 0),
  depth and time per move, forfeits split into time, crash and illegal
  move, the termination breakdown and the decisive/drawn split — in
  `result.json`, the standings query (J) and the snapshot fact (F).
  **Exit:** the figures equal 1.x's on a frozen fixture replayed through
  both, as Phase 7 matched the GUI's ratings.
- **J.2 Tournament options for the application — `I1`.** From the
  inventory (PLY-05, PLY-19, PLY-20) and whatever B.1–B.5 put in the
  CLI: an opening count cap, the tournament's name and date in the PGN
  `Event` and `Date`, and "name version" labels carried to every output.
  **Exit:** a test per option; the run-file schema and the command
  reference regenerated.
- **K Conformance and candidate — `V`, EXIT.** The conformance
  suite green; the schema and fixtures packaged as one release asset;
  `docs/cli/protocol.md` final; a CLI release candidate built from the
  protocol's pull request (or dispatched on its branch), which GUI 4.8
  accepts against (M7). The release itself is L.

### Release

- **L Releases — `V`, EXIT.** The CLI minor release that carries the
  protocol, from `maelic13/colosseum-cli`; then the desktop 2.0.0 release,
  pinned to it (GUI 5.1), with no bridge release (desktop ADR-0001). No
  executable is renamed (ADR-0012). (M9)

---

## G4. Risks

| Risk | Mitigation |
|---|---|
| CLI churn destabilises the released GUI | Phase 0 architecture/release design; independent releases; shared-layer regression suite |
| GUI v2 loses a 1.x capability users rely on | GUI 1.1 inventory with every item classified; GUI 4.8 exit checks every *keep* item |
| The CLI cannot serve a GUI requirement, discovered late | B evaluates the CLI against every requirement before screens are specified; *not feasible* items resolved with the maintainer first |
| The web front end is not fast enough, notably WebKitGTK on Linux | Budgets set as numbers at GUI 1.2 and measured at GUI 2.3 on all three platforms against the CLI's synthetic streams before anything is built, in both Electron and Tauri with Electron the default expectation; the process boundary makes the whole front end replaceable |
| Protocol drift between two repositories | A versioned handshake, the generated JSON Schema and fixtures published with the CLI, the GUI repository running the pinned CLI against those fixtures in its CI, the desktop application bundling one CLI release |
| The protocol alters what a run measures | G2: emission never on the game path; protocol-driven and plain runs produce identical durable artifacts with stub engines; a F exit criterion |
| A stalled front end stalls an engine's clock | G2: bounded queue on its own task, state dropped first, `gap` facts instead of blocking |
| An amendment corrupts a tournament's evidence | G2: append-only journal, stable game identity, amendments recorded; H kill/amend/resume tests and `stats` replay |
| Design work never converges | Each design step ends in a maintainer sign-off, reviewed in pairs; GUI 1.3 limits the concepts to two or three; the gallery (GUI 2.2) tests the catalogue on the real stack before screens are specified |

## G5. Reference

| Path | What |
|---|---|
| `docs/architecture/gui-gap-analysis.md` | the CLI against the GUI requirements (B) |
| `docs/cli/protocol.md` | the front-end protocol contract (C, final at K) |
| `crates/colosseum-engine/src/live.rs` | the runner's live-game state, the protocol's source at F |
| `crates/colosseum-cli/src/tournament_driver.rs` | the tournament driver, which gains the G2 protocol in D–K |
