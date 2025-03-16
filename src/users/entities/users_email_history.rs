use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users_email_history")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub old_email: String,
	pub new_email: String,
	pub email_change_at: DateTimeWithTimeZone,
	pub created_at: DateTimeWithTimeZone,
	pub updated_at: Option<DateTimeWithTimeZone>,
	pub user_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "crate::users::entities::users::Entity",
		from = "Column::UserId",
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
