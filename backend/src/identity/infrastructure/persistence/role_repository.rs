use std::collections::{BTreeSet, HashMap};

use super::records::{PermissionRecord, RolePermissionRecord, RoleRecord, UserRoleRecord};
use super::user_repository::persistence;
use crate::identity::domain::{IdentityError, RegisteredPermission, Role, RoleName, RoleRepository};

/// Roles and the registry of permissions in PostgreSQL. There are a handful of
/// roles, so a read loads them whole rather than joining per name.
#[derive(Clone)]
pub struct ToastyRoleRepository {
    db: toasty::Db,
}

impl ToastyRoleRepository {
    #[must_use]
    pub fn new(db: toasty::Db) -> Self {
        Self { db }
    }

    async fn load(&self) -> Result<Vec<Role>, IdentityError> {
        let mut db = self.db.clone();
        let roles = RoleRecord::all().exec(&mut db).await.map_err(persistence)?;
        let grants = RolePermissionRecord::all().exec(&mut db).await.map_err(persistence)?;
        let permissions = PermissionRecord::all().exec(&mut db).await.map_err(persistence)?;

        let names: HashMap<i64, &str> = permissions
            .iter()
            .map(|permission| (permission.id, permission.name.as_str()))
            .collect();
        let mut granted: HashMap<i64, BTreeSet<String>> = HashMap::new();
        for grant in &grants {
            if let Some(name) = names.get(&grant.permission_id) {
                granted.entry(grant.role_id).or_default().insert((*name).to_owned());
            }
        }
        roles
            .into_iter()
            .map(|record| {
                let permissions = granted.remove(&record.id).unwrap_or_default();
                to_domain(record, permissions)
            })
            .collect()
    }
}

fn to_domain(record: RoleRecord, permissions: BTreeSet<String>) -> Result<Role, IdentityError> {
    Ok(Role {
        name: RoleName::parse(&record.name)
            .map_err(|_| IdentityError::CorruptData(format!("role {}: invalid name", record.id)))?,
        description: record.description.unwrap_or_default(),
        unrestricted: record.unrestricted,
        permissions,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

impl RoleRepository for ToastyRoleRepository {
    async fn list(&self) -> Result<Vec<Role>, IdentityError> {
        self.load().await
    }

    async fn find(&self, name: &RoleName) -> Result<Option<Role>, IdentityError> {
        Ok(self.load().await?.into_iter().find(|role| &role.name == name))
    }

    async fn find_many(&self, names: &[RoleName]) -> Result<Vec<Role>, IdentityError> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self
            .load()
            .await?
            .into_iter()
            .filter(|role| names.contains(&role.name))
            .collect())
    }

    async fn create(&self, role: &Role) -> Result<(), IdentityError> {
        let mut db = self.db.clone();
        let mut tx = db.transaction().await.map_err(persistence)?;

        let record = toasty::create!(RoleRecord {
            name: role.name.as_str(),
            description: role.description.as_str(),
            unrestricted: role.unrestricted,
        })
        .exec(&mut tx)
        .await
        .map_err(persistence)?;
        grant(&mut tx, record.id, &role.permissions).await?;

        tx.commit().await.map_err(persistence)
    }

    async fn update(&self, role: &Role) -> Result<(), IdentityError> {
        let mut db = self.db.clone();
        let mut tx = db.transaction().await.map_err(persistence)?;

        let record = RoleRecord::filter_by_name(role.name.as_str())
            .first()
            .exec(&mut tx)
            .await
            .map_err(persistence)?
            .ok_or(IdentityError::RoleNotFound)?;
        toasty::update!(RoleRecord::filter_by_id(record.id) {
            description: Some(role.description.clone()),
            unrestricted: role.unrestricted,
        })
        .exec(&mut tx)
        .await
        .map_err(persistence)?;
        RolePermissionRecord::filter_by_role_id(record.id)
            .delete()
            .exec(&mut tx)
            .await
            .map_err(persistence)?;
        grant(&mut tx, record.id, &role.permissions).await?;

        tx.commit().await.map_err(persistence)
    }

    async fn delete(&self, name: &RoleName) -> Result<(), IdentityError> {
        let mut db = self.db.clone();
        let mut tx = db.transaction().await.map_err(persistence)?;

        if let Some(record) = RoleRecord::filter_by_name(name.as_str())
            .first()
            .exec(&mut tx)
            .await
            .map_err(persistence)?
        {
            UserRoleRecord::filter_by_role_id(record.id)
                .delete()
                .exec(&mut tx)
                .await
                .map_err(persistence)?;
            RolePermissionRecord::filter_by_role_id(record.id)
                .delete()
                .exec(&mut tx)
                .await
                .map_err(persistence)?;
            RoleRecord::filter_by_id(record.id)
                .delete()
                .exec(&mut tx)
                .await
                .map_err(persistence)?;
        }

        tx.commit().await.map_err(persistence)
    }

    async fn permissions(&self) -> Result<Vec<RegisteredPermission>, IdentityError> {
        let mut db = self.db.clone();
        let records = PermissionRecord::all().exec(&mut db).await.map_err(persistence)?;
        let mut permissions: Vec<RegisteredPermission> = records
            .into_iter()
            .map(|record| RegisteredPermission {
                module: record.module,
                name: record.name,
                description: record.description,
            })
            .collect();
        permissions.sort();
        Ok(permissions)
    }

    async fn register(&self, declared: &[RegisteredPermission]) -> Result<(), IdentityError> {
        let mut db = self.db.clone();
        let mut tx = db.transaction().await.map_err(persistence)?;

        let stored = PermissionRecord::all().exec(&mut tx).await.map_err(persistence)?;
        for permission in declared {
            match stored.iter().find(|record| record.name == permission.name) {
                Some(record) if record.module == permission.module && record.description == permission.description => {}
                Some(record) => {
                    toasty::update!(PermissionRecord::filter_by_id(record.id) {
                        module: permission.module.clone(),
                        description: permission.description.clone(),
                    })
                    .exec(&mut tx)
                    .await
                    .map_err(persistence)?;
                }
                None => {
                    toasty::create!(PermissionRecord {
                        name: permission.name.as_str(),
                        module: permission.module.as_str(),
                        description: permission.description.as_str(),
                    })
                    .exec(&mut tx)
                    .await
                    .map_err(persistence)?;
                }
            }
        }
        for withdrawn in stored
            .iter()
            .filter(|record| declared.iter().all(|permission| permission.name != record.name))
        {
            RolePermissionRecord::filter_by_permission_id(withdrawn.id)
                .delete()
                .exec(&mut tx)
                .await
                .map_err(persistence)?;
            PermissionRecord::filter_by_id(withdrawn.id)
                .delete()
                .exec(&mut tx)
                .await
                .map_err(persistence)?;
            tracing::info!(permission = %withdrawn.name, "permission withdrawn: no module declares it any more");
        }

        tx.commit().await.map_err(persistence)
    }
}

/// Links the role to each of `permissions`; the service has checked they are registered.
async fn grant<E>(executor: &mut E, role_id: i64, permissions: &BTreeSet<String>) -> Result<(), IdentityError>
where
    E: toasty::Executor,
{
    for name in permissions {
        let permission = PermissionRecord::filter_by_name(name.as_str())
            .first()
            .exec(executor)
            .await
            .map_err(persistence)?
            .ok_or_else(|| IdentityError::UnknownPermission(name.clone()))?;
        toasty::create!(RolePermissionRecord {
            role_id: role_id,
            permission_id: permission.id,
        })
        .exec(executor)
        .await
        .map_err(persistence)?;
    }
    Ok(())
}
