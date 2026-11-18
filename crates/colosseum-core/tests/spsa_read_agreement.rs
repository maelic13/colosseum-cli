//! `spsa read` against the engine projects' reference reader on five
//! recorded tunes (PLAN 12.11; `docs/architecture/spsa-read-definitions.md`).
//!
//! Each fixture is a finished Basilisk tune reduced by
//! `tools/spsa_read_fixtures.py` to its schedule, starting centres and
//! scores, with the reference's figures at the pinned revision. The centres
//! are re-derived with the runner's own update and must be the recorded ones
//! exactly, read with exact float parsing as the CLI reads them; every
//! deterministic figure must agree with the reference at its
//! rounding; the null SDs, drawn from the CLI's own stream, must agree with
//! the reference's within sampling error and with the values committed here.

use colosseum_core::spsa_read::{
    MOVE_TOLERANCE, SpsaRead, SpsaReadOptions, SpsaReadRun, read, worst_move_deviation,
};
use colosseum_core::{SpsaKnob, SpsaScheduleArtifact, prepare_iteration, update_centers};
use serde_json::Value;

const FIXTURES: [(&str, &str); 5] = [
    (
        "b33-spsa-block1",
        include_str!("../../../tests/fixtures/spsa-read/b33-spsa-block1.json"),
    ),
    (
        "b46-spsa-block1",
        include_str!("../../../tests/fixtures/spsa-read/b46-spsa-block1.json"),
    ),
    (
        "b54-spsa-block1",
        include_str!("../../../tests/fixtures/spsa-read/b54-spsa-block1.json"),
    ),
    (
        "b23-spsa-block1",
        include_str!("../../../tests/fixtures/spsa-read/b23-spsa-block1.json"),
    ),
    (
        "b23-spsa-block2",
        include_str!("../../../tests/fixtures/spsa-read/b23-spsa-block2.json"),
    ),
];

/// The CLI's own null SDs (since iteration 0, then over the window) for each
/// fixture's two reads, at the default 40 re-draws from seed 1.
const CLI_NULL_SDS: [(&str, [[f64; 2]; 2]); 5] = [
    (
        "b33-spsa-block1",
        [
            [5.441398844242583, 2.9314820128333414],
            [5.6050223876798855, 3.2138075567885482],
        ],
    ),
    (
        "b46-spsa-block1",
        [
            [5.017512897685374, 2.8067483375947595],
            [3.936922678610024, 3.6006432777145565],
        ],
    ),
    (
        "b54-spsa-block1",
        [
            [3.054455130872456, 1.6420776416172091],
            [2.6215417447907785, 1.942528175851933],
        ],
    ),
    (
        "b23-spsa-block1",
        [
            [9.867457127438808, 5.054088887772805],
            [8.941722180798843, 5.6422617338406305],
        ],
    ),
    (
        "b23-spsa-block2",
        [
            [8.680289488808455, 4.63475479935731],
            [7.700203563310394, 4.643698715088148],
        ],
    ),
];

struct Fixture {
    names: Vec<String>,
    knobs: Vec<SpsaKnob>,
    artifact: SpsaScheduleArtifact,
    games: u32,
    centres: Vec<Vec<f64>>,
    scores: Vec<i32>,
    reference: Value,
}

