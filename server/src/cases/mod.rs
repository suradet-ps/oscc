//! Case registry: intake, queue, and detail (M2).
//!
//! Identity is masked on every response; the full snapshot leaves the
//! server only through the audited reveal path (M2-4).

pub mod dto;
pub mod mask;
pub mod routes;
pub mod store;
