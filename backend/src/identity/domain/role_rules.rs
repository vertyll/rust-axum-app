//! The rules that keep the roles manageable, which no single role can check alone: a domain service over the roles
//! and the accounts, given what it needs, so it does no I/O.

use super::error::IdentityError;
use super::role::Role;
use super::user::User;

/// Whether `role` may be removed: never a system role or an unrestricted one, which must be made restricted first,
/// and never while `holders` accounts hold it.
pub fn check_removal(role: &Role, holders: usize) -> Result<(), IdentityError> {
    if role.name.is_system() {
        return Err(IdentityError::SystemRole);
    }
    if role.unrestricted {
        return Err(IdentityError::UnrestrictedRole);
    }
    if holders > 0 {
        return Err(IdentityError::RoleStillAssigned);
    }
    Ok(())
}

/// Whether `role` may stop being unrestricted: only while another role of `roles` is.
pub fn check_restriction(role: &Role, roles: &[Role]) -> Result<(), IdentityError> {
    if roles.iter().any(|other| other.unrestricted && other.name != role.name) {
        Ok(())
    } else {
        Err(IdentityError::LastUnrestrictedRole)
    }
}

/// Whether `user` may lose `role`: not when it is the last unrestricted role any account of `users` holds.
pub fn check_taking(user: &User, role: &Role, roles: &[Role], users: &[User]) -> Result<(), IdentityError> {
    if !role.unrestricted || !user.has_role(&role.name) {
        return Ok(());
    }
    let unrestricted = |name: &_| roles.iter().any(|other| other.unrestricted && &other.name == name);
    let keeps_one = user.roles.iter().any(|held| held != &role.name && unrestricted(held));
    let others = users
        .iter()
        .any(|other| other.id != user.id && other.roles.iter().any(unrestricted));
    if keeps_one || others {
        Ok(())
    } else {
        Err(IdentityError::LastUnrestrictedHolder)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use jiff::Timestamp;

    use super::{check_removal, check_restriction, check_taking};
    use crate::identity::domain::{Email, IdentityError, KeycloakId, Role, RoleName, User, UserId};

    fn role(name: &str, unrestricted: bool) -> Role {
        Role {
            name: RoleName::parse(name).unwrap(),
            description: String::new(),
            unrestricted,
            permissions: BTreeSet::new(),
            created_at: Timestamp::UNIX_EPOCH,
            updated_at: Timestamp::UNIX_EPOCH,
        }
    }

    fn user(id: i64, roles: &[&str]) -> User {
        User {
            id: UserId(id),
            keycloak_id: KeycloakId::new(format!("k{id}")),
            email: Email::parse(format!("u{id}@example.com")).unwrap(),
            first_name: String::new(),
            last_name: String::new(),
            roles: roles.iter().map(|name| RoleName::parse(name).unwrap()).collect(),
            created_at: Timestamp::UNIX_EPOCH,
            updated_at: Timestamp::UNIX_EPOCH,
        }
    }

    #[test]
    fn a_role_is_removed_only_when_ordinary_restricted_and_unheld() {
        assert!(matches!(
            check_removal(&role("user", false), 0),
            Err(IdentityError::SystemRole)
        ));
        assert!(matches!(
            check_removal(&role("owner", true), 0),
            Err(IdentityError::UnrestrictedRole)
        ));
        assert!(matches!(
            check_removal(&role("editor", false), 2),
            Err(IdentityError::RoleStillAssigned)
        ));
        assert!(check_removal(&role("editor", false), 0).is_ok());
    }

    #[test]
    fn some_role_stays_unrestricted() {
        let admin = role("admin", true);
        assert!(matches!(
            check_restriction(&admin, std::slice::from_ref(&admin)),
            Err(IdentityError::LastUnrestrictedRole)
        ));
        assert!(check_restriction(&admin, &[admin.clone(), role("owner", true)]).is_ok());
    }

    #[test]
    fn somebody_keeps_an_unrestricted_role() {
        let roles = [role("admin", true), role("owner", true), role("user", false)];
        let ada = user(1, &["admin", "user"]);
        assert!(matches!(
            check_taking(&ada, &roles[0], &roles, &[ada.clone(), user(2, &["user"])]),
            Err(IdentityError::LastUnrestrictedHolder)
        ));
        assert!(check_taking(&ada, &roles[0], &roles, &[ada.clone(), user(2, &["owner"])]).is_ok());
        assert!(check_taking(&user(3, &["admin", "owner"]), &roles[0], &roles, &[]).is_ok());
        assert!(
            check_taking(&ada, &roles[2], &roles, &[]).is_ok(),
            "a restricted role is always given up"
        );
    }
}
