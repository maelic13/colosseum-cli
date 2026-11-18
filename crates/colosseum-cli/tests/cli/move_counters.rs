//! An EPD opening's move counters reach the engines.
//!
//! Two integers after the four EPD fields are the halfmove clock and the
//! fullmove number; otherwise the `hmvc` and `fmvn` opcodes; otherwise
//! `0 1`. The fixture engine logs every command it receives, so the test
//! reads from the engines' side which start position each game sent, and the
//! resolved configuration says where each opening's counters came from.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_colosseum-uci-fixture"))
}

/// After 1.d4 d5 2.c4 e6 3.Nc3 Nf6, with its counters as a FEN's last fields.
const FIELDS: &str = "rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 2 4";
/// After 1.e4 e5 2.Nf3 Nc6, with its counters as opcodes, and a quoted
/// operand whose `;` and `hmvc` are not operations.
const OPCODES: &str = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - \
                       id \"two knights; hmvc 9\"; hmvc 2; fmvn 3;";
/// After 1.e4, with an opcode but no counters.
const DEFAULT: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - bm e5;";

/// Four legal plies from each opening, which both engines play from their
/// script; `--max-moves 2` then ends the game.
const LINES: [(&str, &str); 3] = [
    (FIELDS, "c1g5 f8e7 e2e3 e8g8"),
    (OPCODES, "f1b5 a7a6 b5a4 g8f6"),
    (DEFAULT, "e7e5 g1f3 b8c6 f1c4"),
];

/// Every start position the engine was sent.
fn start_positions(log: &Path) -> BTreeSet<String> {
    std::fs::read_to_string(log)
        .unwrap()
        .lines()
        .filter_map(|line| line.split_once(" position fen "))
        .map(|(_, rest)| {
            rest.split_once(" moves ")
                .map_or(rest, |(fen, _)| fen)
                .to_owned()
        })
        .collect()
}

/// A slice of a book is a shard of it: its positions keep their counters,
/// written as a FEN's last two fields.
#[test]
fn a_book_slice_keeps_its_move_counters() {
    let root = tempfile::tempdir().unwrap();
    let book = root.path().join("book.epd");
    std::fs::write(&book, format!("{FIELDS}\n{OPCODES}\n{DEFAULT}\n")).unwrap();
    let shard = root.path().join("shard.epd");
    let sliced = Command::new(env!("CARGO_BIN_EXE_colosseum-cli"))
        .args(["book", "slice"])
        .arg(&book)
        .arg(&shard)
        .args(["--count", "3"])
        .output()
        .unwrap();
    assert!(
        sliced.status.success(),
        "{}",
        String::from_utf8_lossy(&sliced.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&shard).unwrap(),
        format!(
            "{FIELDS}\n\
             r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3\n\
             rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1\n"
        )
    );
}

#[test]
fn an_epd_opening_sends_its_own_move_counters() {
    let root = tempfile::tempdir().unwrap();
    let book = root.path().join("book.epd");
    std::fs::write(&book, format!("{FIELDS}\n{OPCODES}\n{DEFAULT}\n")).unwrap();
    let script = root.path().join("script.txt");
    let lines = LINES
        .map(|(epd, moves)| {
            let start = epd.split_whitespace().take(4).collect::<Vec<_>>().join(" ");
            format!("{start} | {moves}\n")
        })
        .concat();
    std::fs::write(&script, lines).unwrap();
    let (a_log, b_log) = (root.path().join("a.log"), root.path().join("b.log"));
    let run = root.path().join("run");
    let output = Command::new(env!("CARGO_BIN_EXE_colosseum-cli"))
        .args(["match", "--games", "6"])
        .arg(fixture())
        .arg(fixture())
        .arg(format!("--a-engine-arg=--script={}", script.display()))
        .arg(format!("--b-engine-arg=--script={}", script.display()))
        .args([
            "--max-moves",
            "2",
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
        .arg("--book")
        .arg(&book)
        .arg("--dir")
        .arg(&run)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["report"]["games_completed"], 6, "{report}");

    // Each opening reaches both engines with its own counters.
    let expected = BTreeSet::from([
        FIELDS.to_owned(),
        "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3".to_owned(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1".to_owned(),
    ]);
    assert_eq!(start_positions(&a_log), expected);
    assert_eq!(start_positions(&b_log), expected);

    // The resolved configuration says where the counters came from.
    let resolved: Value =
        serde_json::from_slice(&std::fs::read(run.join("resolved-config.json")).unwrap()).unwrap();
    assert_eq!(
        resolved["openings"]["move_counters"],
        serde_json::json!({ "fields": 1, "opcodes": 1, "default": 1 }),
        "{}",
        resolved["openings"]
    );
}
