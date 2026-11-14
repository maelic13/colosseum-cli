# Colosseum CLI

Colosseum CLI is a command-line tool for chess-engine development. It plays
fixed matches, tells you with an SPRT when a change has made an engine
stronger or weaker, tunes engine parameters with SPSA, measures search speed,
and runs round robins and gauntlets. It works with any ordinary UCI engine:
give it an executable, and it needs no manifest, build integration or source
access.

Looking for a desktop application? [Colosseum](https://github.com/maelic13/colosseum)
is a separate program for running and watching engine tournaments; it is not
part of this archive.

## Start here

Check the executable and an engine:

```text
colosseum-cli --version
colosseum-cli self-test
colosseum-cli engine check ./my-engine
```

On Windows, run `colosseum-cli.exe`. On Linux and macOS, run
`./colosseum-cli` if the current directory is not on your `PATH`.

Play a first fixed match at 100 ms a move:

```text
colosseum-cli match ./candidate ./baseline --games 100 --a-movetime-ms 100 --b-movetime-ms 100
```

Every run writes a self-contained directory under `./colosseum-runs/`, or
where `--dir <path>` says. Stop a long run with one Ctrl+C and start the same
command again to resume it.

## Documentation

This archive holds the guide for this exact version, for offline use:

- [Quickstart](docs/cli/quickstart.md)
- [Complete guide](docs/cli/README.md), with a page per command
- [Command reference](docs/cli/command-reference.md)
- [How to trust and report a result](docs/cli/trust-results.md)
- [What the CLI needs from an engine](docs/cli/compatibility.md)
- [Changelog](CHANGELOG.md)

`colosseum-cli <command> --help` lists every option of a command.

Newer versions are on the
[release page](https://github.com/maelic13/colosseum-cli/releases). Report
problems through the [issue tracker](https://github.com/maelic13/colosseum-cli/issues).
Colosseum CLI is licensed under the [GNU GPL v3 or later](LICENSE).
