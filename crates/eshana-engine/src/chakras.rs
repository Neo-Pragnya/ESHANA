//! Chakras: Sarvatobhadra (with vedha), Kota and Saṅghaṭṭa.

use crate::analysis::{AnalysisCtx, Derived};
use eshana_core::Result;

/// Compute this module's attributes for a chart (empty until implemented).
///
/// # Errors
/// Propagates ephemeris errors.
pub fn compute(_x: &AnalysisCtx) -> Result<Vec<Derived>> {
    Ok(vec![])
}
