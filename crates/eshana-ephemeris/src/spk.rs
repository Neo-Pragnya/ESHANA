//! A reader for JPL SPK kernels (DAF container, segment type 2: Chebyshev position polynomials), enough to
//! serve the planetary DE kernels such as `de440s.bsp` / `de440.bsp` / `de441`.
//!
//! The kernel is **user supplied**. NASA/JPL ephemerides are public-domain data; ESHANA does not bundle them,
//! so no kernel licence is carried into the package. Only little-endian IEEE kernels are accepted.
//!
//! Time: ET is taken as TT plus the dominant periodic TDB−TT term (about 1.7 ms), documented as an assumption.

use crate::frames::{apparent_of_date, norm, AU_KM, C_KM_S};
use crate::{EphemerisProvider, ProviderInfo};
use eshana_core::{Body, EshanaError, Result};
use eshana_time::JD_J2000;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Segment {
    start_et: f64,
    end_et: f64,
    start_addr: usize,
    end_addr: usize,
}

/// An SPK kernel loaded into memory.
#[derive(Debug)]
pub struct SpkProvider {
    data: Vec<u8>,
    /// Keyed by (target, center); a pair can have several segments covering different time spans.
    segs: HashMap<(i32, i32), Vec<Segment>>,
    identity: String,
    jd_min: f64,
    jd_max: f64,
}

fn rd_f64(d: &[u8], off: usize) -> Result<f64> {
    d.get(off..off + 8)
        .map(|b| f64::from_le_bytes(b.try_into().expect("8 bytes")))
        .ok_or_else(|| EshanaError::Provider("kernel truncated".into()))
}

fn rd_i32(d: &[u8], off: usize) -> Result<i32> {
    d.get(off..off + 4)
        .map(|b| i32::from_le_bytes(b.try_into().expect("4 bytes")))
        .ok_or_else(|| EshanaError::Provider("kernel truncated".into()))
}

fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

impl SpkProvider {
    /// Read and index a kernel file.
    ///
    /// # Errors
    /// [`EshanaError::Provider`] when the file is missing or not a little-endian SPK.
    pub fn open(path: &str) -> Result<Self> {
        let data =
            std::fs::read(path).map_err(|e| EshanaError::Provider(format!("cannot read '{path}': {e}")))?;
        Self::from_bytes(data, path)
    }

    /// Index a kernel already in memory (used by tests).
    ///
    /// # Errors
    /// [`EshanaError::Provider`] when the bytes are not a little-endian SPK.
    pub fn from_bytes(data: Vec<u8>, name: &str) -> Result<Self> {
        if data.len() < 1024 || !data.starts_with(b"DAF/SPK") {
            return Err(EshanaError::Provider(format!("'{name}' is not a DAF/SPK file")));
        }
        if !data[88..96].starts_with(b"LTL-IEEE") {
            return Err(EshanaError::Provider(
                "only little-endian SPK kernels are supported".into(),
            ));
        }
        let (nd, ni) = (rd_i32(&data, 8)? as usize, rd_i32(&data, 12)? as usize);
        if nd != 2 || ni != 6 {
            return Err(EshanaError::Provider(format!(
                "unexpected SPK summary shape ND={nd} NI={ni}"
            )));
        }
        let internal = String::from_utf8_lossy(&data[16..76]).trim().to_string();
        let mut rec = rd_i32(&data, 76)? as usize; // forward pointer: first summary record (1-based)
        let sum_doubles = nd + ni.div_ceil(2);
        let mut segs: HashMap<(i32, i32), Vec<Segment>> = HashMap::new();
        let (mut min_et, mut max_et) = (f64::INFINITY, f64::NEG_INFINITY);
        while rec != 0 {
            let base = (rec - 1) * 1024;
            let next = rd_f64(&data, base)? as usize;
            let n = rd_f64(&data, base + 16)? as usize;
            for k in 0..n {
                let off = base + 24 + k * sum_doubles * 8;
                let (start_et, end_et) = (rd_f64(&data, off)?, rd_f64(&data, off + 8)?);
                let ints = off + 16;
                let (target, center, _frame, ty) = (
                    rd_i32(&data, ints)?,
                    rd_i32(&data, ints + 4)?,
                    rd_i32(&data, ints + 8)?,
                    rd_i32(&data, ints + 12)?,
                );
                let (sa, ea) = (
                    rd_i32(&data, ints + 16)? as usize,
                    rd_i32(&data, ints + 20)? as usize,
                );
                if ty != 2 {
                    continue; // other segment types are ignored; planetary DE kernels use type 2
                }
                min_et = min_et.min(start_et);
                max_et = max_et.max(end_et);
                segs.entry((target, center)).or_default().push(Segment {
                    start_et,
                    end_et,
                    start_addr: sa,
                    end_addr: ea,
                });
            }
            rec = next;
        }
        if segs.is_empty() {
            return Err(EshanaError::Provider("no type-2 segments found in kernel".into()));
        }
        let identity = format!(
            "{} | {} | {} bytes | fnv1a64 {:016x}",
            name_of(name),
            internal,
            data.len(),
            fnv1a64(&data)
        );
        Ok(Self {
            data,
            segs,
            identity,
            jd_min: JD_J2000 + min_et / 86_400.0,
            jd_max: JD_J2000 + max_et / 86_400.0,
        })
    }

