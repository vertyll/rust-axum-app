//! Roles and what they grant: the registry of the permissions the modules
//! declare, the roles an administrator makes from them, and who holds which
//! role. Keycloak holds who has a role; this application holds what it grants.

use std::collections::BTreeSet;

use jiff::Timestamp;

use crate::identity::application::ports::{IdentityPorts, RealmAdmin};
use crate::identity::application::service::IdentityService;
use crate::identity::domain::role_rules::{check_removal, check_restriction, check_taking};
use crate::identity::domain::{
    Catalogue, IdentityError, RegisteredPermission, Role, RoleName, RoleRepository, UserId, UserRepository,
};

#[derive(Debug, Clone)]
pub struct NewRole {
    pub name: String,
    pub description: String,
    pub unrestricted: bool,
    pub permissions: BTreeSet<String>,
}

/// What a role becomes; its name stays.
#[derive(Debug, Clone)]
pub struct RoleChanges {
    pub description: String,
    pub unrestricted: bool,
    pub permissions: BTreeSet<String>,
}

impl<P: IdentityPorts> IdentityService<P> {
    /// Brings the registry in line with what the modules declare, makes the
    /// stock roles that do not exist yet and gives the existing ones only the
    /// permissions never registered before, so an administrator's changes
    /// survive every start.
    pub async fn register(&self, catalogues: &[Catalogue]) -> Result<(), IdentityError> {
        let declared = RegisteredPermission::declared_in(catalogues);
        let names: BTreeSet<&str> = declared.iter().map(|permission| permission.name.as_str()).collect();
        if let Some(duplicate) = declared
            .iter()
            .enumerate()
            .find(|(index, permission)| declared[..*index].iter().any(|earlier| earlier.name == permission.name))
        {
            return Err(IdentityError::CorruptData(format!(
                "permission {} is declared twice",
                duplicate.1.name
            )));
        }
        let stock_roles = catalogues.iter().flat_map(|catalogue| catalogue.stock_roles);
        if let Some(grant) = stock_roles
            .clone()
            .flat_map(|stock| stock.grants)
            .find(|grant| !names.contains(*grant))
        {
            return Err(IdentityError::UnknownPermission((*grant).to_owned()));
        }

        let seen: BTreeSet<String> = self
            .roles
            .permissions()
            .await?
            .into_iter()
            .map(|permission| permission.name)
            .collect();
        self.roles.register(&declared).await?;

        let now = Timestamp::now();
        for stock in stock_roles {
            let name = RoleName::parse(stock.name)?;
            let grants = stock.grants.iter().map(|grant| (*grant).to_owned());
            match self.roles.find(&name).await? {
                None => {
                    if let Err(err) = self.realm.create_role(&name, stock.description).await {
                        tracing::warn!(role = %name, "could not make the realm role in Keycloak: {err}");
                    }
                    self.roles
                        .create(&Role {
                            name,
                            description: stock.description.to_owned(),
                            unrestricted: stock.unrestricted,
                            permissions: grants.collect(),
                            created_at: now,
                            updated_at: now,
                        })
                        .await?;
                }
                Some(mut role) => {
                    let before = role.permissions.len();
                    role.permissions.extend(grants.filter(|grant| !seen.contains(grant)));
                    if role.permissions.len() != before {
                        role.updated_at = now;
                        self.roles.update(&role).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn permissions(&self) -> Result<Vec<RegisteredPermission>, IdentityError> {
        self.roles.permissions().await
    }

    pub async fn list_roles(&self) -> Result<Vec<Role>, IdentityError> {
        let mut roles = self.roles.list().await?;
        roles.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(roles)
    }

    pub async fn get_role(&self, name: &str) -> Result<Role, IdentityError> {
        self.role(&RoleName::parse(name).map_err(|_| IdentityError::RoleNotFound)?)
            .await
    }

    /// Makes the role here and the realm role in Keycloak, which is what a
    /// person is given.
    pub async fn create_role(&self, new: NewRole) -> Result<Role, IdentityError> {
        let name = RoleName::parse(&new.name)?;
        if self.roles.find(&name).await?.is_some() {
            return Err(IdentityError::RoleAlreadyExists);
        }
        self.check_registered(&new.permissions).await?;
        let now = Timestamp::now();
        let role = Role {
            name,
            description: new.description,
            unrestricted: new.unrestricted,
            permissions: new.permissions,
            created_at: now,
            updated_at: now,
        };
        self.realm.create_role(&role.name, &role.description).await?;
        self.roles.create(&role).await?;
        tracing::info!(role = %role.name, "role created");
        Ok(role)
    }

    /// Changes what a role grants. The last unrestricted role stays
    /// unrestricted, so somebody can always manage the roles.
    pub async fn update_role(&self, name: &str, changes: RoleChanges) -> Result<Role, IdentityError> {
        let mut role = self.get_role(name).await?;
        self.check_registered(&changes.permissions).await?;
        if role.unrestricted && !changes.unrestricted {
            check_restriction(&role, &self.roles.list().await?)?;
        }
        role.description = changes.description;
        role.unrestricted = changes.unrestricted;
        role.permissions = changes.permissions;
        role.updated_at = Timestamp::now();
        self.roles.update(&role).await?;
        tracing::info!(role = %role.name, "role updated");
        Ok(role)
    }

    /// Removes a role here, then in Keycloak; a realm role left behind when
    /// Keycloak cannot be reached grants nothing, as no role here matches it.
    pub async fn delete_role(&self, name: &str) -> Result<(), IdentityError> {
        let role = self.get_role(name).await?;
        let holders = self
            .users
            .list()
            .await?
            .iter()
            .filter(|user| user.has_role(&role.name))
            .count();
        check_removal(&role, holders)?;
        self.roles.delete(&role.name).await?;
        if let Err(err) = self.realm.delete_role(&role.name).await {
            tracing::warn!(role = %role.name, "could not remove the realm role from Keycloak: {err}");
        }
        tracing::info!(role = %role.name, "role deleted");
        Ok(())
    }

    /// Gives a person a role in Keycloak; their token carries it, and this
    /// application honours it, from their next refresh.
    pub async fn assign_role(&self, user: UserId, name: &str) -> Result<(), IdentityError> {
        let user = self.get_user(user).await?;
        let role = self.get_role(name).await?;
        self.realm.assign_role(&user.keycloak_id, &role.name).await?;
        tracing::info!(user = %user.id, role = %role.name, "role assigned");
        Ok(())
    }

    /// Takes a role from a person in Keycloak. The last person holding an
    /// unrestricted role keeps it, so somebody can always manage the roles.
    pub async fn remove_role(&self, user: UserId, name: &str) -> Result<(), IdentityError> {
        let user = self.get_user(user).await?;
        let role = self.get_role(name).await?;
        check_taking(&user, &role, &self.roles.list().await?, &self.users.list().await?)?;
        self.realm.remove_role(&user.keycloak_id, &role.name).await?;
        tracing::info!(user = %user.id, role = %role.name, "role removed");
        Ok(())
    }

    async fn role(&self, name: &RoleName) -> Result<Role, IdentityError> {
        self.roles.find(name).await?.ok_or(IdentityError::RoleNotFound)
    }

    async fn check_registered(&self, permissions: &BTreeSet<String>) -> Result<(), IdentityError> {
        let registered: BTreeSet<String> = self
            .roles
            .permissions()
            .await?
            .into_iter()
            .map(|permission| permission.name)
            .collect();
        match permissions.difference(&registered).next() {
            Some(unknown) => Err(IdentityError::UnknownPermission(unknown.clone())),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::identity::application::permissions::CATALOGUE;
    use crate::identity::application::testing::{Harness, identity};
    use crate::identity::application::{NewRole, RoleChanges};
    use crate::identity::domain::{
        Catalogue, Declared, IdentityError, RoleName, RoleRepository, StockRole, UserRepository,
    };

    fn names(permissions: &[&str]) -> BTreeSet<String> {
        permissions.iter().map(ToString::to_string).collect()
    }

    fn new_role(name: &str, unrestricted: bool, permissions: &[&str]) -> NewRole {
        NewRole {
            name: name.into(),
            description: String::new(),
            unrestricted,
            permissions: names(permissions),
        }
    }

    fn changes(unrestricted: bool, permissions: &[&str]) -> RoleChanges {
        RoleChanges {
            description: String::new(),
            unrestricted,
            permissions: names(permissions),
        }
    }

    const fn declared(name: &'static str) -> Declared {
        Declared { name, description: "" }
    }

    #[tokio::test]
    async fn registering_makes_the_stock_roles_once() {
        let harness = Harness::new().await;
        let roles = harness.service.list_roles().await.unwrap();

        let shipped: Vec<(&str, bool)> = roles
            .iter()
            .map(|role| (role.name.as_str(), role.unrestricted))
            .collect();
        assert_eq!(shipped, [("admin", true), ("user", false)]);
        assert_eq!(harness.realm.calls(), ["create ADMIN", "create USER"]);

        harness
            .service
            .update_role("user", changes(false, &["USERS_VIEW"]))
            .await
            .unwrap();
        harness.service.register(&[CATALOGUE]).await.unwrap();
        let user = harness.service.get_role("user").await.unwrap();
        assert_eq!(
            user.permissions,
            names(&["USERS_VIEW"]),
            "an administrator's change survives a start"
        );
    }

    #[tokio::test]
    async fn a_stock_role_gains_only_permissions_never_seen_before() {
        const FIRST: Catalogue = Catalogue {
            module: "reports",
            permissions: &[declared("REPORTS_VIEW")],
            stock_roles: &[StockRole {
                name: "analyst",
                description: "",
                unrestricted: false,
                grants: &["REPORTS_VIEW"],
            }],
        };
        const LATER: Catalogue = Catalogue {
            module: "reports",
            permissions: &[declared("REPORTS_VIEW"), declared("REPORTS_EXPORT")],
            stock_roles: &[StockRole {
                name: "analyst",
                description: "",
                unrestricted: false,
                grants: &["REPORTS_VIEW", "REPORTS_EXPORT"],
            }],
        };
        let harness = Harness::new().await;
        harness.service.register(&[CATALOGUE, FIRST]).await.unwrap();
        harness
            .service
            .update_role("analyst", changes(false, &[]))
            .await
            .unwrap();

        harness.service.register(&[CATALOGUE, LATER]).await.unwrap();

        let analyst = harness.service.get_role("analyst").await.unwrap();
        assert_eq!(analyst.permissions, names(&["REPORTS_EXPORT"]));
    }

    #[tokio::test]
    async fn a_permission_no_module_declares_is_withdrawn_from_every_role() {
        const FEWER: Catalogue = Catalogue {
            permissions: &[declared("USERS_VIEW")],
            ..CATALOGUE
        };

        let harness = Harness::new().await;
        harness
            .service
            .create_role(new_role("auditor", false, &["USERS_VIEW", "ROLES_VIEW"]))
            .await
            .unwrap();
        harness.service.register(&[FEWER]).await.unwrap();

        let auditor = harness.service.get_role("auditor").await.unwrap();
        assert_eq!(auditor.permissions, names(&["USERS_VIEW"]));
    }

    #[tokio::test]
    async fn a_catalogue_that_contradicts_itself_stops_the_start() {
        const TWICE: Catalogue = Catalogue {
            module: "twice",
            permissions: &[declared("USERS_VIEW")],
            stock_roles: &[],
        };
        const UNDECLARED: Catalogue = Catalogue {
            module: "undeclared",
            permissions: &[],
            stock_roles: &[StockRole {
                name: "x",
                description: "",
                unrestricted: false,
                grants: &["NOWHERE"],
            }],
        };
        let harness = Harness::new().await;
        assert!(matches!(
            harness.service.register(&[CATALOGUE, TWICE]).await,
            Err(IdentityError::CorruptData(_))
        ));
        assert!(matches!(
            harness.service.register(&[CATALOGUE, UNDECLARED]).await,
            Err(IdentityError::UnknownPermission(name)) if name == "NOWHERE"
        ));
    }

    #[tokio::test]
    async fn a_role_is_made_in_keycloak_too_and_grants_only_what_is_declared() {
        let harness = Harness::new().await;

        let editor = harness
            .service
            .create_role(new_role("Editor", false, &["USERS_VIEW"]))
            .await
            .unwrap();

        assert_eq!(editor.name.as_str(), "editor");
        assert_eq!(harness.realm.calls().last().unwrap(), "create EDITOR");
        assert!(matches!(
            harness.service.create_role(new_role("editor", false, &[])).await,
            Err(IdentityError::RoleAlreadyExists)
        ));
        assert!(matches!(
            harness
                .service
                .create_role(new_role("other", false, &["NOWHERE"]))
                .await,
            Err(IdentityError::UnknownPermission(_))
        ));
        assert!(matches!(
            harness.service.create_role(new_role("two words", false, &[])).await,
            Err(IdentityError::InvalidRoleName)
        ));
    }

    #[tokio::test]
    async fn a_role_keycloak_refuses_is_not_stored() {
        let harness = Harness::new().await;
        harness.realm.fail();

        let result = harness.service.create_role(new_role("editor", false, &[])).await;

        assert!(matches!(result, Err(IdentityError::IdentityProviderUnavailable(_))));
        assert!(matches!(
            harness.service.get_role("editor").await,
            Err(IdentityError::RoleNotFound)
        ));
    }

    #[tokio::test]
    async fn the_last_unrestricted_role_stays_unrestricted() {
        let harness = Harness::new().await;
        assert!(matches!(
            harness.service.update_role("admin", changes(false, &[])).await,
            Err(IdentityError::LastUnrestrictedRole)
        ));

        harness.service.create_role(new_role("owner", true, &[])).await.unwrap();
        harness.service.update_role("owner", changes(false, &[])).await.unwrap();
    }

    #[tokio::test]
    async fn system_unrestricted_and_held_roles_stay() {
        let harness = Harness::new().await;
        harness.service.create_role(new_role("owner", true, &[])).await.unwrap();
        harness
            .service
            .create_role(new_role("editor", false, &[]))
            .await
            .unwrap();
        harness
            .verifier
            .issue(identity(vec![RoleName::parse("editor").unwrap()]));
        harness.service.sign_in("code", "verifier").await.unwrap();

        for (name, refusal) in [
            ("admin", "system"),
            ("user", "system"),
            ("owner", "unrestricted"),
            ("editor", "assigned"),
        ] {
            let result = harness.service.delete_role(name).await;
            let refused = match refusal {
                "system" => matches!(result, Err(IdentityError::SystemRole)),
                "unrestricted" => matches!(result, Err(IdentityError::UnrestrictedRole)),
                _ => matches!(result, Err(IdentityError::RoleStillAssigned)),
            };
            assert!(refused, "{name}: {result:?}");
        }
        assert!(!harness.realm.calls().iter().any(|call| call.starts_with("delete")));
    }

    #[tokio::test]
    async fn a_role_is_removed_here_even_when_keycloak_cannot_be_reached() {
        let harness = Harness::new().await;
        harness
            .service
            .create_role(new_role("editor", false, &[]))
            .await
            .unwrap();
        harness.realm.fail();

        harness.service.delete_role("editor").await.unwrap();

        let left = harness
            .roles
            .find_many(&[RoleName::parse("editor").unwrap()])
            .await
            .unwrap();
        assert_eq!(left, []);
    }

    #[tokio::test]
    async fn roles_are_given_and_taken_in_keycloak() {
        let harness = Harness::new().await;
        harness.service.sign_in("code", "verifier").await.unwrap();
        let user = harness.users.list().await.unwrap()[0].clone();

        harness.service.assign_role(user.id, "admin").await.unwrap();
        harness.service.remove_role(user.id, "user").await.unwrap();

        let account = user.keycloak_id.to_string();
        assert_eq!(
            harness.realm.calls()[2..],
            [format!("assign {account} ADMIN"), format!("remove {account} USER")]
        );
        assert!(matches!(
            harness.service.assign_role(user.id, "nobody").await,
            Err(IdentityError::RoleNotFound)
        ));
    }

    #[tokio::test]
    async fn the_last_holder_of_an_unrestricted_role_keeps_it() {
        let harness = Harness::new().await;
        harness
            .verifier
            .issue(identity(vec![RoleName::parse("admin").unwrap()]));
        harness.service.sign_in("code", "verifier").await.unwrap();
        let admin = harness.users.list().await.unwrap()[0].clone();

        assert!(matches!(
            harness.service.remove_role(admin.id, "admin").await,
            Err(IdentityError::LastUnrestrictedHolder)
        ));
    }
}
