<p align="center">
  <img src="docs/logo.svg" width="88" alt="Colosseum logo">
</p>

<h1 align="center">Colosseum CLI</h1>

<p align="center">
  Test UCI chess engines from the command line, with results you can trust.
</p>

> **Windows · Linux · macOS · GPL-3.0-or-later**

Colosseum CLI is a command-line tool for chess-engine development. It answers
*is this build stronger than that one, and can I believe the answer?* It plays
fixed matches, tells you with an SPRT when a change has made an engine
stronger or weaker, tunes engine parameters with SPSA, measures search speed,
and runs round robins and gauntlets. It works with any ordinary
[UCI](https://www.shredderchess.com/chess-info/features/uci-universal-chess-interface.html)
engine: give it an executable, and it needs no manifest, build integration or
source access.

Looking for a desktop application? [Colosseum](https://github.com/maelic13/colosseum)
is a separate program for running and watching engine tournaments: it shows
the games live, keeps a library of your engines and rates them.

## Install

Download the archive for your machine from the
[latest release](https://github.com/maelic13/colosseum-cli/releases/latest)
and extract it anywhere.

| Platform | File |
|---|---|
| Windows, Intel or AMD | `colosseum-cli-…-windows-x64.zip` |
| Windows, Arm (Snapdragon) | `colosseum-cli-…-windows-arm64.zip` |
| Linux, Intel or AMD | `colosseum-cli-…-linux-x64.tar.gz` |
| macOS, Apple Silicon | `colosseum-cli-…-macos-arm64.tar.gz` |

Then check it works:

```text
colosseum-cli --version
colosseum-cli self-test
```

On Linux and macOS, run `./colosseum-cli` unless the directory is on your
`PATH`. The archive also holds the licence and the complete guide for that
exact version, so it works offline.

To check a download arrived intact, compare its SHA-256 with the digest GitHub
shows beside that asset on the release page:

```text
Get-FileHash colosseum-cli-0.5.0-windows-x64.zip
sha256sum colosseum-cli-0.5.0-linux-x64.tar.gz
shasum -a 256 colosseum-cli-0.5.0-macos-arm64.tar.gz
```

The three lines are Windows, Linux and macOS respectively. If you pin a version
in automation, pin that digest with it.

## First experiments

Check two engines, then play a fixed match between them at 100 ms a move:

```text
colosseum-cli engine check ./candidate
colosseum-cli engine check ./baseline
colosseum-cli match ./candidate ./baseline --games 100 --a-movetime-ms 100 --b-movetime-ms 100
```

Gate a change with a sequential test that stops as soon as the evidence is in:

```text
colosseum-cli sprt ./candidate ./baseline --model normalized --elo0 0 --elo1 5 --alpha 0.05 --beta 0.05 --max-pairs 5000 --book ./openings.epd --dir ./runs/gate-42
```

Every opening is played twice, once with each colour. A run refuses to start
if the book is too small for it, rather than quietly replaying openings — size
the book to the run, or pass `--book-wrap` to reuse openings on purpose.

Tune UCI options with SPSA from a small TOML file that lists the parameters:

```text
colosseum-cli spsa ./engine --tune ./tune.toml --r-end 0.002 --iterations 500 --book ./openings.epd --dir ./runs/tune-1
```

Stop a long run with one Ctrl+C and start the same command again to resume
it.

## What it can do

| Command | What it is for |
|---|---|
| `match` | A fixed number of games between two engines |
| `sprt` | Decide whether a change gained or lost strength, stopping as soon as the evidence is in |
| `spsa` | Tune numeric UCI options, with history, status and seeding from an earlier tune |
| `tournament`, `gauntlet` | Round robins and gauntlets with joint ratings and error bars |
| `calibrate` | Check that an engine against itself scores even under your conditions |
| `nps` | Measure search speed at a fixed node count, and how it scales with threads |
| `suite` | Fixed-work searches over an EPD or FEN position set |
| `book` | Inspect, verify, hash or slice an opening book |
| `stats` | Recompute results from a run's games, or plan how many games a test needs |
| `engine` | Inspect an engine's UCI options or check it behaves |
| `status` | Read the state of any run, running or finished, without touching it |
| `capabilities`, `self-test` | What this machine supports, and whether this executable works |

`colosseum-cli <command> --help` lists every option.

## Results you can trust

Every run writes a self-contained directory, under `./colosseum-runs/` unless
`--dir` names one. It records the resolved settings, the engine executables'
hashes, one master seed, every game, checkpoints and the final statistics, so
a result can be checked, replayed or resumed later. Engine crashes, time
losses and illegal moves are counted and shown, never folded silently into a
score. Engines can be pinned to CPU cores so parallel games do not disturb
each other.

The settings every experiment in a project shares — the book, the
concurrency, the seed — belong in a run file rather than in each command line,
so every result was measured the same way:

```toml
# colosseum.toml, beside your books/ directory
[options]
concurrency = 4
placement = "auto"
book = "books/openings.epd"
book-order = "random"
seed = 42
```

```text
colosseum-cli --run-file ./colosseum.toml match ./candidate ./baseline --games 100
```

A path inside a run file is relative to the file itself, so the file travels
with the project. Anything on the command line replaces what the file says.

## Learn more

- [Quickstart](docs/cli/quickstart.md)
- [Complete guide](docs/cli/README.md), with a page per command
- [Command reference](docs/cli/command-reference.md)
- [How to trust and report a result](docs/cli/trust-results.md)
- [What the CLI needs from an engine](docs/cli/compatibility.md)
- [Run files](docs/cli/run-files.md)

## What changed

See the [changelog](CHANGELOG.md).

## Building from source

Clone the repository and run it with Cargo. [rustup](https://rustup.rs/)
installs the exact Rust version the project pins on the first build:

```text
git clone https://github.com/maelic13/colosseum-cli.git
cd colosseum-cli
cargo run -p colosseum-cli -- --help
```

`cargo xtask` builds what a release ships:

```text
cargo xtask build   cli          # the executable, path printed
cargo xtask package cli          # the downloadable archive, into target/dist/
```

Windows on Arm also needs `clang`. The
[development guide](docs/DEVELOPMENT.md) covers that, the tests and the
release process.

## Help

Something broken or missing? [Open an issue](https://github.com/maelic13/colosseum-cli/issues).

## License

[GPL-3.0-or-later](LICENSE). Copyright © 2026 Miloslav Macůrek.