    /// Position (km) and velocity (km/s) of `target` relative to `center` at ET seconds past J2000.
    fn segment_state(&self, target: i32, center: i32, et: f64) -> Result<([f64; 3], [f64; 3])> {
        let list = self.segs.get(&(target, center)).ok_or_else(|| {
            EshanaError::Provider(format!("kernel has no segment {target} relative to {center}"))
        })?;
        let seg = list
            .iter()
            .find(|s| et >= s.start_et && et <= s.end_et)
            .ok_or_else(|| {
                EshanaError::OutOfRange(format!("ET {et:.0} s is outside the kernel segment for {target}"))
            })?;
        // Directory at the end of the segment: INIT, INTLEN, RSIZE, N.
        let a = |addr: usize| (addr - 1) * 8;
        let end = seg.end_addr;
        let init = rd_f64(&self.data, a(end - 3))?;
        let intlen = rd_f64(&self.data, a(end - 2))?;
        let rsize = rd_f64(&self.data, a(end - 1))? as usize;
        let n = rd_f64(&self.data, a(end))? as usize;
        let idx = (((et - init) / intlen).floor() as usize).min(n - 1);
        let rec0 = seg.start_addr + idx * rsize;
        let mid = rd_f64(&self.data, a(rec0))?;
        let radius = rd_f64(&self.data, a(rec0 + 1))?;
        let ncoef = (rsize - 2) / 3;
        let s = (et - mid) / radius;
        // Chebyshev values and derivatives.
        let mut tk = vec![0.0; ncoef];
        let mut dk = vec![0.0; ncoef];
        tk[0] = 1.0;
        if ncoef > 1 {
            tk[1] = s;
            dk[1] = 1.0;
        }
        for k in 2..ncoef {
            tk[k] = 2.0 * s * tk[k - 1] - tk[k - 2];
            dk[k] = 2.0 * tk[k - 1] + 2.0 * s * dk[k - 1] - dk[k - 2];
        }
        let (mut p, mut v) = ([0.0; 3], [0.0; 3]);
        for c in 0..3 {
            for k in 0..ncoef {
                let coef = rd_f64(&self.data, a(rec0 + 2 + c * ncoef + k))?;
                p[c] += coef * tk[k];
                v[c] += coef * dk[k];
            }
            v[c] /= radius;
        }
        Ok((p, v))
    }

    /// Barycentric state (km, km/s) of a body id: 10 Sun, 301 Moon, 399 Earth, 1-9 planet barycentres.
    fn bary(&self, id: i32, et: f64) -> Result<([f64; 3], [f64; 3])> {
        let add = |a: ([f64; 3], [f64; 3]), b: ([f64; 3], [f64; 3])| {
            (
                [a.0[0] + b.0[0], a.0[1] + b.0[1], a.0[2] + b.0[2]],
                [a.1[0] + b.1[0], a.1[1] + b.1[1], a.1[2] + b.1[2]],
            )
        };
        match id {
            301 | 399 => Ok(add(self.segment_state(id, 3, et)?, self.segment_state(3, 0, et)?)),
            other => self.segment_state(other, 0, et),
        }
    }
}

fn name_of(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map_or_else(|| path.to_string(), |s| s.to_string_lossy().into_owned())
}

