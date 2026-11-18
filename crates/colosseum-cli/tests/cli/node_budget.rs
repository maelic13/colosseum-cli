//! With a node budget, the end of a run says how each side kept to it: its
//! largest reported node count and how many of its moves reported more than
//! its budget, in `results.txt`, the run record's telemetry and `stats`.
//!
//! The fixture engine reports whatever node count it is told to, so one side
//! ignores its budget and the other keeps to it.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_colosseum-uci-fixture"))
}

/// A four-game match at 1,000 nodes a move for both sides; `a_nodes` and
/// `b_nodes` are the node counts each side's searches report.
fn budgeted_match(run: &Path, a_nodes: u64, b_nodes: u64) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_colosseum-cli"))
        .arg("match")
        .arg(fixture())
        .arg(fixture())
        .args([
            "--games",
            "4",
            "--a-label",
            "Alpha",
            "--b-label",
            "Beta",
            "--a-engine-arg=--legal-sequence",
            "--b-engine-arg=--legal-sequence",
            "--a-nodes",
            "1000",
            "--b-nodes",
            "1000",
            "--max-moves",
            "2",
            "--seed",
            "7",
            "--placement",
            "off",
            "--json",
        ])
        .arg(format!("--a-engine-arg=--reported-nodes={a_nodes}"))
        .arg(format!("--b-engine-arg=--reported-nodes={b_nodes}"))
        .arg("--dir")
        .arg(run)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

/// The telemetry of the engine named `name`.
fn engine<'a>(telemetry: &'a Value, name: &str) -> &'a Value {
    telemetry["engines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|engine| engine["engine"] == name)
        .unwrap_or_else(|| panic!("no {name} in {telemetry}"))
}

#[test]
fn an_engine_that_ignores_its_node_budget_is_named_with_its_moves_over() {
    let root = tempfile::tempdir().unwrap();
    let run = root.path().join("run");
    let output = budgeted_match(&run, 5_000, 1_000);
    let telemetry = &output["results"]["telemetry"];

    // Four games of four plies: each side searches eight moves.
    let alpha = &engine(telemetry, "Alpha")["node_budget"];
    assert_eq!(
        *alpha,
        serde_json::json!({
            "budget": 1000, "moves_with_budget": 8, "reported_moves": 8,
            "max_nodes": 5000, "over_budget": 8
        })
    );
    // Exactly the budget is within it.
    let beta = &engine(telemetry, "Beta")["node_budget"];
    assert_eq!(beta["max_nodes"], 1000, "{beta}");
    assert_eq!(beta["over_budget"], 0, "{beta}");

    let text = std::fs::read_to_string(run.join("results.txt")).unwrap();
    assert!(
        text.lines().any(|line| line
            == "Alpha: node budget 1,000: largest 5,000 nodes reported; 8 of 8 moves over"),
        "{text}"
    );
    assert!(
        text.lines().any(|line| line
            == "Beta: node budget 1,000: largest 1,000 nodes reported; 0 of 8 moves over"),
        "{text}"
    );

    // The run record and `stats` read the same from the directory.
    let record: Value =
        serde_json::from_slice(&std::fs::read(run.join("run-record.json")).unwrap()).unwrap();
    assert_eq!(record["results"]["telemetry"], *telemetry);
    let stats = Command::new(env!("CARGO_BIN_EXE_colosseum-cli"))
        .arg("stats")
        .arg(&run)
        .output()
        .unwrap();
    let stats = String::from_utf8_lossy(&stats.stdout);
    assert!(
        stats.contains("Alpha: node budget 1,000: largest 5,000 nodes reported; 8 of 8 moves over"),
        "{stats}"
    );
}

#[test]
fn a_compliant_run_reports_no_moves_over_its_node_budget() {
    let root = tempfile::tempdir().unwrap();
    let run = root.path().join("run");
    let output = budgeted_match(&run, 1, 1);
    let telemetry = &output["results"]["telemetry"];
    for name in ["Alpha", "Beta"] {
        let budget = &engine(telemetry, name)["node_budget"];
        assert_eq!(budget["over_budget"], 0, "{name}: {budget}");
        assert_eq!(budget["max_nodes"], 1, "{name}: {budget}");
    }
    let text = std::fs::read_to_string(run.join("results.txt")).unwrap();
    for name in ["Alpha", "Beta"] {
        assert!(
            text.lines().any(|line| line
                == format!(
                    "{name}: node budget 1,000: largest 1 node reported; 0 of 8 moves over"
                )),
            "{text}"
        );
    }
}
