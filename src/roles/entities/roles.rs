use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "roles")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	#[sea_orm(unique)]
	pub name: String,
	pub description: Option<String>,
	pub created_at: DateTimeWithTimeZone,
	pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::user_roles::Entity")]
	UserRole,
}

impl Related<super::user_roles::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::UserRole.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
