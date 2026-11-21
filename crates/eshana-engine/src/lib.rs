//! ESHANA engine: houses, Vedic, Western, Human Design, numerology and a symbolic graph, all with provenance.
//!
//! Start at [`api::Engine::execute_json`]: one JSON request in, one JSON envelope out.

pub mod api;
pub mod astro;
pub mod graph;
pub mod hd;
pub mod numerology;
pub mod vedic;

pub use api::{Engine, Request, ENGINE_VERSION};
