//! Permission checks for authenticated callers (AGENTS.md §2 rule 5).
//!
//! The role matrix lives in `oscc-core::rbac`; this module only adapts it
//! to the API error type. Handlers call [`AuthUser::require`] before doing
//! anything that reveals or changes case data, and reasoned actions also
//! carry a reason recorded in the audit entry.

use oscc_core::{Permission, allows};

use super::service::AuthUser;
use crate::error::AppError;

impl AuthUser {
    /// Returns `Ok(())` when the caller's role holds `permission`.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Forbidden`] when the role does not hold it.
    pub fn require(&self, permission: Permission) -> Result<(), AppError> {
        if allows(self.role, permission) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use oscc_models::Role;

    use super::*;

    fn user(role: Role) -> AuthUser {
        AuthUser {
            user_id: 1,
            username: "tester".to_string(),
            display_name: "Tester".to_string(),
            role,
            token_hash: "hash".to_string(),
        }
    }

    #[test]
    fn permitted_roles_pass() {
        assert!(user(Role::ErNurse).require(Permission::ViewCase).is_ok());
        assert!(
            user(Role::OsccLead)
                .require(Permission::ExportAggregate)
                .is_ok()
        );
    }

    #[test]
    fn unpermitted_roles_are_forbidden() {
        assert!(user(Role::Admin).require(Permission::ViewCase).is_err());
        assert!(
            user(Role::ErNurse)
                .require(Permission::RevealIdentity)
                .is_err()
        );
        assert!(
            user(Role::Psychologist)
                .require(Permission::CloseCase)
                .is_err()
        );
    }

    #[test]
    fn forbidden_maps_to_the_right_api_error() {
        let err = user(Role::Admin)
            .require(Permission::ViewCase)
            .expect_err("admin cannot view cases");
        assert!(matches!(err, AppError::Forbidden));
    }
}
