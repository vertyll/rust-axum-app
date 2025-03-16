use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::roles::entities::roles;
use crate::roles::entities::roles::Entity as Role;

pub async fn seed_roles(db: &DatabaseConnection) -> Result<(), AppError> {
	println!("Seeding roles...");

	let existing_roles = Role::find().all(db).await.map_err(|_| AppError::InternalError)?;

	if !existing_roles.is_empty() {
		println!("Roles already seeded, skipping...");
		return Ok(());
	}

	let txn = db.begin().await.map_err(|_| AppError::InternalError)?;

	let roles = [
		(RoleEnum::Admin, "Administrator with full access"),
		(RoleEnum::User, "Regular user with limited access"),
		(RoleEnum::Manager, "User with management privileges"),
	];

	let now: DateTime<Utc> = Utc::now();

	for (role_type, description) in roles {
		let role_model = roles::ActiveModel {
			id: Default::default(),
			name: Set(role_type.as_str().to_string()),
			description: Set(Some(description.to_string())),
			created_at: Set(now.into()),
			updated_at: Set(now.into()),
		};

		role_model.save(&txn).await.map_err(|_| AppError::InternalError)?;

		println!("Created role: {}", role_type.as_str());
	}

	txn.commit().await.map_err(|_| AppError::InternalError)?;
	println!("Roles seeded successfully.");

	Ok(())
}
