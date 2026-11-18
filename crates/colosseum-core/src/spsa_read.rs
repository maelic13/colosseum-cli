//! `spsa read`: whether a tune still has gradient, read from its own signs,
//! schedule and scores.
//!
//! Every definition is pinned in `docs/architecture/spsa-read-definitions.md`
//! against the engine projects' reference reader. The read is pure: it takes
//! a run's centres and scores and draws its signs from the run's own
//! perturbation stream, so the caller supplies no sign and no gain.

use serde::Serialize;
use thiserror::Error;

use crate::rng::{NamedRng, stream_names};
use crate::spsa::{
    SpsaError, SpsaKnob, SpsaSchedule, arm_clamp, perturbations_for_iteration,
    round_half_away_from_zero,
};
use crate::stats::{StatisticsError, inverse_normal_cdf};

/// The versioned schema of a read.
pub const SPSA_READ_SCHEMA: &str = "colosseum-spsa-read-v1";

/// Elo per unit of expected score near an even score, as the reference
/// writes it: `1600 / ln 10` is 694.87.
pub const ELO_PER_SCORE: f64 = 695.0;

/// A run as the read needs it, in tune-file knob order.
#[derive(Debug, Clone, Copy)]
pub struct SpsaReadRun<'a> {
    pub names: &'a [String],
    pub knobs: &'a [SpsaKnob],
    pub schedule: SpsaSchedule,
    /// The master seed of the run's `spsa-perturbations` stream.
    pub master_seed: u64,
    pub games_per_iteration: u32,
    /// `centres[0]` is the start; `centres[k + 1]` the centres after
    /// iteration `k`. One more than `scores`.
    pub centres: &'a [Vec<f64>],
    /// `D[k]`: the plus arm's wins minus its losses at iteration `k`.
    pub scores: &'a [i32],
}

/// The read's parameters; [`Default`] is the documented defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpsaReadOptions {
    /// First iteration of the window; `None` is `to − 1,000`, at least 0.
    pub from: Option<u32>,
    /// One past the last iteration of the window; `None` is every completed
    /// iteration.
    pub to: Option<u32>,
    pub null_reps: u32,
    pub null_seed: u64,
    pub gain_segment: u32,
    /// Iterations the forecast covers; 0 turns it off.
    pub forecast_span: u32,
    pub family_alpha: f64,
    /// The excess's line in units of its null SD.
    pub excess_sd: f64,
}

/// The default window: the last 1,000 completed iterations.
pub const DEFAULT_WINDOW: u32 = 1_000;

impl Default for SpsaReadOptions {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            null_reps: 40,
            null_seed: 1,
            gain_segment: 500,
            forecast_span: 2_000,
            family_alpha: 0.05,
            excess_sd: 2.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SpsaReadError {
    #[error("the tune has no completed iteration to read")]
    NoIterations,
    #[error(
        "the window {from}..{to} is not inside the {completed} completed iterations (it needs from < to <= {completed})"
    )]
    Window { from: u32, to: u32, completed: u32 },
    #[error("{0}")]
    Parameter(String),
    #[error("the run has {centres} centre vectors for {scores} scores and {knobs} knobs")]
    Shape {
        centres: usize,
        scores: usize,
        knobs: usize,
    },
    #[error(transparent)]
    Spsa(#[from] SpsaError),
    #[error(transparent)]
    Statistics(#[from] StatisticsError),
}

/// What a read says, in knob order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpsaRead {
    pub schema: &'static str,
    /// `[from, to)`.
    pub window: [u32; 2],
    pub completed_iterations: u32,
    pub planned_iterations: u32,
    pub coordinate_count: usize,
    pub games_per_iteration: u32,
    /// Elo across one perturbation per unit of mean `D`: `695 / (2G)`.
    pub elo_per_unit_score: f64,
    pub family_alpha: f64,
    /// The `|z|` that `n` independent coordinates pass together with
    /// probability `family_alpha`.
    pub familywise_line: f64,
    pub max_abs_z: f64,
    /// The coordinates whose `|z|` reaches the familywise line.
    pub over_line: Vec<String>,
    /// `Σ z² − n`.
    pub excess: f64,
    /// The excess in units of its null SD, `sqrt(2n)`.
    pub excess_in_sd: f64,
    /// The excess's line, in null SD.
    pub excess_line_sd: f64,
    /// Some `|z|` reaches the familywise line, or the excess reaches its
    /// line.
    pub live: bool,
    pub quiet_rules_out: SpsaQuietBound,
    pub gain: SpsaGainReads,
    /// Absent when off or at the end of the schedule.
    pub forecast: Option<SpsaForecast>,
    /// Coordinates near a bound whose outward `z` reaches the line.
    pub rails: Vec<String>,
    /// Coordinates with an arm clamped on more than 1% of iterations.
    pub clamped_over_one_percent: Vec<String>,
    pub coordinates: Vec<SpsaReadCoordinate>,
}

