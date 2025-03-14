use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub username: String,
	pub email: String,
	#[sea_orm(column_name = "password_hash")]
	#[serde(skip_serializing)]
	pub password_hash: String,
	#[sea_orm(column_type = "Timestamp", default_value = "CURRENT_TIMESTAMP")]
	pub created_at: chrono::DateTime<Utc>,
	#[sea_orm(column_type = "Timestamp", default_value = "CURRENT_TIMESTAMP")]
	pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
