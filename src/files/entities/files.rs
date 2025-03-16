use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "files")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub filename: String,
	pub original_name: String,
	pub path: String,
	pub mime_type: String,
	pub encoding: String,
	pub size: i32,
	pub storage_type: String,
	pub url: String,
	#[sea_orm(column_type = "Json")]
	pub metadata: Value,
	pub created_at: DateTimeWithTimeZone,
	pub updated_at: Option<DateTimeWithTimeZone>,
	pub deleted_at: Option<DateTimeWithTimeZone>,
	pub deleted_by_user_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "crate::users::entities::users::Entity",
		from = "Column::DeletedByUserId",
		to = "crate::users::entities::users::Column::Id"
	)]
	User,
}

impl Related<crate::users::entities::users::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
