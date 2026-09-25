//! Local authentication: password hashing, session tokens, and the
//! authenticated-user extractor. AD/LDAP integration is a later decision
//! (docs/architecture.md open decision 1); local accounts are the Phase 1
//! baseline.

pub mod password;
pub mod routes;
pub mod service;
pub mod token;

pub use service::AuthUser;
