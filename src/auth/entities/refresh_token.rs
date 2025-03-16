use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "refresh_tokens")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub token: String,
	pub expires_at: DateTimeWithTimeZone,
	pub created_at: DateTimeWithTimeZone,
	pub updated_at: DateTimeWithTimeZone,
	pub user_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "crate::users::entities::user::Entity",
		from = "Column::UserId",
		to = "crate::users::entities::user::Column::Id"
	)]
	User,
}

impl Related<crate::users::entities::user::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
