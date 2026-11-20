//! Runtime-neutral design contract for capped sequential experiments.

use colosseum_core::{EloModel, StatisticsError, sprt_wald_bounds};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SprtParameters {
    pub model: EloModel,
    pub elo0: f64,
    pub elo1: f64,
    pub alpha: f64,
    pub beta: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SprtDesign {
    pub parameters: SprtParameters,
    pub max_pairs: u32,
    pub lower_bound: f64,
    pub upper_bound: f64,
    /// Always `null` now. Earlier versions named a preset here, and the
    /// design is part of a run's resolved configuration, so the field stays:
    /// a design without a preset hashes as it always has, and its
    /// interrupted run still resumes.
    #[serde(default, rename = "bundle")]
    pub retired_preset: Option<RetiredPreset>,
}

/// The presets earlier versions offered, read back from their run records
/// and never set again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetiredPreset {
    Gainer,
    Simplify,
}

impl SprtDesign {
    pub fn new(parameters: SprtParameters, max_pairs: u32) -> Result<Self, SprtDesignError> {
        if max_pairs == 0 {
            return Err(SprtDesignError::ZeroPairCap);
        }
        let (lower_bound, upper_bound) = sprt_wald_bounds(
            parameters.elo0,
            parameters.elo1,
            parameters.alpha,
            parameters.beta,
        )?;
        Ok(Self {
            parameters,
            max_pairs,
            lower_bound,
            upper_bound,
            retired_preset: None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum SprtDesignError {
    #[error("SPRT max-pairs must be at least one")]
    ZeroPairCap,
    #[error(transparent)]
    Statistics(#[from] StatisticsError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn design() -> SprtParameters {
        SprtParameters {
            model: EloModel::Normalized,
            elo0: 0.0,
            elo1: 5.0,
            alpha: 0.05,
            beta: 0.05,
        }
    }

    #[test]
    fn design_requires_a_finite_cap_and_valid_explicit_statistics() {
        assert!(matches!(
            SprtDesign::new(design(), 0),
            Err(SprtDesignError::ZeroPairCap)
        ));
        let mut invalid = design();
        invalid.elo1 = invalid.elo0;
        assert!(matches!(
            SprtDesign::new(invalid, 1),
            Err(SprtDesignError::Statistics(
                StatisticsError::InvalidHypotheses { .. }
            ))
        ));
    }
}
