//! `spsa read` on the run directory a tune leaves behind: the window, the
//! schema, the integrity check against the run's own records, mid-run reads
//! and the failures. The figures themselves are pinned in the core, against
//! the engine projects' reference on recorded tunes.

use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_colosseum-cli"))
}

/// A synthetic two-knob tune of `iterations`, stopped after `stop` of them
/// when given one.
fn tune(root: &Path, run: &Path, iterations: u32, stop: Option<u32>) {
    let tune = root.join("tune.toml");
    std::fs::write(
        &tune,
        r#"
[[parameters]]
name = "Hash"
initial = 16
min = 1
max = 1024
c_end = 1.0

[[parameters]]
name = "Threads"
initial = 2
min = 1
max = 8
c_end = 1.0
"#,
    )
    .unwrap();
    let mut command = cli();
    if let Some(stop) = stop {
        command.args(["--__stop-after-units", &stop.to_string()]);
    }
    let output = command
        .arg("--json")
        .arg("spsa")
        .arg(env!("CARGO_BIN_EXE_colosseum-cli"))
        .arg("--engine-arg=__uci-stub")
        .arg("--tune")
        .arg(&tune)
        .args([
            "--r-end",
            "0.002",
            "--iterations",
            &iterations.to_string(),
            "--games-per-iteration",
            "2",
            "--depth",
            "1",
            "--seed",
            "7",
            "--__synthetic-games",
            "--dir",
        ])
        .arg(run)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(if stop.is_some() { 6 } else { 0 }),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn spsa_read(run: &Path, extra: &[&str]) -> Output {
    cli()
        .args(["--json", "spsa", "read"])
        .arg(run)
        .args(extra)
        .output()
        .unwrap()
}

fn report(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["type"], "spsa-read", "{value}");
    value["report"].clone()
}

fn history(run: &Path) -> Vec<Value> {
    let output = cli()
        .args(["--json", "spsa", "history"])
        .arg(run)
        .output()
        .unwrap();
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    value["report"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["centers"].clone())
        .collect()
}

#[test]
fn a_finished_tune_reads_its_whole_run_and_matches_its_own_records() {
    let root = tempfile::tempdir().unwrap();
    let run = root.path().join("run");
    tune(root.path(), &run, 40, None);
    let read = report(&spsa_read(&run, &[]));
    assert_eq!(read["schema"], "colosseum-spsa-read-v1");
    assert_eq!(read["run_status"], "completed");
    // Fewer than 1,000 completed: the window is the whole run.
    assert_eq!(read["window"], serde_json::json!([0, 40]));
    assert_eq!(read["completed_iterations"], 40);
    assert_eq!(read["planned_iterations"], 40);
    assert_eq!(read["coordinate_count"], 2);
    assert_eq!(read["games_per_iteration"], 2);
    assert_eq!(read["family_alpha"], 0.05);
    assert_eq!(read["excess_line_sd"], 2.0);
    assert_eq!(read["gain"]["null_stream"], "spsa-read-null");
    assert_eq!(
        read["gain"]["since_start"]["span"],
        serde_json::json!([0, 40])
    );
    // At the end of its schedule a tune has nothing to forecast.
    assert!(read["forecast"].is_null(), "{read}");
    // The replay is the run's own centres, the checkpoint's and result.json's.
    let integrity = &read["integrity"];
    assert_eq!(integrity["worst_deviation"], 0.0, "{integrity}");
    assert_eq!(
        integrity["compared"].as_array().unwrap().len(),
        2,
        "{integrity}"
    );
    assert_eq!(read["faults"]["engine_a"], 0);
    // Each coordinate's centre and start are the history's.
    let rows = history(&run);
    let coordinates = read["coordinates"].as_array().unwrap();
    assert_eq!(coordinates[0]["name"], "Hash");
    assert_eq!(coordinates[1]["name"], "Threads");
    for (index, coordinate) in coordinates.iter().enumerate() {
        assert_eq!(coordinate["start"], rows[0][index]);
        assert_eq!(coordinate["centre"], rows[40][index]);
        assert!(coordinate["z"].is_number());
        assert!(coordinate["clamp_share"].is_number());
        assert!(coordinate.get("rail").is_some());
    }

    // A window inside the run, and the same text read.
    let window = report(&spsa_read(&run, &["--from", "10", "--to", "30"]));
    assert_eq!(window["window"], serde_json::json!([10, 30]));
    assert_eq!(
        window["gain"]["window"]["span"],
        serde_json::json!([10, 30])
    );
    assert_eq!(
        window["gain"]["since_start"]["span"],
        serde_json::json!([0, 30])
    );
    assert_eq!(window["forecast"]["span"], serde_json::json!([30, 40]));
    let text = cli().args(["spsa", "read"]).arg(&run).output().unwrap();
    assert!(text.status.success());
    let text = String::from_utf8(text.stdout).unwrap();
    for line in [
        "familywise line",
        "sum z^2 - n",
        "gain since iteration 0",
        "integrity",
    ] {
        assert!(text.contains(line), "{line}: {text}");
    }
    assert!(
        text.lines()
            .any(|line| line.trim_start().starts_with("Hash ")),
        "{text}"
    );
}

