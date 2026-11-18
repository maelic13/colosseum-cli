//! `spsa read`: whether a tune still has gradient, read from the run
//! directory without changing it (PLAN 12.11;
//! `docs/architecture/spsa-read-definitions.md`).

use super::*;

use colosseum_core::spsa_read::{
    MOVE_TOLERANCE, SpsaRead, SpsaReadError, SpsaReadOptions, SpsaReadRun, read,
    worst_move_deviation,
};

#[derive(Debug, Args)]
pub(crate) struct SpsaReadCommand {
    /// Self-contained SPSA run directory to read without mutation.
    pub(crate) run_directory: PathBuf,
    /// First iteration of the window. Default: 1,000 before its end, or 0.
    #[arg(long)]
    pub(crate) from: Option<u32>,
    /// One past the window's last iteration. Default: every completed
    /// iteration.
    #[arg(long)]
    pub(crate) to: Option<u32>,
    /// Sign-randomised re-draws behind each gain read's null SD.
    #[arg(long, default_value_t = 40, value_parser = clap::value_parser!(u32).range(2..))]
    pub(crate) null_reps: u32,
    /// Seed of the re-draws' `spsa-read-null` stream.
    #[arg(long, default_value_t = 1)]
    pub(crate) null_seed: u64,
    /// Iterations per segment of the cross-fitted gain read.
    #[arg(long, default_value_t = 500, value_parser = clap::value_parser!(u32).range(2..))]
    pub(crate) gain_segment: u32,
    /// Iterations the forecast covers after the window; 0 turns it off.
    #[arg(long, default_value_t = 2_000)]
    pub(crate) forecast_span: u32,
    /// Family alpha of the familywise line.
    #[arg(long, default_value_t = 0.05)]
    pub(crate) family_alpha: f64,
    /// The excess's line, in units of its null SD.
    #[arg(long, default_value_t = 2.0)]
    pub(crate) excess_sd: f64,
}

/// A read as `spsa read` reports it: the core's read, and the run's own
/// status, faults and integrity.
#[derive(Debug, Serialize)]
pub(crate) struct SpsaReadReport {
    pub(crate) run_status: RunStatus,
    #[serde(flatten)]
    pub(crate) read: SpsaRead,
    /// The run's own fault counters, from its checkpoint; they decide
    /// nothing.
    pub(crate) faults: MatchFaultCounts,
    pub(crate) integrity: SpsaReadIntegrity,
}

/// The replay against every record of centres the run keeps.
#[derive(Debug, Serialize)]
pub(crate) struct SpsaReadIntegrity {
    /// What the replay was compared with.
    pub(crate) compared: Vec<String>,
    /// The worst deviation, in unit moves `c_k r_k`.
    pub(crate) worst_deviation: f64,
    pub(crate) tolerance: f64,
}

/// Exit status of an inconsistent run: the replay is not the run's
/// recorded centres.
const INCONSISTENT: u8 = 3;

