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

impl Role {
    /// The stable machine name used by storage and the API.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ErNurse => "er_nurse",
            Self::ForensicPhysician => "forensic_physician",
            Self::SocialWorker => "social_worker",
            Self::Psychologist => "psychologist",
            Self::OsccLead => "oscc_lead",
            Self::Admin => "admin",
        }
    }

    /// Parses a stored machine name back into a role.
    ///
    /// # Examples
    ///
    /// ```
    /// use oscc_models::Role;
    ///
    /// assert_eq!(Role::parse("oscc_lead"), Some(Role::OsccLead));
    /// assert_eq!(Role::parse("nope"), None);
    /// ```
    pub fn parse(raw: &str) -> Option<Self> {
        [
            Self::ErNurse,
            Self::ForensicPhysician,
            Self::SocialWorker,
            Self::Psychologist,
            Self::OsccLead,
            Self::Admin,
        ]
        .into_iter()
        .find(|role| role.as_str() == raw)
    }
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

    #[test]
    fn role_machine_names_are_stable() {
        assert_eq!(Role::ErNurse.as_str(), "er_nurse");
        assert_eq!(Role::ForensicPhysician.as_str(), "forensic_physician");
        assert_eq!(Role::Admin.as_str(), "admin");
    }

    #[test]
    fn role_parse_round_trips_every_role() {
        for role in [
            Role::ErNurse,
            Role::ForensicPhysician,
            Role::SocialWorker,
            Role::Psychologist,
            Role::OsccLead,
            Role::Admin,
        ] {
            assert_eq!(Role::parse(role.as_str()), Some(role));
        }
        assert_eq!(Role::parse("nope"), None);
    }
}