/// What a quiet read rules out.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpsaQuietBound {
    /// No coordinate is worth more, in Elo across one perturbation: the line
    /// times the standard error.
    pub coordinate_elo: f64,
    /// A coordinate's standard error in Elo across one perturbation.
    pub coordinate_se_elo: f64,
    /// Twice the forecast's SD: its upper reading; absent with no forecast.
    pub forecast_two_sd_elo: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpsaGainReads {
    pub segment: u32,
    pub null_reps: u32,
    pub null_seed: u64,
    pub null_stream: &'static str,
    /// Since iteration 0, to the window's end.
    pub since_start: SpsaGainRead,
    /// Over the window.
    pub window: SpsaGainRead,
}

/// The cross-fitted gain over a span, in Elo, and its sign-randomised null
/// SD.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpsaGainRead {
    pub span: [u32; 2],
    pub elo: f64,
    pub null_sd: f64,
}

/// An upper bound on the next iterations' gain, from the window's excess
/// held constant through the schedule.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpsaForecast {
    pub span: [u32; 2],
    pub elo: f64,
    pub sd: f64,
    pub upper_bound: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpsaReadCoordinate {
    pub name: String,
    pub z: f64,
    pub elo_across_perturbation: f64,
    /// The centre at the window's end, and the value the runner sends for
    /// it.
    pub centre: f64,
    pub value: i64,
    pub start: f64,
    /// The perturbation at the window's last iteration.
    pub perturbation: f64,
    /// To the nearer bound.
    pub distance_to_bound: f64,
    /// `z` toward the bound the centre is within one perturbation of; absent
    /// when it is near neither.
    pub outward_z: Option<f64>,
    pub rail: bool,
    /// Iterations `[0, to)` on which either arm left the range.
    pub clamp_share: f64,
}

/// Over this, a recorded centre is not the runner's update of the scores:
/// the reference's tolerance, in unit moves.
pub const MOVE_TOLERANCE: f64 = 1e-6;

/// How far `recorded` centres after iteration `iteration` are from the
/// `derived` ones, at the worst coordinate, in units of that iteration's unit
/// move `c_k r_k`: zero when the run's own arithmetic wrote them, a last-bit
/// difference when a runner of another version did.
pub fn worst_move_deviation(
    schedule: SpsaSchedule,
    knobs: &[SpsaKnob],
    iteration: u32,
    derived: &[f64],
    recorded: &[f64],
) -> Result<f64, SpsaReadError> {
    if derived.len() != knobs.len() || recorded.len() != knobs.len() {
        return Err(SpsaReadError::Shape {
            centres: recorded.len(),
            scores: derived.len(),
            knobs: knobs.len(),
        });
    }
    let mut worst = 0.0_f64;
    for ((knob, derived), recorded) in knobs.iter().zip(derived).zip(recorded) {
        let coefficients = schedule.coefficients(iteration, *knob)?;
        let unit = (coefficients.c * coefficients.r).max(1e-12);
        worst = worst.max((derived - recorded).abs() / unit);
    }
    Ok(worst)
}

/// The familywise line for `n` coordinates at family alpha `alpha`.
pub fn familywise_line(n: usize, alpha: f64) -> Result<f64, SpsaReadError> {
    if n == 0 || !(alpha > 0.0 && alpha < 1.0) {
        return Err(SpsaReadError::Parameter(format!(
            "the family alpha must be in (0, 1) and a tune needs a coordinate; got {alpha} for {n}"
        )));
    }
    let per = 1.0 - (1.0 - alpha).powf(1.0 / n as f64);
    Ok(inverse_normal_cdf(1.0 - per / 2.0)?)
}

/// The run's gains and signs at every completed iteration.
struct Prepared {
    /// `c[k][i]`.
    c: Vec<Vec<f64>>,
    /// `r[k][i]`.
    r: Vec<Vec<f64>>,
    /// `δ[k][i]`.
    delta: Vec<Vec<i8>>,
}

fn prepare(run: &SpsaReadRun<'_>) -> Result<Prepared, SpsaReadError> {
    let completed = run.scores.len();
    let mut prepared = Prepared {
        c: Vec::with_capacity(completed),
        r: Vec::with_capacity(completed),
        delta: Vec::with_capacity(completed),
    };
    for k in 0..completed {
        let iteration = u32::try_from(k).map_err(|_| SpsaError::DrawIndexOverflow)?;
        let coefficients = run
            .knobs
            .iter()
            .map(|knob| run.schedule.coefficients(iteration, *knob))
            .collect::<Result<Vec<_>, _>>()?;
        prepared.c.push(coefficients.iter().map(|c| c.c).collect());
        prepared.r.push(coefficients.iter().map(|c| c.r).collect());
        prepared.delta.push(perturbations_for_iteration(
            run.master_seed,
            iteration,
            run.knobs.len(),
        )?);
    }
    Ok(prepared)
}

