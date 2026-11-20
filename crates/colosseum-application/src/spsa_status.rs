//! Runtime-neutral, explicitly heuristic SPSA trajectory diagnostics.

use colosseum_core::SpsaScheduleArtifact;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{SpsaBoundTune, SpsaCenterSample, SpsaRunSettings, SpsaRunSettingsError};

pub const SPSA_STATUS_SCHEMA_VERSION: u32 = 2;
pub const SPSA_DIAGNOSTIC_MIN_HISTORY: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpsaTrajectoryPoint {
    /// Zero-based iteration whose completed update produced this centre.
    pub iteration: u32,
    pub value: f64,
    pub normalized_to_range: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpsaThirdSummary {
    pub first_iteration: u32,
    pub last_iteration: u32,
    pub samples: u32,
    pub mean: f64,
    pub mean_normalized_to_range: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpsaThirdsComparison {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thirds: Option<[SpsaThirdSummary; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpsaKnobDiagnostics {
    pub name: String,
    pub initial: i64,
    pub min: i64,
    pub max: i64,
    pub current: f64,
    pub current_normalized_to_range: f64,
    pub current_perturbation: f64,
    pub trajectory: Vec<SpsaTrajectoryPoint>,
    pub thirds: SpsaThirdsComparison,
    /// The share of completed centres exactly on either bound; absent
    /// before the first iteration.
    pub bound_contact_share: Option<f64>,
    /// How far the centre has moved from its initial value, as a share of
    /// the range.
    pub net_movement_share_of_range: f64,
    /// The span of the latest third of the centres, as a share of the range;
    /// absent before [`SPSA_DIAGNOSTIC_MIN_HISTORY`] iterations.
    pub latest_third_span_share_of_range: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpsaEta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_seconds: Option<f64>,
    pub completed_iterations_basis: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    pub interpretation: String,
    /// The time to the iteration this invocation was asked to stop at, when
    /// that stop falls before the end of the tune. Absent without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_point: Option<SpsaStopPointEta>,
}

/// The time left to a requested stop before the horizon, projected from the
/// same observed rate as the time left to the end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpsaStopPointEta {
    /// The committed-iteration count the tune stops at.
    pub iteration: u32,
    /// Zero once the stop point is reached; absent while the rate is unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_seconds: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpsaStatusReport {
    pub schema_version: u32,
    pub settings: SpsaRunSettings,
    pub completed_iterations: u32,
    pub percent_complete: f64,
    pub invalid: bool,
    pub eta: SpsaEta,
    pub knobs: Vec<SpsaKnobDiagnostics>,
    pub interpretation: String,
}

pub fn diagnose_spsa(
    tune: &SpsaBoundTune,
    schedule: &SpsaScheduleArtifact,
    settings: SpsaRunSettings,
    history: &[SpsaCenterSample],
    invalid: bool,
    elapsed_to_last_checkpoint_seconds: Option<f64>,
    stop_after_iteration: Option<u32>,
) -> Result<SpsaStatusReport, SpsaStatusError> {
    settings.validate()?;
    schedule.validate()?;
    if schedule.schedule.iterations() != settings.iterations {
        return Err(SpsaStatusError::ScheduleHorizonMismatch {
            schedule: schedule.schedule.iterations(),
            settings: settings.iterations,
        });
    }
    if tune.parameters.is_empty() || tune.parameters.len() != schedule.knobs.len() {
        return Err(SpsaStatusError::DimensionMismatch);
    }
    if history.len() > settings.iterations as usize {
        return Err(SpsaStatusError::HistoryBeyondHorizon);
    }
    for (index, sample) in history.iter().enumerate() {
        if sample.iteration != index as u32 || sample.centers.len() != tune.parameters.len() {
            return Err(SpsaStatusError::HistoryMismatch { index });
        }
    }
    let completed_iterations = u32::try_from(history.len()).expect("history is bounded by u32");
    let percent_complete = f64::from(completed_iterations) * 100.0 / f64::from(settings.iterations);
    let mut eta = eta(
        settings,
        completed_iterations,
        invalid,
        elapsed_to_last_checkpoint_seconds,
    )?;
    eta.stop_point = stop_point(
        settings,
        completed_iterations,
        invalid,
        elapsed_to_last_checkpoint_seconds,
        stop_after_iteration,
    );
    let coefficient_iteration = completed_iterations.min(settings.iterations - 1);
    let knobs = tune
        .parameters
        .iter()
        .enumerate()
        .map(|(knob_index, bound)| {
            let parameter = &bound.parameter;
            let range = (parameter.max - parameter.min) as f64;
            let values = history
                .iter()
                .map(|sample| sample.centers[knob_index])
                .collect::<Vec<_>>();
            for value in &values {
                if !value.is_finite()
                    || *value < parameter.min as f64
                    || *value > parameter.max as f64
                {
                    return Err(SpsaStatusError::CenterOutsideRange {
                        name: parameter.name.clone(),
                        value: *value,
                    });
                }
            }
            let current = values.last().copied().unwrap_or(parameter.initial as f64);
            let trajectory = history
                .iter()
                .zip(&values)
                .map(|(sample, value)| SpsaTrajectoryPoint {
                    iteration: sample.iteration,
                    value: *value,
                    normalized_to_range: normalize(*value, parameter.min, range),
                })
                .collect::<Vec<_>>();
            let current_perturbation = schedule
                .schedule
                .coefficients(coefficient_iteration, schedule.knobs[knob_index].knob()?)?
                .c;
            let contacts = values
                .iter()
                .filter(|value| **value == parameter.min as f64 || **value == parameter.max as f64)
                .count();
            let bound_contact_share =
                (!values.is_empty()).then(|| contacts as f64 / values.len() as f64);
            let net_movement_share_of_range = (current - parameter.initial as f64).abs() / range;
            let latest = &values[values.len() * 2 / 3..];
            let latest_third_span_share_of_range = (values.len() >= SPSA_DIAGNOSTIC_MIN_HISTORY)
                .then(|| {
                    let low = latest.iter().copied().fold(f64::INFINITY, f64::min);
                    let high = latest.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    (high - low) / range
                });
            Ok(SpsaKnobDiagnostics {
                name: parameter.name.clone(),
                initial: parameter.initial,
                min: parameter.min,
                max: parameter.max,
                current,
                current_normalized_to_range: normalize(current, parameter.min, range),
                current_perturbation,
                trajectory,
                thirds: thirds(history, &values, parameter.min, range),
                bound_contact_share,
                net_movement_share_of_range,
                latest_third_span_share_of_range,
            })
        })
        .collect::<Result<Vec<_>, SpsaStatusError>>()?;
    Ok(SpsaStatusReport {
        schema_version: SPSA_STATUS_SCHEMA_VERSION,
        settings,
        completed_iterations,
        percent_complete,
        invalid,
        eta,
        knobs,
        interpretation: "the figures describe the trajectory; they are not evidence of causation or convergence and never advice to continue or abandon a tune".into(),
    })
}

fn eta(
    settings: SpsaRunSettings,
    completed: u32,
    invalid: bool,
    elapsed: Option<f64>,
) -> Result<SpsaEta, SpsaStatusError> {
    if invalid {
        return Ok(SpsaEta {
            remaining_seconds: None,
            completed_iterations_basis: completed,
            unavailable_reason: Some("the tune is invalid and will not advance".into()),
            interpretation: "unavailable".into(),
            stop_point: None,
        });
    }
    if completed == settings.iterations {
        return Ok(SpsaEta {
            remaining_seconds: Some(0.0),
            completed_iterations_basis: completed,
            unavailable_reason: None,
            interpretation: "horizon complete".into(),
            stop_point: None,
        });
    }
    let Some(elapsed) = elapsed else {
        return Ok(SpsaEta {
            remaining_seconds: None,
            completed_iterations_basis: completed,
            unavailable_reason: Some(
                "no uncontaminated elapsed-to-checkpoint timing is available".into(),
            ),
            interpretation: "unavailable".into(),
            stop_point: None,
        });
    };
    if !elapsed.is_finite() || elapsed < 0.0 {
        return Err(SpsaStatusError::InvalidElapsed { value: elapsed });
    }
    if completed == 0 || elapsed == 0.0 {
        return Ok(SpsaEta {
            remaining_seconds: None,
            completed_iterations_basis: completed,
            unavailable_reason: Some("no completed timed iteration is available".into()),
            interpretation: "unavailable".into(),
            stop_point: None,
        });
    }
    Ok(SpsaEta {
        remaining_seconds: Some(
            elapsed / f64::from(completed) * f64::from(settings.iterations - completed),
        ),
        completed_iterations_basis: completed,
        unavailable_reason: None,
        interpretation: "linear projection from durable elapsed time through the last committed checkpoint; setup and host-load changes may make it inaccurate".into(),
        stop_point: None,
    })
}

/// The time to a stop requested before the horizon, at the rate the time to
/// the end uses. A stop at or past the horizon is the end, so it adds nothing.
fn stop_point(
    settings: SpsaRunSettings,
    completed: u32,
    invalid: bool,
    elapsed: Option<f64>,
    stop_after_iteration: Option<u32>,
) -> Option<SpsaStopPointEta> {
    let iteration = stop_after_iteration.filter(|stop| *stop < settings.iterations)?;
    if invalid {
        return None;
    }
    let remaining_seconds = if completed >= iteration {
        Some(0.0)
    } else {
        elapsed
            .filter(|elapsed| elapsed.is_finite() && *elapsed > 0.0 && completed > 0)
            .map(|elapsed| elapsed / f64::from(completed) * f64::from(iteration - completed))
    };
    Some(SpsaStopPointEta {
        iteration,
        remaining_seconds,
    })
}

fn thirds(
    history: &[SpsaCenterSample],
    values: &[f64],
    min: i64,
    range: f64,
) -> SpsaThirdsComparison {
    if values.len() < SPSA_DIAGNOSTIC_MIN_HISTORY {
        return SpsaThirdsComparison {
            thirds: None,
            unavailable_reason: Some(format!(
                "need at least {SPSA_DIAGNOSTIC_MIN_HISTORY} completed iterations; have {}",
                values.len()
            )),
        };
    }
    let summaries = std::array::from_fn(|third| {
        let start = third * values.len() / 3;
        let end = (third + 1) * values.len() / 3;
        let slice = &values[start..end];
        let mean = slice.iter().sum::<f64>() / slice.len() as f64;
        SpsaThirdSummary {
            first_iteration: history[start].iteration,
            last_iteration: history[end - 1].iteration,
            samples: slice.len() as u32,
            mean,
            mean_normalized_to_range: normalize(mean, min, range),
        }
    });
    SpsaThirdsComparison {
        thirds: Some(summaries),
        unavailable_reason: None,
    }
}

fn normalize(value: f64, min: i64, range: f64) -> f64 {
    (value - min as f64) / range
}

#[derive(Debug, Error)]
pub enum SpsaStatusError {
    #[error(transparent)]
    Settings(#[from] SpsaRunSettingsError),
    #[error(transparent)]
    Schedule(#[from] colosseum_core::SpsaError),
    #[error("SPSA schedule horizon {schedule} does not match run setting {settings}")]
    ScheduleHorizonMismatch { schedule: u32, settings: u32 },
    #[error("SPSA tune/schedule dimensions do not match")]
    DimensionMismatch,
    #[error("SPSA history extends beyond the configured horizon")]
    HistoryBeyondHorizon,
    #[error("SPSA history is non-contiguous or has the wrong dimension at index {index}")]
    HistoryMismatch { index: usize },
    #[error("SPSA centre {name}={value} is outside its requested range")]
    CenterOutsideRange { name: String, value: f64 },
    #[error("elapsed-to-checkpoint timing must be finite and nonnegative; got {value}")]
    InvalidElapsed { value: f64 },
}

#[cfg(test)]
mod tests {
    use colosseum_core::{SpsaEndSpec, SpsaScheduleArtifact};

    use super::*;
    use crate::{SpsaBoundParameter, SpsaLiveSpin, SpsaTuneParameter};

    fn bound(c_end: f64) -> SpsaBoundTune {
        SpsaBoundTune {
            parameters: vec![SpsaBoundParameter {
                parameter: SpsaTuneParameter {
                    name: "Reduction".into(),
                    initial: 50,
                    min: 0,
                    max: 100,
                    c_end,
                },
                advertised: SpsaLiveSpin {
                    name: "Reduction".into(),
                    default: 50,
                    min: 0,
                    max: 100,
                },
            }],
        }
    }

    fn schedule(iterations: u32, c_end: f64) -> SpsaScheduleArtifact {
        SpsaScheduleArtifact::derive(
            iterations,
            0.002,
            7,
            &[SpsaEndSpec {
                name: "Reduction".into(),
                min: 0,
                max: 100,
                c_end,
            }],
        )
        .unwrap()
    }

    fn history(values: &[f64]) -> Vec<SpsaCenterSample> {
        values
            .iter()
            .enumerate()
            .map(|(iteration, value)| SpsaCenterSample {
                iteration: iteration as u32,
                centers: vec![*value],
            })
            .collect()
    }

    #[test]
    fn diagnostics_match_hand_computed_thirds_and_eta() {
        let values = [0.0, 50.0, 50.0, 51.0, 51.1, 51.05];
        let report = diagnose_spsa(
            &bound(1.0),
            &schedule(9, 1.0),
            SpsaRunSettings::new(9, 2).unwrap(),
            &history(&values),
            false,
            Some(60.0),
            None,
        )
        .unwrap();
        assert_eq!(report.completed_iterations, 6);
        assert!((report.percent_complete - 66.666_666_666_666_67).abs() < 1e-12);
        assert_eq!(report.eta.remaining_seconds, Some(30.0));
        let knob = &report.knobs[0];
        let thirds = knob.thirds.thirds.unwrap();
        assert_eq!(thirds[0].mean, 25.0);
        assert_eq!(thirds[1].mean, 50.5);
        assert_eq!(thirds[2].mean, 51.075);
        // One centre of six on the minimum; 1.05 moved from 50 in 100; the
        // latest third, 51.1 and 51.05, spans 0.05.
        assert_eq!(knob.bound_contact_share, Some(1.0 / 6.0));
        assert!((knob.net_movement_share_of_range - 0.0105).abs() < 1e-12);
        let span = knob.latest_third_span_share_of_range.unwrap();
        assert!((span - 0.0005).abs() < 1e-12, "{span}");
    }

    #[test]
    fn short_history_is_explicitly_insufficient() {
        let report = diagnose_spsa(
            &bound(1.0),
            &schedule(10, 1.0),
            SpsaRunSettings::new(10, 2).unwrap(),
            &history(&[50.0, 50.1]),
            false,
            None,
            None,
        )
        .unwrap();
        let knob = &report.knobs[0];
        assert!(knob.thirds.thirds.is_none());
        assert_eq!(knob.latest_third_span_share_of_range, None);
        assert_eq!(knob.bound_contact_share, Some(0.0));
        assert!(report.eta.remaining_seconds.is_none());
    }

    #[test]
    fn a_small_perturbation_is_reported_without_inventing_advice() {
        let report = diagnose_spsa(
            &bound(0.4),
            &schedule(6, 0.4),
            SpsaRunSettings::new(6, 2).unwrap(),
            &history(&[50.0; 6]),
            false,
            Some(6.0),
            None,
        )
        .unwrap();
        assert!(report.knobs[0].current_perturbation < 0.5);
        assert!(report.interpretation.contains("never advice"));
    }

    #[test]
    fn a_stop_before_the_horizon_has_its_own_time_from_the_same_rate() {
        let status = |completed: usize, stop: Option<u32>| {
            diagnose_spsa(
                &bound(1.0),
                &schedule(9, 1.0),
                SpsaRunSettings::new(9, 2).unwrap(),
                &history(&vec![50.0; completed]),
                false,
                Some(10.0 * completed as f64),
                stop,
            )
            .unwrap()
            .eta
        };
        // Ten seconds an iteration: four to the stop at 6, seven to the end.
        let eta = status(2, Some(6));
        assert_eq!(eta.remaining_seconds, Some(70.0));
        assert_eq!(
            eta.stop_point,
            Some(SpsaStopPointEta {
                iteration: 6,
                remaining_seconds: Some(40.0)
            })
        );
        // A reached stop has no time left; the end still has its own.
        let eta = status(6, Some(6));
        assert_eq!(eta.stop_point.unwrap().remaining_seconds, Some(0.0));
        assert_eq!(eta.remaining_seconds, Some(30.0));
        // No stop, or a stop that is the end, adds nothing.
        assert_eq!(status(2, None).stop_point, None);
        assert_eq!(status(2, Some(9)).stop_point, None);
        assert_eq!(status(2, Some(12)).stop_point, None);
    }
}
