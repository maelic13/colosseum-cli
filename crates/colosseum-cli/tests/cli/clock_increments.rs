//! Each side's increment reaches the engines from that side's own control.
//!
//! The fixture engine logs every command it receives, so the test reads from
//! the engines' side what every `go` told them: with unequal clocks, the
//! mover is told its opponent's real increment, not its own twice.

use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_colosseum-uci-fixture"))
}

/// The side to move in a logged `position` command.
fn side_to_move(position: &str) -> char {
    let rest = position
        .strip_prefix("position ")
        .expect("a position command");
    let (start, moves) = match rest.split_once(" moves ") {
        Some((start, moves)) => (start, moves.split_whitespace().count()),
        None => (rest, 0),
    };
    let first = match start.strip_prefix("fen ") {
        Some(fen) => fen.split_whitespace().nth(1).expect("a FEN side field"),
        None => "w",
    };
    let white_first = first == "w";
    if white_first == (moves % 2 == 0) {
        'w'
    } else {
        'b'
    }
}

/// The value after `name` in a `go` command.
fn field(go: &str, name: &str) -> u64 {
    let mut words = go.split_whitespace();
    words.find(|word| *word == name);
    words
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("`{go}` has no {name}"))
}

/// Every `go` the engine was sent, with the colour it was moving for.
fn searches(log: &Path) -> Vec<(char, String)> {
    let mut position = None;
    let mut searches = Vec::new();
    for line in std::fs::read_to_string(log).unwrap().lines() {
        let (_, command) = line.split_once(' ').unwrap();
        if command.starts_with("position ") {
            position = Some(side_to_move(command));
        } else if command.starts_with("go ") {
            searches.push((position.expect("a position before go"), command.to_owned()));
        }
    }
    searches
}

#[test]
fn unequal_clocks_tell_each_engine_both_sides_own_increments() {
    let root = tempfile::tempdir().unwrap();
    let (a_log, b_log) = (root.path().join("a.log"), root.path().join("b.log"));
    let output = Command::new(env!("CARGO_BIN_EXE_colosseum-cli"))
        .args(["match", "--games", "2"])
        .arg(fixture())
        .arg(fixture())
        .args([
            "--a-engine-arg=--legal-sequence",
            "--b-engine-arg=--legal-sequence",
            "--a-base-ms",
            "10000",
            "--a-increment-ms",
            "100",
            "--b-base-ms",
            "11270",
            "--b-increment-ms",
            "113",
            "--max-moves",
            "4",
            "--seed",
            "7",
            "--concurrency",
            "1",
            "--placement",
            "off",
            "--json",
        ])
        .arg(format!("--a-engine-arg=--log-commands={}", a_log.display()))
        .arg(format!("--b-engine-arg=--log-commands={}", b_log.display()))
        .arg("--dir")
        .arg(root.path().join("run"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["report"]["games_completed"], 2, "{report}");

    // Engine A's control is 10000+100 and B's 11270+113, whichever colour
    // each plays: the increments follow the engines, and so do the colours.
    for (log, own, other) in [(&a_log, 100, 113), (&b_log, 113, 100)] {
        let searches = searches(log);
        let mut colours = std::collections::BTreeSet::new();
        for (colour, go) in &searches {
            colours.insert(*colour);
            let (winc, binc) = (field(go, "winc"), field(go, "binc"));
            let expected = if *colour == 'w' {
                (own, other)
            } else {
                (other, own)
            };
            assert_eq!((winc, binc), expected, "{colour} to move: `{go}`");
        }
        assert_eq!(colours.len(), 2, "both colours played: {searches:?}");
    }
}