/// Coordinate `i`'s gradient over `ks`, in Elo per unit of the coordinate:
/// the least-squares slope of `D` on the signed perturbation, every
/// iteration in the denominator.
fn gradient(
    elo: f64,
    ks: &[usize],
    i: usize,
    delta: impl Fn(usize, usize) -> f64,
    scores: &[i32],
    c: &[Vec<f64>],
) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    for &k in ks {
        num += delta(k, i) * f64::from(scores[k]) * c[k][i];
        den += c[k][i] * c[k][i];
    }
    if den == 0.0 {
        0.0
    } else {
        elo * num / (2.0 * den)
    }
}

/// The gain over `[start, stop)`: in each segment, each coordinate's
/// gradient on the even iterations times its move summed over the odd ones,
/// and the reverse. A short last segment joins the one before it.
#[allow(clippy::too_many_arguments)]
fn crossfit_gain(
    elo: f64,
    start: usize,
    stop: usize,
    segment: usize,
    n: usize,
    delta: impl Fn(usize, usize) -> f64 + Copy,
    moves: impl Fn(usize, usize) -> f64,
    scores: &[i32],
    c: &[Vec<f64>],
) -> f64 {
    let mut starts = (start..stop).step_by(segment).collect::<Vec<_>>();
    if starts.len() > 1 && stop - starts[starts.len() - 1] < segment {
        starts.pop();
    }
    let mut total = 0.0;
    for (index, &lo) in starts.iter().enumerate() {
        let hi = starts.get(index + 1).copied().unwrap_or(stop);
        let even = (lo..hi).filter(|k| k % 2 == 0).collect::<Vec<_>>();
        let odd = (lo..hi).filter(|k| k % 2 == 1).collect::<Vec<_>>();
        for i in 0..n {
            let moved_odd = odd.iter().map(|&k| moves(k, i)).sum::<f64>();
            let moved_even = even.iter().map(|&k| moves(k, i)).sum::<f64>();
            total += gradient(elo, &even, i, delta, scores, c) * moved_odd
                + gradient(elo, &odd, i, delta, scores, c) * moved_even;
        }
    }
    total
}

/// The population SD of the gain over `[start, stop)` across `reps`
/// re-draws of every sign at an iteration with `D ≠ 0`, from a fresh
/// `spsa-read-null` stream.
#[allow(clippy::too_many_arguments)]
fn null_gain_sd(
    elo: f64,
    start: usize,
    stop: usize,
    segment: usize,
    reps: u32,
    seed: u64,
    run: &SpsaReadRun<'_>,
    prepared: &Prepared,
) -> f64 {
    let n = run.knobs.len();
    let mut rng = NamedRng::new(seed, stream_names::SPSA_READ_NULL)
        .expect("the built-in null stream name is valid");
    let mut values = Vec::with_capacity(reps as usize);
    let mut drawn = vec![vec![0_i8; n]; stop];
    for _ in 0..reps {
        for (k, row) in drawn.iter_mut().enumerate().take(stop).skip(start) {
            for sign in row.iter_mut() {
                *sign = if run.scores[k] == 0 {
                    0
                } else {
                    rng.rademacher()
                };
            }
        }
        let delta = |k: usize, i: usize| f64::from(drawn[k][i]);
        let moves = |k: usize, i: usize| {
            f64::from(drawn[k][i]) * prepared.r[k][i] * prepared.c[k][i] * f64::from(run.scores[k])
        };
        values.push(crossfit_gain(
            elo,
            start,
            stop,
            segment,
            n,
            delta,
            moves,
            run.scores,
            &prepared.c,
        ));
    }
    population_sd(&values)
}

fn population_sd(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt()
}

/// Elo of the gain over `[to, segment_end)` per unit of the window's excess,
/// held constant through the schedule: `Σ r_k f_k²` over the span, times the
/// window's mean `D²` and `E`, over `2 × mean f_k² × W`, with
/// `f_k = (k + 1)^−γ`; a coordinate's `c0` cancels. `None` when the span is
/// empty. Every knob of a tune shares `r_k`, so `knob` is any of them.
pub fn forecast_factor(
    schedule: SpsaSchedule,
    knob: SpsaKnob,
    [from, to]: [u32; 2],
    segment_end: u32,
    mean_d2: f64,
    elo: f64,
) -> Result<Option<f64>, SpsaReadError> {
    if segment_end <= to || to <= from {
        return Ok(None);
    }
    let gamma = schedule.gamma();
    let f2 = |k: u32| (f64::from(k) + 1.0).powf(-gamma).powi(2);
    let mut moved = 0.0;
    for k in to..segment_end {
        moved += schedule.coefficients(k, knob)?.r * f2(k);
    }
    let width = f64::from(to - from);
    let held = (from..to).map(f2).sum::<f64>() / width;
    Ok(Some(moved * mean_d2 * elo / (2.0 * held * width)))
}