pub(crate) fn run_spsa_read(command: &SpsaReadCommand, machine: bool) -> ExitCode {
    let inspected = match inspect_spsa_run(&command.run_directory, "SPSA read") {
        Ok(inspected) => inspected,
        Err(code) => return code,
    };
    let schedule = &inspected.workflow.schedule;
    let knobs = match schedule
        .knobs
        .iter()
        .map(SpsaDerivedKnob::knob)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(knobs) => knobs,
        Err(error) => {
            eprintln!("SPSA read failed: the stored schedule is unreadable: {error}");
            return ExitCode::FAILURE;
        }
    };
    let names = schedule
        .knobs
        .iter()
        .map(|knob| knob.name.clone())
        .collect::<Vec<_>>();
    let completed = &inspected.replay.completed_iterations;
    let mut centres = vec![inspected.workflow.bound_tune.initial_centers()];
    centres.extend(
        completed
            .iter()
            .map(|iteration| iteration.centers_after.clone()),
    );
    let scores = completed
        .iter()
        .map(|iteration| iteration.score.difference)
        .collect::<Vec<_>>();

    let snapshot: Option<Value> =
        RunDirectory::read_checkpoint_snapshot(&command.run_directory).ok();
    let integrity = match integrity(
        &command.run_directory,
        snapshot.as_ref(),
        schedule.schedule,
        &knobs,
        &centres,
    ) {
        Ok(integrity) => integrity,
        Err(message) => {
            eprintln!("SPSA read failed: {message}");
            return ExitCode::from(INCONSISTENT);
        }
    };
    if integrity.worst_deviation > MOVE_TOLERANCE {
        eprintln!(
            "SPSA read failed: the journal's replay disagrees with the centres the run recorded ({}), worst by {:.2e} of a unit move; the journal, the schedule or the seed does not belong to this run",
            integrity.compared.join(", "),
            integrity.worst_deviation
        );
        return ExitCode::from(INCONSISTENT);
    }
    let faults = snapshot
        .as_ref()
        .and_then(|payload| payload.get("faults").cloned())
        .and_then(|faults| serde_json::from_value::<MatchFaultCounts>(faults).ok())
        .unwrap_or_else(|| {
            let mut faults = MatchFaultCounts::default();
            for iteration in completed {
                faults.absorb(iteration.faults);
            }
            faults
        });

    let run = SpsaReadRun {
        names: &names,
        knobs: &knobs,
        schedule: schedule.schedule,
        master_seed: schedule.perturbations.master_seed,
        games_per_iteration: inspected.workflow.settings.games_per_iteration,
        centres: &centres,
        scores: &scores,
    };
    let options = SpsaReadOptions {
        from: command.from,
        to: command.to,
        null_reps: command.null_reps,
        null_seed: command.null_seed,
        gain_segment: command.gain_segment,
        forecast_span: command.forecast_span,
        family_alpha: command.family_alpha,
        excess_sd: command.excess_sd,
    };
    let read = match read(&run, options) {
        Ok(read) => read,
        Err(
            error @ (SpsaReadError::NoIterations
            | SpsaReadError::Window { .. }
            | SpsaReadError::Parameter(_)),
        ) => {
            eprintln!("SPSA read failed: {error}");
            return ExitCode::from(2);
        }
        Err(error) => {
            eprintln!("SPSA read failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let report = SpsaReadReport {
        run_status: inspected.record.status,
        read,
        faults,
        integrity,
    };
    if machine {
        print_json(&MachineOutput::SpsaRead {
            run_directory: &command.run_directory,
            report,
        });
    } else {
        print!("{}", spsa_read_text(&report, &command.run_directory));
    }
    ExitCode::SUCCESS
}

/// The replay against the checkpoint's centres at its completed count, and
/// a finished run's per-iteration centres in `result.json`.
fn integrity(
    root: &Path,
    snapshot: Option<&Value>,
    schedule: colosseum_core::SpsaSchedule,
    knobs: &[colosseum_core::SpsaKnob],
    centres: &[Vec<f64>],
) -> Result<SpsaReadIntegrity, String> {
    let mut compared = Vec::new();
    let mut worst = 0.0_f64;
    let mut compare = |k: usize, recorded: &[f64]| -> Result<(), String> {
        let derived = centres.get(k).ok_or_else(|| {
            format!(
                "the run records centres after {k} iterations, and its journal replays {}",
                centres.len() - 1
            )
        })?;
        if k == 0 {
            return Ok(());
        }
        let deviation = worst_move_deviation(schedule, knobs, (k - 1) as u32, derived, recorded)
            .map_err(|error| error.to_string())?;
        worst = worst.max(deviation);
        Ok(())
    };
    if let Some(payload) = snapshot {
        let count = payload
            .get("completed_iterations")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        if let Some(recorded) = payload.get("centers").and_then(Value::as_array) {
            let recorded = recorded
                .iter()
                .map(|centre| {
                    centre
                        .as_f64()
                        .ok_or("the checkpoint's centres are not numbers")
                })
                .collect::<Result<Vec<_>, _>>()?;
            compare(count, &recorded)?;
            compared.push(format!("the checkpoint's centres after {count} iterations"));
        }
    }
    if let Ok(text) = fs::read_to_string(root.join("result.json")) {
        let result: Value = serde_json::from_str(&text)
            .map_err(|error| format!("result.json does not parse: {error}"))?;
        if let Some(iterations) = result
            .pointer("/driver/completed_iterations")
            .and_then(Value::as_array)
        {
            for (k, iteration) in iterations.iter().enumerate() {
                let recorded = iteration
                    .get("centers_after")
                    .and_then(Value::as_array)
                    .ok_or("result.json has an iteration without centres")?
                    .iter()
                    .map(|centre| {
                        centre
                            .as_f64()
                            .ok_or("result.json's centres are not numbers")
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                compare(k + 1, &recorded)?;
            }
            compared.push(format!(
                "result.json's centres after each of {} iterations",
                iterations.len()
            ));
        }
    }
    Ok(SpsaReadIntegrity {
        compared,
        worst_deviation: worst,
        tolerance: MOVE_TOLERANCE,
    })
}

/// A summary block, then one row per coordinate by `|z|`.
pub(crate) fn spsa_read_text(report: &SpsaReadReport, run_directory: &Path) -> String {
    let read = &report.read;
    let mut lines = vec![
        format!(
            "SPSA read of {}: iterations {}-{} of {} completed ({} planned), run {:?}",
            run_directory.display(),
            read.window[0],
            read.window[1],
            read.completed_iterations,
            read.planned_iterations,
            report.run_status
        ),
        format!(
            "  {} coordinates; familywise line {:.2} (family alpha {}); max |z| {:.2}; over the line: {}",
            read.coordinate_count,
            read.familywise_line,
            read.family_alpha,
            read.max_abs_z,
            list_or_none(&read.over_line)
        ),
        format!(
            "  sum z^2 - n = {:+.1} ({:+.1} SD; line {} SD); {}",
            read.excess,
            read.excess_in_sd,
            read.excess_line_sd,
            if read.live { "LIVE" } else { "quiet" }
        ),
        format!(
            "  quiet rules out: no coordinate above {:.1} Elo across a perturbation (SE {:.2}){}",
            read.quiet_rules_out.coordinate_elo,
            read.quiet_rules_out.coordinate_se_elo,
            read.quiet_rules_out
                .forecast_two_sd_elo
                .map_or_else(String::new, |floor| format!(
                    "; the forecast's 2 SD is {floor:.1} Elo"
                ))
        ),
        format!(
            "  gain since iteration 0: {:+.1} Elo (null SD {:.1}); in the window: {:+.1} Elo (null SD {:.1}); {} sign-randomised re-draws, seed {}",
            read.gain.since_start.elo,
            read.gain.since_start.null_sd,
            read.gain.window.elo,
            read.gain.window.null_sd,
            read.gain.null_reps,
            read.gain.null_seed
        ),
        match &read.forecast {
            Some(forecast) => format!(
                "  forecast for iterations {}-{}: at most {:+.1} +/- {:.1} Elo (an upper bound)",
                forecast.span[0], forecast.span[1], forecast.elo, forecast.sd
            ),
            None => "  forecast: none".to_owned(),
        },
        format!(
            "  rails: {}; arms clamped on over 1% of iterations: {}",
            list_or_none(&read.rails),
            list_or_none(&read.clamped_over_one_percent)
        ),
        format!("  faults: {}", fault_text(&report.faults)),
        format!(
            "  integrity: worst deviation {:.1e} of a unit move against {}",
            report.integrity.worst_deviation,
            if report.integrity.compared.is_empty() {
                "nothing recorded".to_owned()
            } else {
                report.integrity.compared.join(" and ")
            }
        ),
        "  coordinates by |z| (z, Elo across one perturbation, centre, start, value, clamp share):"
            .to_owned(),
    ];
    let mut coordinates = read.coordinates.iter().collect::<Vec<_>>();
    coordinates.sort_by(|a, b| b.z.abs().total_cmp(&a.z.abs()));
    let width = coordinates
        .iter()
        .map(|coordinate| coordinate.name.len())
        .max()
        .unwrap_or(0);
    for coordinate in coordinates {
        lines.push(format!(
            "    {:width$}  z {:+6.2}  {:+7.2} Elo  {:12.4}  {:12.4}  {:>8}  {:5.1}%{}",
            coordinate.name,
            coordinate.z,
            coordinate.elo_across_perturbation,
            coordinate.centre,
            coordinate.start,
            coordinate.value,
            coordinate.clamp_share * 100.0,
            if coordinate.rail { "  RAIL" } else { "" }
        ));
    }
    lines.join("\n") + "\n"
}

fn list_or_none(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

fn fault_text(faults: &MatchFaultCounts) -> String {
    let parts = [
        ("engine a", faults.engine_a),
        ("engine b", faults.engine_b),
        ("time losses a", faults.time_losses_a),
        ("time losses b", faults.time_losses_b),
        ("infrastructure", faults.infrastructure),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(label, count)| format!("{label} {count}"))
    .collect::<Vec<_>>();
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(", ")
    }
}