#[test]
fn a_tune_read_while_it_runs_reads_only_its_completed_iterations() {
    let root = tempfile::tempdir().unwrap();
    let run = root.path().join("run");
    tune(root.path(), &run, 40, Some(12));
    let read = report(&spsa_read(&run, &[]));
    assert_eq!(read["completed_iterations"], 12);
    assert_eq!(read["planned_iterations"], 40);
    assert_eq!(read["window"], serde_json::json!([0, 12]));
    assert_eq!(read["forecast"]["span"], serde_json::json!([12, 40]));
    assert_eq!(read["forecast"]["upper_bound"], true);
    assert_eq!(read["integrity"]["worst_deviation"], 0.0);
    // Past the completed iterations is a usage error.
    assert_eq!(spsa_read(&run, &["--to", "13"]).status.code(), Some(2));
    assert_eq!(
        spsa_read(&run, &["--from", "8", "--to", "8"]).status.code(),
        Some(2)
    );
}

#[test]
fn recorded_centres_off_the_replay_fail_loudly() {
    let root = tempfile::tempdir().unwrap();
    let run = root.path().join("run");
    tune(root.path(), &run, 20, None);
    let path = run.join("result.json");
    let mut result: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let centre = &mut result["driver"]["completed_iterations"][5]["centers_after"][0];
    let moved = centre.as_f64().unwrap() + 0.5;
    *centre = serde_json::json!(moved);
    std::fs::write(&path, serde_json::to_vec(&result).unwrap()).unwrap();
    let output = spsa_read(&run, &[]);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("disagrees"), "{stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn a_journal_or_seed_that_is_not_the_runs_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let run = root.path().join("run");
    tune(root.path(), &run, 20, None);

    // A game's result changed after the checkpoint: the journal no longer
    // matches it.
    let journal = run.join("games.jsonl");
    let original = std::fs::read_to_string(&journal).unwrap();
    let tampered = original.replacen("\"WhiteWin\"", "\"BlackWin\"", 1);
    let tampered = if tampered == original {
        original.replacen("\"Draw\"", "\"WhiteWin\"", 1)
    } else {
        tampered
    };
    assert_ne!(tampered, original);
    std::fs::write(&journal, &tampered).unwrap();
    let output = spsa_read(&run, &[]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    std::fs::write(&journal, &original).unwrap();
    report(&spsa_read(&run, &[]));

    // Another seed in the schedule: it no longer is the run's.
    let schedule = run.join("spsa-schedule.json");
    let mut artifact: Value = serde_json::from_slice(&std::fs::read(&schedule).unwrap()).unwrap();
    let seed = artifact["perturbations"]["master_seed"].as_u64().unwrap();
    artifact["perturbations"]["master_seed"] = serde_json::json!(seed + 1);
    std::fs::write(&schedule, serde_json::to_vec(&artifact).unwrap()).unwrap();
    let output = spsa_read(&run, &[]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn a_directory_that_is_not_a_tune_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let output = spsa_read(root.path(), &[]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("SPSA read failed"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