/// Read a run.
pub fn read(run: &SpsaReadRun<'_>, options: SpsaReadOptions) -> Result<SpsaRead, SpsaReadError> {
    let n = run.knobs.len();
    let completed = run.scores.len();
    if n == 0 || run.names.len() != n || run.centres.len() != completed + 1 {
        return Err(SpsaReadError::Shape {
            centres: run.centres.len(),
            scores: completed,
            knobs: n,
        });
    }
    if run.centres.iter().any(|centres| centres.len() != n) {
        return Err(SpsaReadError::Shape {
            centres: run.centres.len(),
            scores: completed,
            knobs: n,
        });
    }
    if completed == 0 {
        return Err(SpsaReadError::NoIterations);
    }
    let completed_u32 = u32::try_from(completed).map_err(|_| SpsaError::DrawIndexOverflow)?;
    let to = options.to.unwrap_or(completed_u32);
    let from = options
        .from
        .unwrap_or_else(|| to.saturating_sub(DEFAULT_WINDOW));
    if from >= to || to > completed_u32 {
        return Err(SpsaReadError::Window {
            from,
            to,
            completed: completed_u32,
        });
    }
    if options.gain_segment == 0 {
        return Err(SpsaReadError::Parameter(
            "the gain segment must be at least one iteration".into(),
        ));
    }
    if !(options.excess_sd.is_finite() && options.excess_sd > 0.0) {
        return Err(SpsaReadError::Parameter(format!(
            "the excess line must be a positive number of SD; got {}",
            options.excess_sd
        )));
    }
    if run.games_per_iteration == 0 {
        return Err(SpsaReadError::Parameter(
            "a tune plays at least one game an iteration".into(),
        ));
    }
    let prepared = prepare(run)?;
    let (lo, hi) = (from as usize, to as usize);
    let width = (hi - lo) as f64;
    let elo = ELO_PER_SCORE / (2.0 * f64::from(run.games_per_iteration));
    let d = |k: usize| f64::from(run.scores[k]);

    let sum_d2 = (lo..hi).map(|k| d(k) * d(k)).sum::<f64>();
    let sums = (0..n)
        .map(|i| {
            (lo..hi)
                .map(|k| f64::from(prepared.delta[k][i]) * d(k))
                .sum::<f64>()
        })
        .collect::<Vec<_>>();
    let z = sums
        .iter()
        .map(|sum| {
            if sum_d2 == 0.0 {
                0.0
            } else {
                sum / sum_d2.sqrt()
            }
        })
        .collect::<Vec<_>>();
    let line = familywise_line(n, options.family_alpha)?;
    let null_sd = (2.0 * n as f64).sqrt();
    let excess = z.iter().map(|z| z * z).sum::<f64>() - n as f64;
    let over_line = (0..n)
        .filter(|&i| z[i].abs() >= line)
        .map(|i| run.names[i].clone())
        .collect::<Vec<_>>();
    let live = !over_line.is_empty() || excess >= options.excess_sd * null_sd;
    let se_elo = elo * sum_d2.sqrt() / width;

    let planned = run.schedule.iterations();
    let segment_end = to.saturating_add(options.forecast_span).min(planned);
    let factor = forecast_factor(
        run.schedule,
        run.knobs[0],
        [from, to],
        segment_end,
        sum_d2 / width,
        elo,
    )?;

    let segment = options.gain_segment as usize;
    let actual_delta = |k: usize, i: usize| f64::from(prepared.delta[k][i]);
    let actual_moves = |k: usize, i: usize| run.centres[k + 1][i] - run.centres[k][i];
    let gain_read = |start: usize| SpsaGainRead {
        span: [start as u32, to],
        elo: crossfit_gain(
            elo,
            start,
            hi,
            segment,
            n,
            actual_delta,
            actual_moves,
            run.scores,
            &prepared.c,
        ),
        null_sd: null_gain_sd(
            elo,
            start,
            hi,
            segment,
            options.null_reps,
            options.null_seed,
            run,
            &prepared,
        ),
    };
    let gain = SpsaGainReads {
        segment: options.gain_segment,
        null_reps: options.null_reps,
        null_seed: options.null_seed,
        null_stream: stream_names::SPSA_READ_NULL,
        since_start: gain_read(0),
        window: gain_read(lo),
    };

    let mut coordinates = Vec::with_capacity(n);
    for (i, knob) in run.knobs.iter().enumerate() {
        let centre = run.centres[hi][i];
        let perturbation = prepared.c[hi - 1][i];
        let (min, max) = (knob.min as f64, knob.max as f64);
        let near_min = centre - min <= perturbation;
        let near_max = max - centre <= perturbation;
        let outward_z = match (near_min, near_max) {
            (true, true) => Some(z[i].abs()),
            (false, true) => Some(z[i]),
            (true, false) => Some(-z[i]),
            (false, false) => None,
        };
        let clamped = (0..hi)
            .filter(|&k| {
                let clamp = arm_clamp(run.centres[k][i], prepared.c[k][i], *knob);
                clamp.at_min || clamp.at_max
            })
            .count();
        coordinates.push(SpsaReadCoordinate {
            name: run.names[i].clone(),
            z: z[i],
            elo_across_perturbation: elo * sums[i] / width,
            centre,
            value: round_half_away_from_zero(centre)?,
            start: run.centres[0][i],
            perturbation,
            distance_to_bound: (centre - min).min(max - centre),
            outward_z,
            rail: outward_z.is_some_and(|outward| outward >= line),
            clamp_share: clamped as f64 / hi as f64,
        });
    }
    let rails = coordinates
        .iter()
        .filter(|coordinate| coordinate.rail)
        .map(|coordinate| coordinate.name.clone())
        .collect();
    let clamped_over_one_percent = coordinates
        .iter()
        .filter(|coordinate| coordinate.clamp_share > 0.01)
        .map(|coordinate| coordinate.name.clone())
        .collect();

    Ok(SpsaRead {
        schema: SPSA_READ_SCHEMA,
        window: [from, to],
        completed_iterations: completed_u32,
        planned_iterations: planned,
        coordinate_count: n,
        games_per_iteration: run.games_per_iteration,
        elo_per_unit_score: elo,
        family_alpha: options.family_alpha,
        familywise_line: line,
        max_abs_z: z.iter().map(|z| z.abs()).fold(0.0, f64::max),
        over_line,
        excess,
        excess_in_sd: excess / null_sd,
        excess_line_sd: options.excess_sd,
        live,
        quiet_rules_out: SpsaQuietBound {
            coordinate_elo: line * se_elo,
            coordinate_se_elo: se_elo,
            forecast_two_sd_elo: factor.map(|factor| 2.0 * factor * null_sd),
        },
        gain,
        forecast: factor.map(|factor| SpsaForecast {
            span: [to, segment_end],
            elo: factor * excess,
            sd: factor * null_sd,
            upper_bound: true,
        }),
        rails,
        clamped_over_one_percent,
        coordinates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spsa::{SpsaEndSpec, SpsaScheduleArtifact, prepare_iteration, update_centers};

    const GAMMA: f64 = 0.102;

    /// A schedule of `iterations` whose knobs all start with `c0 = 8`.
    fn artifact(
        iterations: u32,
        r_end: f64,
        bounds: &[(i64, i64)],
        seed: u64,
    ) -> SpsaScheduleArtifact {
        let c_end = 8.0 * f64::from(iterations).powf(-GAMMA);
        let specs = bounds
            .iter()
            .enumerate()
            .map(|(i, &(min, max))| SpsaEndSpec {
                name: format!("K{i}"),
                min,
                max,
                c_end,
            })
            .collect::<Vec<_>>();
        SpsaScheduleArtifact::derive(iterations, r_end, seed, &specs).unwrap()
    }

    struct Synthetic {
        artifact: SpsaScheduleArtifact,
        names: Vec<String>,
        knobs: Vec<SpsaKnob>,
        centres: Vec<Vec<f64>>,
        scores: Vec<i32>,
        games: u32,
    }

    impl Synthetic {
        fn run(&self) -> SpsaReadRun<'_> {
            SpsaReadRun {
                names: &self.names,
                knobs: &self.knobs,
                schedule: self.artifact.schedule,
                master_seed: self.artifact.perturbations.master_seed,
                games_per_iteration: self.games,
                centres: &self.centres,
                scores: &self.scores,
            }
        }

        fn read(&self, options: SpsaReadOptions) -> SpsaRead {
            read(&self.run(), options).unwrap()
        }
    }

    /// A tune played through the runner's own update: each game is won by
    /// the plus arm with probability `0.3 + b` and lost with `0.3 - b`,
    /// `b = sum(g_i * delta_i) / 2`.
    fn simulate(
        gradients: &[f64],
        starts: &[f64],
        bounds: &[(i64, i64)],
        ceiling: u32,
        completed: u32,
        seed: u64,
    ) -> Synthetic {
        let artifact = artifact(ceiling, 0.00155, bounds, seed);
        let knobs = artifact
            .knobs
            .iter()
            .map(|knob| knob.knob().unwrap())
            .collect::<Vec<_>>();
        let mut games_rng = NamedRng::new(seed, "test-games").unwrap();
        let mut centres = vec![starts.to_vec()];
        let mut scores = Vec::new();
        let games = 6;
        for k in 0..completed {
            let prepared =
                prepare_iteration(artifact.schedule, seed, k, centres.last().unwrap(), &knobs)
                    .unwrap();
            let bias = 0.5
                * gradients
                    .iter()
                    .zip(&prepared.perturbations)
                    .map(|(g, d)| g * f64::from(*d))
                    .sum::<f64>();
            let mut score = 0;
            for _ in 0..games {
                let u = games_rng.next_u64() as f64 / 2f64.powi(64);
                if u < 0.3 + bias {
                    score += 1;
                } else if u < 0.6 {
                    score -= 1;
                }
            }
            let next = update_centers(centres.last().unwrap(), &knobs, &prepared, score).unwrap();
            centres.push(next);
            scores.push(score);
        }
        Synthetic {
            names: artifact
                .knobs
                .iter()
                .map(|knob| knob.name.clone())
                .collect(),
            artifact,
            knobs,
            centres,
            scores,
            games,
        }
    }

    fn quick() -> SpsaReadOptions {
        SpsaReadOptions {
            null_reps: 5,
            ..SpsaReadOptions::default()
        }
    }

    #[test]
    fn familywise_lines_are_the_references() {
        // Python's NormalDist().inv_cdf at the reference's formula.
        for (n, line) in [
            (8, 2.727_007_896_719_966_7),
            (15, 2.927_798_415_447_027_6),
            (29, 3.126_804_994_158_009),
            (49, 3.277_784_002_744_794),
            (82, 3.420_382_087_646_781),
        ] {
            assert!(
                (familywise_line(n, 0.05).unwrap() - line).abs() < 1e-12,
                "{n}"
            );
        }
        assert!(familywise_line(0, 0.05).is_err());
        assert!(familywise_line(4, 0.0).is_err());
        assert!(familywise_line(4, 1.0).is_err());
    }

    #[test]
    fn the_forecast_floor_has_the_recorded_shape() {
        // The reference's B.5.6 forecast table: a read at 2,000 over the last
        // 1,000 for the next 2,000 of a 6,000-iteration tune, mean D^2 11.6,
        // 30 games an iteration.
        let elo = ELO_PER_SCORE / 60.0;
        for (r_end, floors) in [
            (0.0031, [3.6, 5.0, 6.9, 9.0]),
            (0.00155, [1.8, 2.5, 3.5, 4.5]),
        ] {
            let knob = artifact(6_000, r_end, &[(0, 100)], 1).knobs[0]
                .knob()
                .unwrap();
            let schedule = SpsaSchedule::new(6_000).unwrap();
            let factor = forecast_factor(schedule, knob, [1_000, 2_000], 4_000, 11.6, elo)
                .unwrap()
                .unwrap();
            for (n, floor) in [8_usize, 15, 29, 49].into_iter().zip(floors) {
                let ours = 2.0 * factor * (2.0 * n as f64).sqrt();
                assert!((ours - floor).abs() < 0.05, "{r_end} {n}: {ours}");
            }
        }
        let knob = artifact(100, 0.00155, &[(0, 100)], 1).knobs[0]
            .knob()
            .unwrap();
        let schedule = SpsaSchedule::new(100).unwrap();
        assert_eq!(
            forecast_factor(schedule, knob, [0, 100], 100, 1.0, 1.0),
            Ok(None)
        );
    }

    #[test]
    fn the_gradient_counts_every_iteration() {
        // Constant perturbation 8; D = delta on 50 iterations and 0 on 50.
        // The slope is E * (50 * 8) / (2 * 100 * 64); leaving the zeros out
        // doubles it.
        let delta = (0..100)
            .map(|k| if k % 3 == 0 { -1.0 } else { 1.0 })
            .collect::<Vec<_>>();
        let scores = (0..100)
            .map(|k| if k % 2 == 0 { delta[k] as i32 } else { 0 })
            .collect::<Vec<_>>();
        let c = vec![vec![8.0]; 100];
        let ks = (0..100).collect::<Vec<_>>();
        let elo = ELO_PER_SCORE / 12.0;
        let ours = gradient(elo, &ks, 0, |k, _| delta[k], &scores, &c);
        assert!((ours - elo / 32.0).abs() < 1e-12, "{ours}");
    }

    #[test]
    fn a_tune_with_no_gradient_reads_quiet() {
        let run = simulate(&[0.0; 4], &[50.0; 4], &[(0, 100); 4], 3_000, 2_000, 57_001);
        let read = run.read(quick());
        assert_eq!(read.window, [1_000, 2_000]);
        assert!(
            !read.live,
            "max |z| {}, excess {}",
            read.max_abs_z, read.excess_in_sd
        );
        assert!(read.over_line.is_empty());
        let gain = &read.gain.since_start;
        assert!(gain.elo.abs() < 4.0 * gain.null_sd.max(0.5), "{gain:?}");
        // The forecast runs to the ceiling, and quiet bounds it.
        let forecast = read.forecast.unwrap();
        assert_eq!(forecast.span, [2_000, 3_000]);
        assert!(forecast.upper_bound);
        assert_eq!(
            read.quiet_rules_out.forecast_two_sd_elo,
            Some(2.0 * forecast.sd)
        );
    }

    #[test]
    fn a_planted_gradient_reads_live_and_over_the_line() {
        let mut starts = [50.0; 4];
        starts[0] = 30.0;
        let gradients = [0.12, 0.0, 0.0, 0.0];
        let run = simulate(&gradients, &starts, &[(0, 100); 4], 3_000, 2_000, 9_001);
        let read = run.read(quick());
        assert!(read.live);
        assert_eq!(read.over_line, ["K0"]);
        assert!(read.coordinates[0].z > read.familywise_line);
        assert!(read.gain.since_start.elo > 2.0 * read.gain.since_start.null_sd);
        assert!(read.forecast.unwrap().elo > 0.0);
        assert!(read.rails.is_empty());
        // At the ceiling there is no forecast, and no forecast floor.
        let run = simulate(&gradients, &starts, &[(0, 100); 4], 2_000, 2_000, 9_001);
        let read = run.read(quick());
        assert_eq!(read.forecast, None);
        assert_eq!(read.quiet_rules_out.forecast_two_sd_elo, None);
    }

    #[test]
    fn a_centre_at_its_bound_pushed_outward_is_a_rail() {
        let mut starts = [50.0; 4];
        starts[0] = 96.0;
        let gradients = [0.12, 0.0, 0.0, 0.0];
        let run = simulate(&gradients, &starts, &[(0, 100); 4], 3_000, 2_000, 9_001);
        let read = run.read(quick());
        assert_eq!(read.rails, ["K0"]);
        let coordinate = &read.coordinates[0];
        assert!(coordinate.clamp_share > 0.5, "{coordinate:?}");
        assert!(coordinate.outward_z.unwrap() >= read.familywise_line);
        assert!(coordinate.distance_to_bound <= coordinate.perturbation);
        assert_eq!(read.clamped_over_one_percent, ["K0"]);
        assert_eq!(read.coordinates[1].outward_z, None);
    }

    /// One coordinate whose window `z` is set exactly: `D = +-delta` on
    /// `push` iterations and cancelling in pairs on the rest, so
    /// `z = push / sqrt(1,000)`, read with the centre held at `centre` in
    /// `[0, 100]`.
    fn crafted(push: i32, centre: f64) -> SpsaRead {
        let artifact = artifact(3_000, 0.00155, &[(0, 100)], 4_242);
        let knobs = vec![artifact.knobs[0].knob().unwrap()];
        let scores = (0..2_000_u32)
            .map(|k| {
                let delta = i32::from(perturbations_for_iteration(4_242, k, 1).unwrap()[0]);
                let j = k as i32 - 1_000;
                if j < 0 {
                    0
                } else if j < push.abs() {
                    push.signum() * delta
                } else if (j - push.abs()) % 2 == 0 {
                    delta
                } else {
                    -delta
                }
            })
            .collect::<Vec<_>>();
        let centres = vec![vec![centre]; 2_001];
        let names = vec!["K0".to_owned()];
        read(
            &SpsaReadRun {
                names: &names,
                knobs: &knobs,
                schedule: artifact.schedule,
                master_seed: 4_242,
                games_per_iteration: 6,
                centres: &centres,
                scores: &scores,
            },
            SpsaReadOptions {
                null_reps: 2,
                ..SpsaReadOptions::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn a_rail_needs_an_outward_z_past_the_line() {
        let expected = 120.0 / 1_000_f64.sqrt();
        let at_max = crafted(120, 98.0);
        assert!((at_max.coordinates[0].z - expected).abs() < 1e-12);
        assert_eq!(at_max.rails, ["K0"]);
        assert_eq!(crafted(-120, 2.0).rails, ["K0"]);
        // Inward at a bound: live, but no rail.
        let inward = crafted(-120, 98.0);
        assert!(inward.live);
        assert!(inward.rails.is_empty());
        assert!(inward.coordinates[0].outward_z.unwrap() < 0.0);
        // Under the line at a bound: an outward z, no rail.
        let weak = crafted(40, 98.0);
        assert!(weak.coordinates[0].outward_z.unwrap() > 0.0);
        assert!(weak.rails.is_empty());
        // Mid-range: no bound to push against.
        let free = crafted(120, 50.0);
        assert_eq!(free.coordinates[0].outward_z, None);
        assert!(free.rails.is_empty() && free.live);
    }

    #[test]
    fn without_gradient_z_has_variance_one_and_the_gain_mean_zero() {
        let mut zs = Vec::new();
        let mut gains = Vec::new();
        for seed in 0..120 {
            let run = simulate(
                &[0.0; 4],
                &[50.0; 4],
                &[(0, 100); 4],
                400,
                400,
                1_000 + seed,
            );
            let read = run.read(SpsaReadOptions {
                null_reps: 1,
                ..SpsaReadOptions::default()
            });
            zs.extend(read.coordinates.iter().map(|coordinate| coordinate.z));
            gains.push(read.gain.since_start.elo);
        }
        let mean = zs.iter().sum::<f64>() / zs.len() as f64;
        let variance = zs.iter().map(|z| (z - mean).powi(2)).sum::<f64>() / zs.len() as f64;
        assert!(
            (variance - 1.0).abs() < 0.15,
            "z variance {variance} over {}",
            zs.len()
        );
        let gain_mean = gains.iter().sum::<f64>() / gains.len() as f64;
        let gain_sd = population_sd(&gains);
        assert!(
            gain_mean.abs() < 3.0 * gain_sd / (gains.len() as f64).sqrt(),
            "gain mean {gain_mean}, SD {gain_sd}"
        );
    }

    fn read_window(
        run: &Synthetic,
        from: Option<u32>,
        to: Option<u32>,
    ) -> Result<SpsaRead, SpsaReadError> {
        read(
            &run.run(),
            SpsaReadOptions {
                from,
                to,
                ..quick()
            },
        )
    }

    #[test]
    fn the_window_defaults_to_the_last_thousand_and_is_checked() {
        let run = simulate(&[0.0; 2], &[50.0; 2], &[(0, 100); 2], 1_000, 400, 3);
        let read_all = run.read(quick());
        assert_eq!(read_all.window, [0, 400]);
        assert_eq!(read_all.gain.window.span, read_all.gain.since_start.span);
        let narrowed = read_window(&run, Some(100), Some(300)).unwrap();
        assert_eq!(narrowed.window, [100, 300]);
        assert_eq!(narrowed.gain.since_start.span, [0, 300]);
        assert_eq!(narrowed.gain.window.span, [100, 300]);
        for (from, to) in [(Some(300), Some(300)), (None, Some(401)), (Some(500), None)] {
            assert!(matches!(
                read_window(&run, from, to),
                Err(SpsaReadError::Window { .. })
            ));
        }
        let empty = simulate(&[0.0; 2], &[50.0; 2], &[(0, 100); 2], 1_000, 0, 3);
        assert_eq!(
            read(&empty.run(), quick()),
            Err(SpsaReadError::NoIterations)
        );
    }

    #[test]
    fn the_null_is_reproducible_from_its_seed() {
        let run = simulate(&[0.05, 0.0], &[50.0; 2], &[(0, 100); 2], 600, 600, 11);
        let first = run.read(quick());
        assert_eq!(run.read(quick()).gain, first.gain);
        let other = run.read(SpsaReadOptions {
            null_seed: 2,
            ..quick()
        });
        assert_eq!(other.gain.since_start.elo, first.gain.since_start.elo);
        assert_ne!(
            other.gain.since_start.null_sd,
            first.gain.since_start.null_sd
        );
    }

    #[test]
    fn a_recorded_centre_off_the_update_is_measured_in_unit_moves() {
        let run = simulate(&[0.0; 2], &[50.0; 2], &[(0, 100); 2], 100, 10, 5);
        let schedule = run.artifact.schedule;
        assert_eq!(
            worst_move_deviation(schedule, &run.knobs, 9, &run.centres[10], &run.centres[10]),
            Ok(0.0)
        );
        let coefficients = schedule.coefficients(9, run.knobs[1]).unwrap();
        let unit = coefficients.c * coefficients.r;
        let mut off = run.centres[10].clone();
        off[1] += 0.25 * unit;
        let deviation =
            worst_move_deviation(schedule, &run.knobs, 9, &run.centres[10], &off).unwrap();
        assert!((deviation - 0.25).abs() < 1e-9, "{deviation}");
        assert!(deviation > MOVE_TOLERANCE);
    }
}
