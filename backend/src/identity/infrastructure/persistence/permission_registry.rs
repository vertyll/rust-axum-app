use super::records::PermissionRecord;
use super::user_repository::persistence;
use crate::identity::domain::{IdentityError, PermissionRegistry, RegisteredPermission};

/// The registry of permissions in PostgreSQL, rewritten at every start.
#[derive(Clone)]
pub struct ToastyPermissionRegistry {
    db: toasty::Db,
}

impl ToastyPermissionRegistry {
    #[must_use]
    pub fn new(db: toasty::Db) -> Self {
        Self { db }
    }
}

impl PermissionRegistry for ToastyPermissionRegistry {
    async fn list(&self) -> Result<Vec<RegisteredPermission>, IdentityError> {
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
            PermissionRecord::filter_by_id(withdrawn.id)
                .delete()
                .exec(&mut tx)
                .await
                .map_err(persistence)?;
            tracing::info!(permission = %withdrawn.name, "permission removed: no module declares it any more");
        }

        tx.commit().await.map_err(persistence)
    }
}
