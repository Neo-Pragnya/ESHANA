//! The JPL planetary kernels ESHANA knows, where they come from and how to check them (PROV-06).
//!
//! ESHANA never downloads anything by itself: the engine stays offline. A kernel is fetched by the user (or by
//! `scripts/fetch_kernel.py` in the repository), then checked here against the MD5 that NAIF publishes in
//! `aa_checksums.txt` next to the files. A catalogue name (`de440`) given as the SPK path resolves to the file in the
//! kernel folder: `$ESHANA_KERNEL_DIR`, else `$XDG_CACHE_HOME/eshana/kernels`, else `~/.cache/eshana/kernels`.

use eshana_core::{EshanaError, Result};
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Where the planetary kernels are published.
pub const NAIF_PLANETS: &str = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/";

/// One kernel: its file, size, the MD5 NAIF publishes, and the span it covers (calendar dates as NAIF prints them;
/// negative years are astronomical, so -13200 is 13201 BC).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Kernel {
    /// Catalogue name, e.g. `de440`.
    pub name: &'static str,
    /// File name at NAIF.
    pub file: &'static str,
    /// Size in bytes.
    pub bytes: u64,
    /// MD5 from NAIF's `aa_checksums.txt`.
    pub md5: &'static str,
    /// First date covered (TDB), `YYYY-MM-DD`.
    pub start: &'static str,
    /// Last date covered (TDB).
    pub end: &'static str,
    /// What it is for.
    pub note: &'static str,
}

/// The planetary kernels, as published by NAIF (checked 2026-11-24).
pub const KERNELS: &[Kernel] = &[
    Kernel {
        name: "de440s",
        file: "de440s.bsp",
        bytes: 32_726_016,
        md5: "3917ee56769db332790c751e2168843d",
        start: "1849-12-26",
        end: "2150-01-22",
        note: "short DE440: modern charts (the golden tests use it)",
    },
    Kernel {
        name: "de440",
        file: "de440.bsp",
        bytes: 119_799_808,
        md5: "c9d581bfd84209dbeee8b1583939b148",
        start: "1549-12-31",
        end: "2650-01-25",
        note: "DE440: from the Gregorian reform to 2650",
    },
    Kernel {
        name: "de441-part1",
        file: "de441_part-1.bsp",
        bytes: 1_651_119_104,
        md5: "7e5fcf9ecb5d08e1ab70c049baa60cd3",
        start: "-13200-05-06",
        end: "1969-07-30",
        note: "DE441 part 1: ancient dates (read on demand, not loaded into memory)",
    },
    Kernel {
        name: "de441-part2",
        file: "de441_part-2.bsp",
        bytes: 1_656_830_976,
        md5: "ad8dfa4e505ef0e3a5d587a5b4705632",
        start: "1969-06-28",
        end: "17191-03-15",
        note: "DE441 part 2: the far future",
    },
    Kernel {
        name: "de442s",
        file: "de442s.bsp",
        bytes: 32_701_440,
        md5: "cc49327e06088124c0e39d8dde9f0b58",
        start: "1849-12-26",
        end: "2150-01-22",
        note: "short DE442 (2025)",
    },
    Kernel {
        name: "de442",
        file: "de442.bsp",
        bytes: 119_771_136,
        md5: "446656322267e7b819a26cb08a0d8718",
        start: "1549-12-31",
        end: "2650-01-25",
        note: "DE442 (2025)",
    },
];

/// The catalogue entry for a name (`de440`) or a file name (`de440.bsp`).
#[must_use]
pub fn find(name: &str) -> Option<&'static Kernel> {
    let n = name.trim().to_ascii_lowercase();
    KERNELS.iter().find(|k| k.name == n || k.file == n)
}

/// The folder where catalogue names are looked up.
#[must_use]
pub fn kernel_dir() -> PathBuf {
    if let Ok(d) = std::env::var("ESHANA_KERNEL_DIR") {
        return PathBuf::from(d);
    }
    let cache = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .ok()
        .or_else(|| {
            std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .ok()
                .map(|h| Path::new(&h).join(".cache"))
        });
    cache
        .unwrap_or_else(|| PathBuf::from("."))
        .join("eshana")
        .join("kernels")
}

/// An SPK path as given: a catalogue name (no folder, no extension) becomes its file in [`kernel_dir`].
///
/// # Errors
/// `ESH-CONFIG` when a catalogue name is given but the file is not in the kernel folder (with how to fetch it).
pub fn resolve(spec: &str) -> Result<String> {
    let is_name = !spec.contains('/') && !spec.contains('\\') && !Path::new(spec).exists();
    match (is_name, find(spec)) {
        (true, Some(k)) => {
            let path = kernel_dir().join(k.file);
            if path.exists() {
                Ok(path.to_string_lossy().into_owned())
            } else {
                Err(EshanaError::Config(format!(
                    "kernel {} is not in {}: fetch it with `python scripts/fetch_kernel.py {}` from the repository, or download {}{} \
                     ({} bytes) and check it with `eshana kernel verify`",
                    k.name,
                    kernel_dir().display(),
                    k.name,
                    NAIF_PLANETS,
                    k.file,
                    k.bytes
                )))
            }
        }
        _ => Ok(spec.to_string()),
    }
}

