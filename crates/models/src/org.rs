//! Departments and roles used by access decisions and case ownership.

use serde::{Deserialize, Serialize};

/// The hospital units that participate in OSCC care.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Department {
    /// Emergency room.
    Emergency,
    /// Forensic medicine.
    Forensic,
    /// Social work.
    SocialWork,
    /// Psychology / mental health.
    Psychology,
    /// The OSCC team itself.
    Oscc,
    /// Hospital IT.
    It,
}

/// A signed-in user's role. The permission matrix lives in
/// `oscc-core::rbac` and is provisionally documented in `docs/rbac.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Emergency nurse: registers and screens cases.
    ErNurse,
    /// Forensic physician: examinations and clinical closure.
    ForensicPhysician,
    /// Social worker: welfare plans and coordination.
    SocialWorker,
    /// Psychologist: mental-health care.
    Psychologist,
    /// OSCC lead: oversight, break-glass review, aggregate reporting.
    OsccLead,
    /// Hospital IT: system administration without case access.
    Admin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_and_departments_serialize_snake_case() {
        let role = serde_json::to_string(&Role::OsccLead).expect("serialize");
        assert_eq!(role, "\"oscc_lead\"");
        let department = serde_json::to_string(&Department::SocialWork).expect("serialize");
        assert_eq!(department, "\"social_work\"");
    }
}
