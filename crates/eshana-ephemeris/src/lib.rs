//! ESHANA ephemeris: the [`EphemerisProvider`] trait, a dependency-free analytic provider and a reader for
//! JPL SPK kernels that the caller supplies.
//!
//! Every provider returns the **same frame**: geocentric, apparent (light-time and annual aberration
//! corrected), ecliptic longitude and latitude of the true equinox of date. Differences between providers are
//! therefore differences of accuracy, never of definition. Providers state their measured accuracy.

pub mod analytic;
pub mod frames;
pub mod kernels;
pub mod spk;
#[rustfmt::skip]
mod vsop87;

use eshana_core::{Body, EshanaError, Result};
use serde::Serialize;

/// One apparent geocentric position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BodyPosition {
    /// Which body.
    pub body: Body,
    /// Ecliptic longitude, degrees, tropical, true equinox of date, `[0, 360)`.
    pub longitude_deg: f64,
    /// Ecliptic latitude, degrees. `0` for the lunar nodes, which lie on the ecliptic.
    pub latitude_deg: f64,
    /// Distance in astronomical units (Earth–Moon for the Moon). `None` for the nodes.
    pub distance_au: Option<f64>,
    /// Longitude speed in degrees per day (negative when retrograde).
    pub speed_deg_per_day: f64,
}

impl BodyPosition {
    /// True when the longitude is decreasing.
    #[must_use]
    pub fn is_retrograde(&self) -> bool {
        self.speed_deg_per_day < 0.0
    }
}

/// What a provider says about itself. Copied into every result's provenance.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProviderInfo {
    /// Stable id (`analytic`, `spk`).
    pub id: String,
    /// Implementation version.
    pub version: String,
    /// What data the provider used.
    pub data: String,
    /// Measured accuracy statement.
    pub accuracy: String,
    /// First supported Julian day (TT).
    pub jd_min: f64,
    /// Last supported Julian day (TT).
    pub jd_max: f64,
}

/// A source of apparent geocentric positions.
pub trait EphemerisProvider: Send + Sync {
    /// Describe the provider.
    fn info(&self) -> ProviderInfo;

    /// Whether this provider can supply the body.
    fn supports(&self, body: Body) -> bool;

    /// Apparent longitude, latitude and distance at a TT Julian day, without speed.
    ///
    /// # Errors
    /// Out of range, unsupported body or data failure.
    fn apparent(&self, jd_tt: f64, body: Body) -> Result<(f64, f64, Option<f64>)>;

    /// Full position, with the longitude speed from a symmetric difference over ±0.01 day.
    ///
    /// # Errors
    /// As [`EphemerisProvider::apparent`].
    fn position(&self, jd_tt: f64, body: Body) -> Result<BodyPosition> {
        let (lon, lat, dist) = self.apparent(jd_tt, body)?;
        let h = 0.01;
        let (lo1, _, _) = self.apparent(jd_tt - h, body)?;
        let (lo2, _, _) = self.apparent(jd_tt + h, body)?;
        Ok(BodyPosition {
            body,
            longitude_deg: lon,
            latitude_deg: lat,
            distance_au: dist,
            speed_deg_per_day: eshana_core::angular_diff(lo2, lo1) / (2.0 * h),
        })
    }

    /// Check an instant is inside the provider's range.
    ///
    /// # Errors
    /// [`EshanaError::OutOfRange`].
    fn check_range(&self, jd_tt: f64) -> Result<()> {
        let i = self.info();
        if jd_tt < i.jd_min || jd_tt > i.jd_max {
            return Err(EshanaError::OutOfRange(format!(
                "JD {jd_tt:.2} is outside provider '{}' range [{:.1}, {:.1}]",
                i.id, i.jd_min, i.jd_max
            )));
        }
        Ok(())
    }
}

/// Options for building a provider by id.
#[derive(Debug, Clone, Default)]
pub struct ProviderOptions {
    /// Path to a JPL SPK kernel (for example `de440s.bsp`). Falls back to the `ESHANA_SPK` variable.
    pub spk_path: Option<String>,
}

/// Provider ids this build knows.
pub const PROVIDER_IDS: [&str; 2] = ["analytic", "spk"];

/// Build a provider by id.
///
/// # Errors
/// [`EshanaError::Config`] for an unknown id; [`EshanaError::Provider`] when a kernel cannot be read.
pub fn provider_by_id(id: &str, opts: &ProviderOptions) -> Result<Box<dyn EphemerisProvider>> {
    match id {
        "analytic" => Ok(Box::new(analytic::AnalyticProvider::new())),
        "spk" => {
            let path = opts
                .spk_path
                .clone()
                .or_else(|| std::env::var("ESHANA_SPK").ok())
                .ok_or_else(|| {
                    EshanaError::Config(
                        "provider 'spk' needs a kernel: pass --spk PATH or set ESHANA_SPK (for example de440s.bsp \
                         from https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/)"
                            .into(),
                    )
                })?;
            // a catalogue name (de440) is looked up in the kernel folder
            // several kernels, comma-separated (a planetary one first); catalogue names are looked up in the folder
            let paths = path
                .split(',')
                .map(|p| kernels::locate(p.trim()))
                .collect::<Result<Vec<_>>>()?;
            Ok(Box::new(spk::SpkProvider::open_many(&paths)?))
        }
        other => Err(EshanaError::Config(format!(
            "unknown provider '{other}'; known: {}",
            PROVIDER_IDS.join(", ")
        ))),
    }
}
