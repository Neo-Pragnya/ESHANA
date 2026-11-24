//! ESHANA engine: houses, Vedic, Western, Human Design, numerology and a symbolic graph, all with provenance.
//!
//! Start at [`api::Engine::execute_json`]: one JSON request in, one JSON envelope out.

pub mod analysis;
pub mod annual;
pub mod api;
pub mod astro;
pub mod auxiliary;
pub mod bala;
pub mod chakras;
pub mod dashas_ext;
pub mod dated;
pub mod events;
pub mod graph;
pub mod hd;
pub mod jyotish;
pub mod kp;
pub mod maitri;
pub mod matching;
pub mod matching_ext;
pub mod methods;
pub mod muhurta;
pub mod numerology;
pub mod numerology_ext;
pub mod packs;
pub mod panchanga;
pub mod progress;
pub mod research;
pub mod search;
pub mod synthetic;
pub mod vargas_ext;
pub mod vedic;

pub use api::{Engine, Request, ENGINE_VERSION};