/// The result of checking a kernel file.
#[derive(Debug, Clone, Serialize)]
pub struct Verification {
    /// The file checked.
    pub path: String,
    /// Its size in bytes.
    pub bytes: u64,
    /// Its MD5.
    pub md5: String,
    /// The catalogue entry it matches by file name, if any.
    pub kernel: Option<&'static str>,
    /// `verified` (size and MD5 match NAIF), `mismatch`, or `unknown` (not in the catalogue).
    pub status: &'static str,
}

/// Hash a kernel and compare it with the catalogue.
///
/// # Errors
/// `ESH-PROVIDER` when the file cannot be read.
pub fn verify(path_or_name: &str) -> Result<Verification> {
    let path = match find(path_or_name) {
        Some(k) if !Path::new(path_or_name).exists() => kernel_dir().join(k.file),
        _ => PathBuf::from(path_or_name),
    };
    let cannot = |e: std::io::Error| EshanaError::Provider(format!("cannot read '{}': {e}", path.display()));
    let mut f = std::fs::File::open(&path).map_err(cannot)?;
    let mut md5 = Md5::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut bytes = 0u64;
    loop {
        let n = f.read(&mut buf).map_err(cannot)?;
        if n == 0 {
            break;
        }
        md5.update(&buf[..n]);
        bytes += n as u64;
    }
    let digest = md5.hex();
    let file = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let k = find(&file);
    let status = match k {
        Some(k) if k.bytes == bytes && k.md5 == digest => "verified",
        Some(_) => "mismatch",
        None => "unknown",
    };
    Ok(Verification {
        path: path.to_string_lossy().into_owned(),
        bytes,
        md5: digest,
        kernel: k.map(|k| k.name),
        status,
    })
}

/// MD5 (RFC 1321), only to compare a downloaded kernel with the checksum NAIF publishes. Not a security hash.
struct Md5 {
    state: [u32; 4],
    buf: Vec<u8>,
    len: u64,
}

const S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5,
    9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6,
    10, 15, 21, 6, 10, 15, 21,
];

impl Md5 {
    fn new() -> Self {
        Self {
            state: [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476],
            buf: Vec::with_capacity(64),
            len: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.len += data.len() as u64;
        if !self.buf.is_empty() {
            let take = (64 - self.buf.len()).min(data.len());
            self.buf.extend_from_slice(&data[..take]);
            data = &data[take..];
            if self.buf.len() == 64 {
                let block: [u8; 64] = self.buf[..].try_into().expect("64");
                self.block(&block);
                self.buf.clear();
            }
        }
        let mut chunks = data.chunks_exact(64);
        for c in &mut chunks {
            self.block(c.try_into().expect("64"));
        }
        self.buf.extend_from_slice(chunks.remainder());
    }

    fn block(&mut self, b: &[u8; 64]) {
        let m: Vec<u32> = b
            .chunks_exact(4)
            .map(|w| u32::from_le_bytes(w.try_into().expect("4")))
            .collect();
        let [mut a, mut bb, mut c, mut d] = self.state;
        for (i, &shift) in S.iter().enumerate() {
            let (f, g) = match i / 16 {
                0 => ((bb & c) | (!bb & d), i),
                1 => ((d & bb) | (!d & c), (5 * i + 1) % 16),
                2 => (bb ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (bb | !d), (7 * i) % 16),
            };
            let k = ((i as f64 + 1.0).sin().abs() * 4_294_967_296.0) as u32;
            let tmp = d;
            d = c;
            c = bb;
            bb = bb.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(k)
                    .wrapping_add(m[g])
                    .rotate_left(shift),
            );
            a = tmp;
        }
        self.state = [
            self.state[0].wrapping_add(a),
            self.state[1].wrapping_add(bb),
            self.state[2].wrapping_add(c),
            self.state[3].wrapping_add(d),
        ];
    }

    fn hex(mut self) -> String {
        let bits = self.len.wrapping_mul(8);
        let mut pad = vec![0x80u8];
        pad.resize(((119 - (self.len % 64)) % 64 + 1) as usize, 0);
        pad.extend_from_slice(&bits.to_le_bytes());
        let keep = self.len;
        self.update(&pad);
        self.len = keep;
        self.state
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md5_of(s: &[u8]) -> String {
        let mut m = Md5::new();
        m.update(s);
        m.hex()
    }

    #[test]
    fn md5_matches_rfc_1321() {
        assert_eq!(md5_of(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_of(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(md5_of(b"message digest"), "f96b697d7cb7938d525a2f31aaf161d0");
        assert_eq!(
            md5_of(b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"),
            "57edf4a22be3c955ac49da2e2107b67a"
        );
        // split updates give the same digest
        let mut m = Md5::new();
        m.update(b"message ");
        m.update(b"digest");
        assert_eq!(m.hex(), "f96b697d7cb7938d525a2f31aaf161d0");
    }

    #[test]
    fn catalogue_names_resolve() {
        assert_eq!(find("de440").unwrap().file, "de440.bsp");
        assert_eq!(find("de441_part-1.bsp").unwrap().name, "de441-part1");
        assert!(find("de999").is_none());
        // a path or an unknown name is passed through unchanged
        assert_eq!(resolve("/tmp/x.bsp").unwrap(), "/tmp/x.bsp");
        assert_eq!(resolve("mykernel.bsp").unwrap(), "mykernel.bsp");
    }
}