fn fixture(text: &str) -> Fixture {
    let value: Value = serde_json::from_str(text).unwrap();
    let artifact: SpsaScheduleArtifact = serde_json::from_value(value["schedule"].clone()).unwrap();
    let knobs = artifact
        .knobs
        .iter()
        .map(|knob| knob.knob().unwrap())
        .collect::<Vec<_>>();
    let scores = value["scores"]
        .as_array()
        .unwrap()
        .iter()
        .map(|score| i32::try_from(score.as_i64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    // The runner's own update, from the start, over every score.
    let mut centres = vec![
        value["start"]
            .as_array()
            .unwrap()
            .iter()
            .map(|centre| centre.as_f64().unwrap())
            .collect::<Vec<_>>(),
    ];
    for (k, score) in scores.iter().enumerate() {
        let prepared = prepare_iteration(
            artifact.schedule,
            artifact.perturbations.master_seed,
            k as u32,
            centres.last().unwrap(),
            &knobs,
        )
        .unwrap();
        let next = update_centers(centres.last().unwrap(), &knobs, &prepared, *score).unwrap();
        centres.push(next);
    }
    let mut worst = 0.0_f64;
    for (iteration, recorded) in value["recorded_centres"].as_object().unwrap() {
        let k: usize = iteration.parse().unwrap();
        let recorded = recorded
            .as_array()
            .unwrap()
            .iter()
            .map(|centre| centre.as_f64().unwrap())
            .collect::<Vec<_>>();
        if k == 0 {
            assert_eq!(recorded, centres[0]);
            continue;
        }
        let deviation = worst_move_deviation(
            artifact.schedule,
            &knobs,
            u32::try_from(k - 1).unwrap(),
            &centres[k],
            &recorded,
        )
        .unwrap();
        worst = worst.max(deviation);
    }
    assert!(
        worst == 0.0,
        "the runner's update does not reproduce the recorded centres: {worst:e} of a unit move (tolerance {MOVE_TOLERANCE:e})"
    );
    Fixture {
        names: artifact
            .knobs
            .iter()
            .map(|knob| knob.name.clone())
            .collect(),
        knobs,
        games: u32::try_from(value["games_per_iteration"].as_u64().unwrap()).unwrap(),
        artifact,
        centres,
        scores,
        reference: value["reference"].clone(),
    }
}

fn read_window(fixture: &Fixture, window: &Value) -> SpsaRead {
    let run = SpsaReadRun {
        names: &fixture.names,
        knobs: &fixture.knobs,
        schedule: fixture.artifact.schedule,
        master_seed: fixture.artifact.perturbations.master_seed,
        games_per_iteration: fixture.games,
        centres: &fixture.centres,
        scores: &fixture.scores,
    };
    let bound = |index: usize| u32::try_from(window[index].as_u64().unwrap()).unwrap();
    read(
        &run,
        SpsaReadOptions {
            from: Some(bound(0)),
            to: Some(bound(1)),
            ..SpsaReadOptions::default()
        },
    )
    .unwrap()
}

/// `ours` rounds to the reference's figure at `places` decimals.
#[track_caller]
fn agrees(label: &str, ours: f64, reference: &Value, places: i32) {
    let reference = reference
        .as_f64()
        .unwrap_or_else(|| panic!("{label}: {reference}"));
    let half_unit = 0.5 * 10f64.powi(-places);
    assert!(
        (ours - reference).abs() <= half_unit * (1.0 + 1e-9),
        "{label}: {ours} does not round to the reference's {reference} at {places} places"
    );
}

#[test]
fn five_recorded_tunes_read_as_the_reference_reads_them() {
    for (name, text) in FIXTURES {
        let fixture = fixture(text);
        for expected in fixture.reference["reads"].as_array().unwrap() {
            let ours = read_window(&fixture, &expected["window"]);
            let at = |field: &str| format!("{name} {}: {field}", expected["window"]);
            agrees(
                &at("line"),
                ours.familywise_line,
                &expected["familywise_line"],
                4,
            );
            agrees(&at("max |z|"), ours.max_abs_z, &expected["max_abs_z"], 4);
            assert_eq!(
                serde_json::json!(ours.over_line),
                expected["over_line"],
                "{}",
                at("over the line")
            );
            agrees(&at("excess"), ours.excess, &expected["excess"], 3);
            agrees(
                &at("excess SD"),
                ours.excess_in_sd,
                &expected["excess_sd"],
                3,
            );
            assert_eq!(ours.live, expected["live"], "{}", at("live"));
            agrees(
                &at("floor"),
                ours.quiet_rules_out.coordinate_elo,
                &expected["coordinate_elo"],
                2,
            );
            agrees(
                &at("SE"),
                ours.quiet_rules_out.coordinate_se_elo,
                &expected["coordinate_se_elo"],
                3,
            );
            agrees(
                &at("gain since 0"),
                ours.gain.since_start.elo,
                &expected["gain_since_start"],
                2,
            );
            agrees(
                &at("gain in the window"),
                ours.gain.window.elo,
                &expected["gain_window"],
                2,
            );
            match (&ours.forecast, expected["forecast"].as_object()) {
                (Some(forecast), Some(reference)) => {
                    assert_eq!(
                        serde_json::json!(forecast.span),
                        reference["segment"],
                        "{}",
                        at("forecast span")
                    );
                    agrees(&at("forecast"), forecast.elo, &reference["elo"], 2);
                    agrees(&at("forecast SD"), forecast.sd, &reference["sd"], 2);
                    agrees(
                        &at("forecast floor"),
                        ours.quiet_rules_out.forecast_two_sd_elo.unwrap(),
                        &expected["forecast_two_sd_elo"],
                        2,
                    );
                }
                (None, None) => assert!(expected["forecast_two_sd_elo"].is_null()),
                (ours, reference) => panic!("{}: {ours:?} against {reference:?}", at("forecast")),
            }
            assert_eq!(
                serde_json::json!(ours.rails),
                expected["rails"],
                "{}",
                at("rails")
            );
            let coordinates = expected["coordinates"].as_array().unwrap();
            assert_eq!(coordinates.len(), ours.coordinates.len());
            for (mine, theirs) in ours.coordinates.iter().zip(coordinates) {
                let label = |field: &str| at(&format!("{} {field}", mine.name));
                assert_eq!(mine.name, theirs["name"]);
                agrees(&label("z"), mine.z, &theirs["z"], 4);
                agrees(
                    &label("Elo"),
                    mine.elo_across_perturbation,
                    &theirs["elo_across_perturbation"],
                    3,
                );
                assert_eq!(mine.value, theirs["theta"], "{}", label("value"));
                agrees(
                    &label("clamp share"),
                    mine.clamp_share,
                    &theirs["arm_clamped_share"],
                    5,
                );
                assert_eq!(mine.rail, theirs["rail"], "{}", label("rail"));
            }
        }
    }
}

/// The null SD is a sampling estimate: the CLI's re-draws and the
/// reference's are independent, each with a relative SE of about
/// `1 / sqrt(2 × 39)` at 40 re-draws, so their difference stays within
/// three of its SEs, `3 × sqrt(2) × 11%` of the reference. The widest gap,
/// b23 block 1 since iteration 0 at 1,500 (the CLI's 8.94 against 6.22), is
/// the reference's draw: its own 200-draw estimates there are 8.4 to 9.4.
#[test]
fn the_null_sds_agree_within_sampling_error_and_are_the_committed_ones() {
    let mut observed = Vec::new();
    for ((name, text), (committed_name, committed)) in FIXTURES.iter().zip(CLI_NULL_SDS) {
        assert_eq!(*name, committed_name);
        let fixture = fixture(text);
        let mut reads = Vec::new();
        for (expected, committed) in fixture.reference["reads"]
            .as_array()
            .unwrap()
            .iter()
            .zip(committed)
        {
            let ours = read_window(&fixture, &expected["window"]);
            let reference = expected["gain_since_start_null_sd"].as_f64().unwrap();
            let relative_se = 1.0 / (2.0 * 39.0_f64).sqrt();
            assert!(
                (ours.gain.since_start.null_sd - reference).abs()
                    <= 3.0 * 2.0_f64.sqrt() * relative_se * reference,
                "{name}: null SD {} against the reference's {reference}",
                ours.gain.since_start.null_sd
            );
            reads.push([ours.gain.since_start.null_sd, ours.gain.window.null_sd]);
            let _ = committed;
        }
        observed.push((name, reads));
    }
    for ((name, reads), (_, committed)) in observed.iter().zip(CLI_NULL_SDS) {
        for (read, committed) in reads.iter().zip(committed) {
            for (ours, committed) in read.iter().zip(committed) {
                assert!(
                    (ours - committed).abs() < 1e-9,
                    "{name}: the CLI's null SDs are {reads:?}"
                );
            }
        }
    }
}
