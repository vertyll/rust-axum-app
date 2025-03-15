use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_roles")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub user_id: i32,
	pub role_id: i32,
	pub created_at: DateTimeWithTimeZone,
	pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "crate::users::entities::user::Entity",
		from = "Column::UserId",
		to = "crate::users::entities::user::Column::Id"
	)]
	User,
	#[sea_orm(
		belongs_to = "super::role::Entity",
		from = "Column::RoleId",
		to = "super::role::Column::Id"
	)]
	Role,
}

impl Related<crate::users::entities::user::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl Related<super::role::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Role.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