fn naif_id(body: Body) -> Option<i32> {
    Some(match body {
        Body::Sun => 10,
        Body::Moon => 301,
        Body::Mercury => 1,
        Body::Venus => 2,
        Body::Mars => 4,
        Body::Jupiter => 5,
        Body::Saturn => 6,
        Body::Uranus => 7,
        Body::Neptune => 8,
        Body::Pluto => 9,
        Body::MeanNode | Body::TrueNode => return None,
    })
}

/// ICRS equatorial to mean ecliptic of J2000 (rotation by the IAU 2006 obliquity 84381.406″).
fn eq_to_ecl(v: [f64; 3]) -> [f64; 3] {
    let e = (84_381.406_f64 / 3600.0).to_radians();
    [
        v[0],
        v[1] * e.cos() + v[2] * e.sin(),
        -v[1] * e.sin() + v[2] * e.cos(),
    ]
}

impl EphemerisProvider for SpkProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "spk".into(),
            version: "0.1.0".into(),
            data: self.identity.clone(),
            accuracy: "Kernel-limited. Agreement with an independent apparent-place pipeline is recorded in \
                       benchmarks/golden/accuracy.md (arc-second level; nutation uses a 4-term series, no light deflection)."
                .into(),
            jd_min: self.jd_min,
            jd_max: self.jd_max,
        }
    }

    fn supports(&self, body: Body) -> bool {
        body.is_node() || naif_id(body).is_some()
    }

    fn apparent(&self, jd_tt: f64, body: Body) -> Result<(f64, f64, Option<f64>)> {
        self.check_range(jd_tt)?;
        if body == Body::MeanNode {
            // The mean node is a mean element, not a state; the analytic series defines it.
            return Ok((crate::analytic::mean_node_apparent(jd_tt), 0.0, None));
        }
        if body == Body::TrueNode {
            // Osculating node from the kernel's own Moon.
            let n = crate::analytic::osculating_node(jd_tt, &|jd| {
                let (lon, lat, d) = self.apparent(jd, Body::Moon)?;
                Ok((lon, lat, d.unwrap_or(0.0)))
            })?;
            return Ok((n, 0.0, None));
        }
        let id = naif_id(body).ok_or_else(|| EshanaError::BodyUnsupported(body.id().into()))?;
        let g = (357.53 + 0.985_600_3 * (jd_tt - JD_J2000)).to_radians();
        let et = (jd_tt - JD_J2000) * 86_400.0 + 0.001_657 * g.sin() + 0.000_014 * (2.0 * g).sin();
        let (e_pos, e_vel) = self.bary(399, et)?;
        let mut tau = 0.0;
        let mut g_vec = [0.0; 3];
        for _ in 0..4 {
            let (tp, _) = self.bary(id, et - tau)?;
            g_vec = [tp[0] - e_pos[0], tp[1] - e_pos[1], tp[2] - e_pos[2]];
            tau = norm(g_vec) / C_KM_S;
        }
        let ecl = eq_to_ecl(g_vec);
        let v_au_day = eq_to_ecl([
            e_vel[0] * 86_400.0 / AU_KM,
            e_vel[1] * 86_400.0 / AU_KM,
            e_vel[2] * 86_400.0 / AU_KM,
        ]);
        let t = (jd_tt - JD_J2000) / 36_525.0;
        let (lon, lat) = apparent_of_date(ecl, v_au_day, t);
        Ok((lon, lat, Some(norm(g_vec) / AU_KM)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_spk_bytes() {
        let e = SpkProvider::from_bytes(vec![0; 4096], "junk").unwrap_err();
        assert_eq!(e.code(), "ESH-PROVIDER");
    }

    #[test]
    fn rejects_short_files() {
        assert!(SpkProvider::from_bytes(vec![1, 2, 3], "tiny").is_err());
    }

    #[test]
    fn missing_file_is_a_provider_error() {
        let e = SpkProvider::open("/definitely/not/here.bsp").unwrap_err();
        assert_eq!(e.code(), "ESH-PROVIDER");
    }

    #[test]
    fn equatorial_to_ecliptic_keeps_length() {
        let v = eq_to_ecl([0.3, -0.5, 0.8]);
        assert!((norm(v) - norm([0.3, -0.5, 0.8])).abs() < 1e-12);
    }
}
