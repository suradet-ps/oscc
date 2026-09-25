//! Role-based access decisions (AGENTS.md §2 rule 5).
//!
//! The matrix is provisional (AGENTS.md §12 item 4) and must stay in sync
//! with `docs/rbac.md`. Role alone never grants case access: the server
//! additionally checks the case assignment, and every reveal or break-glass
//! action carries a reason and an audit entry. `Admin` deliberately cannot
//! open cases.

use oscc_models::Role;

/// A capability a role may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Open a case the user is assigned to.
    ViewCase,
    /// Unmask patient identity through the reveal dialog.
    RevealIdentity,
    /// Edit case fields.
    UpdateCase,
    /// Record deadline tasks as done or waived.
    CompleteTask,
    /// Close a case.
    CloseCase,
    /// See aggregate, non-identifying dashboards.
    ViewDashboard,
    /// Export aggregate data.
    ExportAggregate,
    /// Take emergency access to a case outside the assignment.
    OpenBreakGlass,
}

const ER_NURSE: &[Permission] = &[
    Permission::ViewCase,
    Permission::UpdateCase,
    Permission::CompleteTask,
    Permission::ViewDashboard,
    Permission::OpenBreakGlass,
];

const FORENSIC_PHYSICIAN: &[Permission] = &[
    Permission::ViewCase,
    Permission::RevealIdentity,
    Permission::UpdateCase,
    Permission::CompleteTask,
    Permission::CloseCase,
    Permission::ViewDashboard,
];

const SOCIAL_WORKER: &[Permission] = &[
    Permission::ViewCase,
    Permission::RevealIdentity,
    Permission::UpdateCase,
    Permission::CompleteTask,
    Permission::ViewDashboard,
];

const PSYCHOLOGIST: &[Permission] = &[
    Permission::ViewCase,
    Permission::UpdateCase,
    Permission::CompleteTask,
    Permission::ViewDashboard,
];

const OSCC_LEAD: &[Permission] = &[
    Permission::ViewCase,
    Permission::RevealIdentity,
    Permission::UpdateCase,
    Permission::CompleteTask,
    Permission::CloseCase,
    Permission::ViewDashboard,
    Permission::ExportAggregate,
    Permission::OpenBreakGlass,
];

const ADMIN: &[Permission] = &[Permission::ViewDashboard];

/// Every permission granted to `role`.
pub fn permissions(role: Role) -> &'static [Permission] {
    match role {
        Role::ErNurse => ER_NURSE,
        Role::ForensicPhysician => FORENSIC_PHYSICIAN,
        Role::SocialWorker => SOCIAL_WORKER,
        Role::Psychologist => PSYCHOLOGIST,
        Role::OsccLead => OSCC_LEAD,
        Role::Admin => ADMIN,
    }
}

/// Returns `true` when `role` holds `permission`.
pub fn allows(role: Role, permission: Permission) -> bool {
    permissions(role).contains(&permission)
}

/// Returns `true` when the permission requires a recorded reason
/// (DESIGN.md privacy rules: reveal, export, and break-glass are two-step,
/// reasoned actions).
pub fn requires_reason(permission: Permission) -> bool {
    matches!(
        permission,
        Permission::RevealIdentity | Permission::ExportAggregate | Permission::OpenBreakGlass
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_can_see_something() {
        for role in [
            Role::ErNurse,
            Role::ForensicPhysician,
            Role::SocialWorker,
            Role::Psychologist,
            Role::OsccLead,
            Role::Admin,
        ] {
            assert!(
                !permissions(role).is_empty(),
                "{role:?} should hold at least one permission"
            );
        }
    }

    #[test]
    fn admin_cannot_open_cases() {
        assert!(!allows(Role::Admin, Permission::ViewCase));
        assert!(!allows(Role::Admin, Permission::RevealIdentity));
        assert!(allows(Role::Admin, Permission::ViewDashboard));
    }

    #[test]
    fn only_documented_roles_may_reveal_identity() {
        for role in [Role::ErNurse, Role::Psychologist, Role::Admin] {
            assert!(
                !allows(role, Permission::RevealIdentity),
                "{role:?} must not reveal identity"
            );
        }
        for role in [Role::ForensicPhysician, Role::SocialWorker, Role::OsccLead] {
            assert!(
                allows(role, Permission::RevealIdentity),
                "{role:?} should reveal identity"
            );
        }
    }

    #[test]
    fn only_lead_may_export_aggregates() {
        assert!(allows(Role::OsccLead, Permission::ExportAggregate));
        assert!(!allows(Role::ErNurse, Permission::ExportAggregate));
        assert!(!allows(
            Role::ForensicPhysician,
            Permission::ExportAggregate
        ));
    }

    #[test]
    fn close_case_is_clinician_or_lead_only() {
        assert!(allows(Role::ForensicPhysician, Permission::CloseCase));
        assert!(allows(Role::OsccLead, Permission::CloseCase));
        assert!(!allows(Role::ErNurse, Permission::CloseCase));
        assert!(!allows(Role::SocialWorker, Permission::CloseCase));
    }

    #[test]
    fn reason_is_required_for_the_three_sensitive_actions() {
        assert!(requires_reason(Permission::RevealIdentity));
        assert!(requires_reason(Permission::ExportAggregate));
        assert!(requires_reason(Permission::OpenBreakGlass));
        assert!(!requires_reason(Permission::ViewCase));
        assert!(!requires_reason(Permission::CompleteTask));
    }
}
