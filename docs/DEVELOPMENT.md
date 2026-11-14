# Development guide

Maintainer-facing notes: building from source, running the tests, and cutting
a release. [`README.md`](../README.md) introduces the CLI;
[`packaging/cli/README.md`](../packaging/cli/README.md) (the front page of
every CLI archive) and [`cli/`](cli/README.md) are the CLI landing page and
complete user guide. The desktop application is developed and released in
its own repository, [maelic13/colosseum](https://github.com/maelic13/colosseum),
as a client of this CLI.

## Prerequisites

| Requirement | Notes |
|---|---|
| **rustup** | installs the pinned toolchain, Rust 1.100.0, on the first build — see [The toolchain](#the-toolchain) |
| **C linker** | Windows: MSVC build tools; Linux: `gcc`; macOS: Xcode CLT |
| **clang** | Windows ARM64 only — see below |

### Windows ARM64: clang

On ARM64 Windows the `ring` crate compiles its assembly with `clang`; MSVC
alone fails with `ToolNotFound: failed to find tool "clang"`. Install
[LLVM](https://releases.llvm.org/) (`winget install LLVM.LLVM`) and make sure
it is on `PATH` — the installer does not add it by default — or point `CC` at
it for the build:

```powershell
$env:CC = "C:\Program Files\LLVM\bin\clang.exe"
cargo build --release
```

x86-64 Windows is unaffected, and GitHub's ARM runners ship LLVM, so CI needs
no extra step.

## The toolchain

`rust-toolchain.toml` pins one exact Rust release (`channel = "1.100.0"`),
with `rustfmt` and `clippy`. rustup reads it in this directory, so a local
build, CI and the release workflow all use that compiler; the workflows
install no toolchain of their own. The workspace `rust-version` names the
same release, and an architecture test keeps the two and the workflows in
step. `rustup update` elsewhere does not move it.

A toolchain update is its own pull request, with nothing else in it:

1. On a branch from `master`, set `channel` to the new release and
   `rust-version` in `Cargo.toml` to its major.minor.
2. Run the verification baseline (`AGENTS.md`) with the new compiler and fix
   what its new lints report; behaviour does not change in this pull
   request, so a test that needs changing is a finding to stop on.
3. Open the pull request titled "Rust X.Y.Z". CI and the release candidate
   prove it on every platform and archive; squash-merge once both are green.

## Build and run

```bash
git clone https://github.com/maelic13/colosseum-cli.git
cd colosseum-cli
cargo run -p colosseum-cli -- --help
```

## One build entry point

`cargo xtask` builds, packages and release-checks the CLI, and the release
workflow calls the same commands, so a local artifact and a published one come
from one recipe:

```bash
cargo xtask build   cli [--target <triple>] [--profile release|ci-release]
cargo xtask package cli [--target <triple>] [--format <list>] [--no-smoke]
cargo xtask release-check <vX.Y.Z>
```

The product is the positional `cli`. `--target` defaults to the host's triple
and is always passed to cargo, so every build lands under
`target/<triple>/<profile>/`; builds are `--locked`.

`build` compiles and prints the binary's path, and copies nothing. `package`
always uses the `release` profile, writes artifacts to `target/dist/` as
`<product>-<version>-<platform>-<arch>.<ext>`, prints each one's SHA-256, and
reads the portable archive back with its smoke script unless `--no-smoke`.
`--format` defaults to the platform's portable archive — `zip` on Windows,
`tar.gz` elsewhere — the only format the CLI ships.

```bash
cargo xtask package cli                       # this host's portable archive
cargo xtask release-check v0.6.0
```

## Tests

```bash
cargo check --workspace --tests
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo run -p colosseum-docs -- --check
```

Those commands are the required hermetic suite: they use only inputs owned by
the repository. They do not discover an installed engine, read an engine-path
environment variable, or establish a platform or release claim.

How the suite is laid out:

- Logic that needs no process — statistics, schedules, parsers, report and
  progress formatting, resume arithmetic — is unit-tested beside its code in
  `src/`. Where a rule depends on time or the host, the code takes the instant
  or the CPU topology as an input so the test can supply a synthetic one.
- Each crate's integration tests drive only what needs a real process. The
  CLI's are one binary, `crates/colosseum-cli/tests/cli/`, with a module per
  behaviour; they run the `colosseum-cli` binary against the repository's
  fixture engines (`colosseum-uci-fixture` and the hidden `__uci-stub` mode)
  for a handful of games, and stop runs with the hidden
  `--__stop-after-units` hook rather than a timed kill.
- No test asserts a wall-clock upper bound, a throughput or a memory figure,
  or depends on which CPUs the host has; measured time is checked only from
  below.

The phase acceptance manifests under `docs/fixtures/` record the evidence as
it stood when each phase closed. Test files and names they cite predate the
suite's reorganisation by behaviour and are historical.

Real-engine interoperability coverage is a separate, explicit local smoke
tier. It receives a local UCI executable through `COLOSSEUM_SMOKE_ENGINE`,
copies that executable to a temporary directory, and fails if the variable is
absent or invalid rather than passing by skip. `uci_smoke` expects `Threads`
and `Hash`; `runner_smoke` additionally exercises Stockfish-style strength
options. Run the targets appropriate to the selected
engine, for example:

```bash
COLOSSEUM_SMOKE_ENGINE=/path/to/engine \
  cargo test -p colosseum-uci --features real-engine-smoke --test uci_smoke -- --nocapture

COLOSSEUM_SMOKE_ENGINE=/path/to/engine \
  cargo test -p colosseum-engine --features real-engine-smoke --test runner_smoke -- --nocapture
```

On PowerShell, set `$env:COLOSSEUM_SMOKE_ENGINE` before running the same Cargo
commands. These opt-in checks are useful local interoperability evidence only;
they never count as required CI, supported-platform, or release evidence.

Hard CPU affinity has its own opt-in `platform-smoke` tier. `affinity_smoke`
pins two busy fixture processes to one logical CPU each and checks that they
ran only there; it needs a host with enforceable affinity (Windows or Linux)
and fails elsewhere rather than passing by skip:

```bash
cargo test -p colosseum-engine --features platform-smoke --test affinity_smoke
```

## Workspace layout

```
colosseum-cli/
├─ crates/
│  ├─ colosseum-core/     Pure domain rules, statistics and opaque identity values
│  ├─ colosseum-application/ Runtime-neutral use cases and ports
│  ├─ colosseum-uci/      UCI protocol & async engine process management (tokio)
│  ├─ colosseum-engine/   One-game runner, PGN, openings, incident forensics; OS topology and affinity adapters
│  └─ colosseum-cli/      the headless CLI composition root
├─ tools/release/         tag/version/changelog validation, archive staging and smoke
├─ tools/xtask/           the build/package/release-check entry point the workflow calls
├─ tools/docs/            parser-derived CLI command-reference generator
├─ packaging/cli/         README.md, the front page of every archive
├─ tests/fixtures/        vendored statistics fixtures and their generator
├─ docs/cli/              CLI user guide (shipped in every archive)
├─ docs/architecture/     architecture, ADRs, phase records and acceptance evidence
├─ docs/                  this guide and the logo
└─ .github/workflows/     pull-request CI and the release workflow
```

## Version and tags

The CLI owns its version in `crates/colosseum-cli/Cargo.toml` and its release
notes in [`CHANGELOG.md`](../CHANGELOG.md). Releases are plain `vX.Y.Z` tags
(ADR-0012); validate a prepared tag locally with:

```bash
cargo xtask release-check v0.6.0
```

`release-check` validates the tag's shape, that the manifest carries exactly
that version and that the changelog has a dated section for it
(`## [X.Y.Z] — YYYY-MM-DD`), then checks the generated command reference
against the parser and the working tree for whitespace damage.

The desktop application is released from its own repository,
[`maelic13/colosseum`](https://github.com/maelic13/colosseum), which bundles a
pinned CLI release; nothing here builds or releases it.

## Delivery

`master` is the trunk; nothing else lives long, and nothing is pushed to it
directly.

1. **One branch per coherent piece of work.** A feature, a fix, a release, or
   one tooling or documentation change gets its own short-lived branch from
   `master`. On it, commit in small verified steps; those commits are working
   records, and their subjects may name the `GUIDE.md` step.
2. **One squash commit per branch.** The branch lands on `master` through a
   pull request the maintainer squash-merges. No merge commits and no rebase
   merges; the repository allows only squash merges. GitHub keeps the
   branch's commits under the pull request (`refs/pull/<n>/head`) after the
   squash.
3. **The squash message is the pull request's title and description**, and
   it alone reaches `master`, so it alone has a fixed shape: a short
   imperative title that says what changed ("Report the time to an SPSA
   stop"), with no phase, step or plan references (a release is
   "Release X.Y.Z"); a few plain lines on what and why, wrapped at 72
   columns; then, after an empty line, the verification, one line each
   (`Tests: cargo test --workspace --all-targets, 1,204 passed`). No AI
   attribution anywhere; `.claude/settings.json` turns Claude Code's off.
4. **Before opening the pull request**, merge `master` into the branch if
   `master` moved, so it shows only the branch's own change.
5. **Two workflows run on every pull request into `master`**, and branch
   protection requires one check from each, which gathers the rest:
   - `CI complete` (`ci.yml`): formatting, the generated command reference,
     `cargo check`, clippy with `-D warnings`, the release build, and the
     hermetic workspace tests on Windows, Linux and macOS, debug and
     optimized;
   - `Verify the complete matrix` (`release.yml`): every release archive
     built, packaged and smoked, exactly as a tag would build it.

   Each always runs and fails unless every job it gathers succeeded: GitHub
   reports a job skipped after a failed dependency as passing, so a required
   check must never be skipped. The protection also requires the branch to
   be up to date with `master`, so the tested merge is the one that lands. A
   newer push to the pull request cancels the superseded runs. A pull
   request green in both is one a tag releases.
6. **After the merge**, the branch is deleted on `origin` (automatically) and
   locally.

```bash
git fetch origin
git switch -c <branch> origin/master
```

The optimized test leg uses `ci-release`, which inherits the shipped release
profile — same `opt-level`, assertions and overflow checks off — but drops the
distribution-only whole-program LTO and single codegen unit, and keeps symbols
so a CI failure still has a backtrace. Those settings change no behaviour the
suite can observe and cost minutes across the test binaries. Run it locally
the same way:

```bash
cargo test --workspace --all-targets --profile ci-release
```

The shipped binary is still built with the full `release` profile; only the
test legs use `ci-release`.

## Releases

A release is a `vX.Y.Z` annotated tag on `master`, cut only on maintainer
instruction, through `release.yml`; nothing else creates one. There are no
permanent release branches.

The workflow validates first, builds, packages and smokes every archive
read-only, verifies the exact archive list by name and count, and only then
one job with write permission publishes the GitHub Release, titled
"Colosseum CLI X.Y.Z" (the tag alone carries the `v`), with the notes taken
from the changelog section. Nothing exists on GitHub until every cell
has passed. Checksums are generated and re-checked between jobs but are not
published as an asset: the per-asset digests GitHub records are what to
compare a download against. It builds Windows x64/Arm64, Linux x64 and macOS
Arm64 archives, stages only the executable, license, README, changelog and
`docs/cli/`, then checks packaged documentation links and runs
version/help/self-test/deterministic JSON smoke against each unpacked archive.

1. **The release commit, by pull request.** On a branch from `master` (it may
   carry the changes being released), `crates/colosseum-cli/Cargo.toml` reads
   `X.Y.Z`, `Cargo.lock` agrees, and the changelog's `[Unreleased]` becomes
   `## [X.Y.Z] — YYYY-MM-DD`, with a fresh empty `[Unreleased]` above it. A
   release record goes under `docs/architecture/` when the release decided
   something. `cargo xtask release-check vX.Y.Z` passes. The pull request is
   titled "Release X.Y.Z".
2. **The rehearsal is that pull request.** Both required checks must be
   green. A candidate whose version is a plain, not yet tagged `X.Y.Z`
   proposes that release, so it also runs the release check before it builds.
   The maintainer squash-merges it; the squash has the pull request's exact
   tree, so its candidate run stands for it.
3. **The tag.** On the merged `master`, the maintainer tags and pushes. The
   tag run checks again, rebuilds every archive, proves the commit is on
   `master`, and only then publishes it as the latest release.
4. **Repair.** A failed tag run publishes nothing. Delete the tag locally and
   on `origin`, fix `master` through a pull request, and tag again.
5. **Reopen.** The next pull request into `master` that changes the product
   bumps its version to the next one with `-dev` (`0.7.0-dev`), so a build
   between releases never reports a released version. A `-dev` candidate
   proposes nothing and skips the release check, and no tag can publish one.
   The changelog's `[Unreleased]` grows as user-facing changes land, so step
   1 only dates it.

```bash
git fetch origin
git tag -a vX.Y.Z -m "Colosseum CLI X.Y.Z" origin/master
git push origin vX.Y.Z
```

**A patch while `master` holds unreleased work.** Land the fix on `master` by
its own pull request first. Then branch from the last release tag,
cherry-pick the fix with `-x`, make the release commit of step 1 for
`X.Y.Z+1`, tag it there, push only the tag, and delete the branch: the tag
keeps the commit. The tag run accepts a patch tag whose commit descends from
the `vX.Y.0` tag on `master`, and does not mark it the latest release.
Rehearse it first by a manual dispatch of the workflow on the branch.

```bash
git switch -c patch-X.Y.Z vX.Y.(Z-1)
git cherry-pick -x <fix-on-master>
git tag -a vX.Y.Z -m "Colosseum CLI X.Y.Z"
git push origin vX.Y.Z
git switch master
git branch -D patch-X.Y.Z
```

A rehearsal without a pull request is a manual dispatch of the workflow on any
ref (**Actions → Release → Run workflow**); it retains the bundle as
`colosseum-cli-candidate-<full-commit-sha>` with its checksums and a candidate
identity file, and creates no tag or release:

```bash
gh workflow run release.yml --ref <branch>
```

## CLI documentation

Canonical CLI user documentation lives under `docs/cli/`. The complete command
reference is generated from the same Clap model as the shipped executable:

```bash
cargo run -p colosseum-docs
cargo run -p colosseum-docs -- --check
```

The first command updates `docs/cli/command-reference.md`; the second is the
read-only CI/release drift gate. Edit parser help rather than the generated file.
GitHub renders the documents from each release tag, and CLI archives contain
the same guide for offline use. The archive's concise `README.md` directs users
to `docs/cli/README.md`; release notes link to the tagged online copy.
