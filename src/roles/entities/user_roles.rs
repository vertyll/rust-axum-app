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
		belongs_to = "crate::users::entities::users::Entity",
		from = "Column::UserId",
		to = "crate::users::entities::users::Column::Id"
	)]
	User,
	#[sea_orm(
		belongs_to = "super::roles::Entity",
		from = "Column::RoleId",
		to = "super::roles::Column::Id"
	)]
	Role,
}

impl Related<crate::users::entities::users::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl Related<super::roles::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Role.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
