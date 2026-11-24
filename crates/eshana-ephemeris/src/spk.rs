//! A reader for JPL SPK kernels (DAF container; segment type 2, Chebyshev position polynomials, and type 3,
//! Chebyshev position and velocity polynomials), enough to serve the planetary DE kernels such as `de440s.bsp`,
//! `de440.bsp`, `de442.bsp` and the two `de441` parts.
//!
//! Kernels up to 256 MB are read into memory; larger ones (each `de441` part is about 1.6 GB) are read on demand,
//! one Chebyshev record at a time, so a 13,000-year kernel costs a file handle, not 1.6 GB of memory.
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
    /// SPK data type: 2 (position coefficients) or 3 (position and velocity coefficients).
    ty: i32,
    /// Which loaded kernel holds it.
    kernel: usize,
}

/// Kernels larger than this are read on demand instead of into memory.
const IN_MEMORY_MAX: u64 = 256 * 1024 * 1024;

/// The kernel bytes: in memory, or a file read on demand.
#[derive(Debug)]
enum Store {
    Mem(Vec<u8>),
    File {
        file: std::sync::Mutex<std::fs::File>,
        len: usize,
    },
}

impl Store {
    fn len(&self) -> usize {
        match self {
            Store::Mem(d) => d.len(),
            Store::File { len, .. } => *len,
        }
    }

    /// `n` bytes at `off`.
    fn bytes(&self, off: usize, n: usize) -> Result<Vec<u8>> {
        let short = || EshanaError::Provider("kernel truncated".into());
        match self {
            Store::Mem(d) => d.get(off..off + n).map(<[u8]>::to_vec).ok_or_else(short),
            Store::File { file, len } => {
                use std::io::{Read, Seek, SeekFrom};
                if off + n > *len {
                    return Err(short());
                }
                let mut f = file
                    .lock()
                    .map_err(|_| EshanaError::Provider("kernel file lock poisoned".into()))?;
                let mut buf = vec![0u8; n];
                f.seek(SeekFrom::Start(off as u64))
                    .and_then(|_| f.read_exact(&mut buf))
                    .map_err(|e| EshanaError::Provider(format!("reading the kernel: {e}")))?;
                Ok(buf)
            }
        }
    }
}

/// An SPK kernel, indexed.
#[derive(Debug)]
pub struct SpkProvider {
    /// The kernel files; each segment says which one it is in.
    stores: Vec<Store>,
    /// Keyed by (target, center); a pair can have several segments covering different time spans.
    segs: HashMap<(i32, i32), Vec<Segment>>,
    identity: String,
    jd_min: f64,
    jd_max: f64,
}

fn rd_f64(d: &Store, off: usize) -> Result<f64> {
    d.bytes(off, 8)
        .map(|b| f64::from_le_bytes(b.try_into().expect("8 bytes")))
}

