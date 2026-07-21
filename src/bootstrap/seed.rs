//! Database seeding. Runs at startup and is idempotent: if any role exists,
//! it does nothing.

use crate::identity::domain::RoleName;
use crate::identity::infrastructure::persistence::records::RoleRecord;

pub async fn seed_roles(db: &toasty::Db) -> anyhow::Result<()> {
	let mut db = db.clone();

	let existing = RoleRecord::all().first().exec(&mut db).await?;
	if existing.is_some() {
		tracing::debug!("roles already seeded, skipping");
		return Ok(());
	}

	tracing::info!("seeding roles");
	let mut tx = db.transaction().await?;

	for role in RoleName::ALL {
		toasty::create!(RoleRecord {
			name: role.as_str(),
			description: role.description(),
		})
		.exec(&mut tx)
		.await?;
		tracing::info!("created role: {role}");
	}

	tx.commit().await?;
	Ok(())
}