fn rd_i32(d: &Store, off: usize) -> Result<i32> {
    d.bytes(off, 4)
        .map(|b| i32::from_le_bytes(b.try_into().expect("4 bytes")))
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
        let cannot = |e: std::io::Error| EshanaError::Provider(format!("cannot read '{path}': {e}"));
        let len = std::fs::metadata(path).map_err(cannot)?.len();
        if len > IN_MEMORY_MAX {
            let file = std::fs::File::open(path).map_err(cannot)?;
            return Self::from_store(
                Store::File {
                    file: std::sync::Mutex::new(file),
                    len: len as usize,
                },
                path,
            );
        }
        Self::from_bytes(std::fs::read(path).map_err(cannot)?, path)
    }

    /// Several kernels at once (a planetary kernel plus, say, an asteroid kernel): the first sets the date range;
    /// a body is read from whichever kernel carries it (the first one that does).
    ///
    /// # Errors
    /// As [`SpkProvider::open`], for any of the files.
    pub fn open_many(paths: &[String]) -> Result<Self> {
        let mut it = paths.iter();
        let first = it
            .next()
            .ok_or_else(|| EshanaError::Config("no kernel given".into()))?;
        let mut base = Self::open(first)?;
        for path in it {
            let more = Self::open(path)?;
            let offset = base.stores.len();
            for (key, list) in more.segs {
                let entry = base.segs.entry(key).or_default();
                entry.extend(list.into_iter().map(|mut s| {
                    s.kernel += offset;
                    s
                }));
            }
            base.stores.extend(more.stores);
            base.identity = format!("{} + {}", base.identity, more.identity);
        }
        Ok(base)
    }

    /// Index a kernel already in memory (used by tests).
    ///
    /// # Errors
    /// [`EshanaError::Provider`] when the bytes are not a little-endian SPK.
    pub fn from_bytes(data: Vec<u8>, name: &str) -> Result<Self> {
        Self::from_store(Store::Mem(data), name)
    }

    fn from_store(data: Store, name: &str) -> Result<Self> {
        if data.len() < 1024 {
            return Err(EshanaError::Provider(format!("'{name}' is not a DAF/SPK file")));
        }
        let head = data.bytes(0, 1024)?;
        if !head.starts_with(b"DAF/SPK") {
            return Err(EshanaError::Provider(format!("'{name}' is not a DAF/SPK file")));
        }
        if !head[88..96].starts_with(b"LTL-IEEE") {
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
        let internal = String::from_utf8_lossy(&head[16..76]).trim().to_string();
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
                if ty != 2 && ty != 3 && ty != 21 {
                    continue; // other segment types are ignored; planetary DE kernels use type 2
                }
                min_et = min_et.min(start_et);
                max_et = max_et.max(end_et);
                segs.entry((target, center)).or_default().push(Segment {
                    start_et,
                    end_et,
                    start_addr: sa,
                    end_addr: ea,
                    ty,
                    kernel: 0,
                });
            }
            rec = next;
        }
        if segs.is_empty() {
            return Err(EshanaError::Provider(
                "no type-2, type-3 or type-21 segments found in kernel".into(),
            ));
        }
        // a kernel in memory is hashed whole; a large one by its first and last MiB (it identifies, not authenticates)
        let identity = match &data {
            Store::Mem(d) => format!(
                "{} | {} | {} bytes | fnv1a64 {:016x}",
                name_of(name),
                internal,
                d.len(),
                fnv1a64(d)
            ),
            Store::File { len, .. } => {
                let mib = 1 << 20;
                let mut sample = data.bytes(0, mib.min(*len))?;
                sample.extend(data.bytes(len.saturating_sub(mib), mib.min(*len))?);
                format!(
                    "{} | {} | {} bytes | fnv1a64(head+tail MiB) {:016x}",
                    name_of(name),
                    internal,
                    len,
                    fnv1a64(&sample)
                )
            }
        };
        Ok(Self {
            stores: vec![data],
            segs,
            identity,
            jd_min: JD_J2000 + min_et / 86_400.0,
            jd_max: JD_J2000 + max_et / 86_400.0,
        })
    }

    /// Type 21 (extended modified difference arrays): the record covering `et` evaluated as SPICE's SPKE21 does.
    ///
    /// Segment layout: N records of 4·MAXDIM + 11 doubles (TL, G[MAXDIM], X VX Y VY Z VZ, DT[MAXDIM×3], KQMAX1,
    /// KQ[3]), then the N record end epochs, then a directory of every 100th epoch, then MAXDIM and N.
    fn type21_state(&self, seg: &Segment, et: f64) -> Result<([f64; 3], [f64; 3])> {
        let st = &self.stores[seg.kernel];
        let a = |addr: usize| (addr - 1) * 8;
        let maxdim = rd_f64(st, a(seg.end_addr - 1))? as usize;
        let n = rd_f64(st, a(seg.end_addr))? as usize;
        let dlsize = 4 * maxdim + 11;
        let epochs0 = seg.start_addr + n * dlsize;
        // the first record whose end epoch is at or after et
        let (mut lo, mut hi) = (0usize, n - 1);
        while lo < hi {
            let mid = (lo + hi) / 2;
            if rd_f64(st, a(epochs0 + mid))? < et {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let rec: Vec<f64> = st
            .bytes(a(seg.start_addr + lo * dlsize), dlsize * 8)?
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().expect("8 bytes")))
            .collect();
        let tl = rec[0];
        let g = &rec[1..=maxdim];
        let refs = &rec[maxdim + 1..maxdim + 7];
        let (refpos, refvel) = ([refs[0], refs[2], refs[4]], [refs[1], refs[3], refs[5]]);
        let dt = |j: usize, i: usize| rec[maxdim + 7 + i * maxdim + j]; // DT(j, i), j and i from 0
        let kqmax1 = rec[4 * maxdim + 7] as usize;
        let kq = [
            rec[4 * maxdim + 8] as usize,
            rec[4 * maxdim + 9] as usize,
            rec[4 * maxdim + 10] as usize,
        ];
        let delta = et - tl;
        let mut tp = delta;
        let mq2 = kqmax1.saturating_sub(2);
        let mut ks = kqmax1 - 1;
        let mut fc = vec![0.0; maxdim + 2];
        let mut wc = vec![0.0; maxdim + 2];
        let mut w = vec![0.0; maxdim + 3];
        fc[1] = 1.0; // FC(1), 1-based in the reference algorithm
        for j in 1..=mq2 {
            fc[j + 1] = tp / g[j - 1];
            wc[j] = delta / g[j - 1];
            tp = delta + g[j - 1];
        }
        for j in 1..=kqmax1 {
            w[j] = 1.0 / j as f64;
        }
        let mut jx = 0;
        let mut ks1 = ks.saturating_sub(1);
        while ks >= 2 {
            jx += 1;
            for j in 1..=jx {
                w[j + ks] = fc[j + 1] * w[j + ks - 1] - wc[j] * w[j + ks];
            }
            ks = ks1;
            ks1 = ks1.saturating_sub(1);
        }
        let (mut p, mut v) = ([0.0; 3], [0.0; 3]);
        for i in 0..3 {
            let sum: f64 = (1..=kq[i]).rev().map(|j| dt(j - 1, i) * w[j + ks]).sum();
            p[i] = refpos[i] + delta * (refvel[i] + delta * sum);
        }
        for j in 1..=jx {
            w[j + ks] = fc[j + 1] * w[j + ks - 1] - wc[j] * w[j + ks];
        }
        ks -= 1;
        for i in 0..3 {
            let sum: f64 = (1..=kq[i]).rev().map(|j| dt(j - 1, i) * w[j + ks]).sum();
            v[i] = refvel[i] + delta * sum;
        }
        Ok((p, v))
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
        if seg.ty == 21 {
            return self.type21_state(seg, et);
        }
        // Directory at the end of the segment: INIT, INTLEN, RSIZE, N.
        let a = |addr: usize| (addr - 1) * 8;
        let end = seg.end_addr;
        let init = rd_f64(&self.stores[seg.kernel], a(end - 3))?;
        let intlen = rd_f64(&self.stores[seg.kernel], a(end - 2))?;
        let rsize = rd_f64(&self.stores[seg.kernel], a(end - 1))? as usize;
        let n = rd_f64(&self.stores[seg.kernel], a(end))? as usize;
        let idx = (((et - init) / intlen).floor() as usize).min(n - 1);
        let rec0 = seg.start_addr + idx * rsize;
        // one read for the whole record: MID, RADIUS, then the coefficients
        let rec: Vec<f64> = self.stores[seg.kernel]
            .bytes(a(rec0), rsize * 8)?
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().expect("8 bytes")))
            .collect();
        let (mid, radius) = (rec[0], rec[1]);
        // type 2: x, y, z position coefficients; type 3: the same, then vx, vy, vz velocity coefficients
        let ncoef = (rsize - 2) / if seg.ty == 3 { 6 } else { 3 };
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
                let coef = rec[2 + c * ncoef + k];
                p[c] += coef * tk[k];
                if seg.ty == 2 {
                    v[c] += coef * dk[k];
                } else {
                    v[c] += rec[2 + (3 + c) * ncoef + k] * tk[k];
                }
            }
            if seg.ty == 2 {
                v[c] /= radius;
            }
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
            0 => Ok(([0.0; 3], [0.0; 3])),
            other if self.segs.contains_key(&(other, 0)) => self.segment_state(other, 0, et),
            // a body stored relative to another centre (asteroids are usually relative to the Sun, 10)
            other => {
                let centre = self
                    .segs
                    .keys()
                    .find(|(t, _)| *t == other)
                    .map(|(_, c)| *c)
                    .ok_or_else(|| {
                        EshanaError::BodyUnsupported(format!(
                            "no kernel loaded carries NAIF id {other}; add one with it (e.g. sb441-n16)"
                        ))
                    })?;
                Ok(add(
                    self.segment_state(other, centre, et)?,
                    self.bary(centre, et)?,
                ))
            }
        }
    }
}

impl SpkProvider {
    /// The id a loaded kernel uses for an asteroid: 2000000 + n (DE small-body files) or 20000000 + n (Horizons).
    fn asteroid_id(&self, id: i32) -> Option<i32> {
        let alt = 20_000_000 + (id - 2_000_000);
        [id, alt]
            .into_iter()
            .find(|c| self.segs.keys().any(|(t, _)| t == c))
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
        // asteroids: NAIF ids 2000000 + number
        Body::Ceres => 2_000_001,
        Body::Pallas => 2_000_002,
        Body::Juno => 2_000_003,
        Body::Vesta => 2_000_004,
        Body::Chiron => 2_002_060,
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
        body.is_node() || naif_id(body).is_some_and(|id| id < 1000 || self.asteroid_id(id).is_some())
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
        let id = if id >= 2_000_000 {
            self.asteroid_id(id).ok_or_else(|| {
                EshanaError::BodyUnsupported(format!("{}: no loaded kernel carries it; add one (sb441-n16 for Ceres, Pallas, Juno and Vesta; a JPL Horizons SPK for Chiron)", body.id()))
            })?
        } else {
            id
        };
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

    /// A minimal DAF/SPK in memory: one segment of the given type, one record, constant polynomials.
    fn tiny_kernel(ty: i32) -> Vec<u8> {
        let mut d = vec![0u8; 4 * 1024];
        d[..8].copy_from_slice(b"DAF/SPK ");
        d[8..12].copy_from_slice(&2i32.to_le_bytes());
        d[12..16].copy_from_slice(&6i32.to_le_bytes());
        d[16..76].copy_from_slice(&[b' '; 60]);
        d[76..80].copy_from_slice(&2i32.to_le_bytes()); // first summary record
        d[88..96].copy_from_slice(b"LTL-IEEE");
        let put = |d: &mut Vec<u8>, off: usize, v: f64| d[off..off + 8].copy_from_slice(&v.to_le_bytes());
        // summary record 2: next 0, prev 0, count 1, then the summary
        put(&mut d, 1024, 0.0);
        put(&mut d, 1024 + 16, 1.0);
        put(&mut d, 1024 + 24, -1e9);
        put(&mut d, 1024 + 32, 1e9);
        let ints = 1024 + 40;
        let ncoef = 2usize;
        let rsize = 2 + ncoef * if ty == 3 { 6 } else { 3 };
        let (sa, n_dir) = (385usize, 4usize); // data begins at word 385 (record 4)
        let ea = sa + rsize + n_dir - 1;
        for (k, v) in [10i32, 0, 1, ty, sa as i32, ea as i32].iter().enumerate() {
            d[ints + 4 * k..ints + 4 * k + 4].copy_from_slice(&v.to_le_bytes());
        }
        let w = |i: usize| (i - 1) * 8;
        let mut words = vec![0.0, 1e9]; // MID, RADIUS
        words.extend([7.0, 0.0, 8.0, 0.0, 9.0, 0.0]); // x, y, z: constant 7, 8, 9 km
        if ty == 3 {
            words.extend([0.5, 0.0, 0.25, 0.0, 0.125, 0.0]); // vx, vy, vz: constant
        }
        words.extend([-1e9, 2e9, rsize as f64, 1.0]); // INIT, INTLEN, RSIZE, N
        for (k, v) in words.iter().enumerate() {
            put(&mut d, w(sa + k), *v);
        }
        d
    }

    #[test]
    fn reads_type_2_and_type_3_segments() {
        let k2 = SpkProvider::from_bytes(tiny_kernel(2), "t2").unwrap();
        let (p, v) = k2.segment_state(10, 0, 0.0).unwrap();
        assert_eq!(p, [7.0, 8.0, 9.0]);
        assert_eq!(v, [0.0, 0.0, 0.0]); // derivative of a constant
        let k3 = SpkProvider::from_bytes(tiny_kernel(3), "t3").unwrap();
        let (p, v) = k3.segment_state(10, 0, 0.0).unwrap();
        assert_eq!(p, [7.0, 8.0, 9.0]);
        assert_eq!(v, [0.5, 0.25, 0.125]); // velocity from its own coefficients
    }

    #[test]
    fn equatorial_to_ecliptic_keeps_length() {
        let v = eq_to_ecl([0.3, -0.5, 0.8]);
        assert!((norm(v) - norm([0.3, -0.5, 0.8])).abs() < 1e-12);
    }
}
