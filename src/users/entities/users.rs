use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	#[sea_orm(unique)]
	pub username: String,
	#[sea_orm(unique)]
	pub email: String,
	#[serde(skip_serializing)]
	pub password_hash: String,
	pub is_email_confirmed: bool,
	#[serde(skip_serializing)]
	pub email_confirmation_token: Option<String>,
	#[serde(skip_serializing)]
	pub email_confirmation_token_expiry: Option<DateTimeWithTimeZone>,
	#[serde(skip_serializing)]
	pub email_change_token: Option<String>,
	#[serde(skip_serializing)]
	pub email_change_token_expiry: Option<DateTimeWithTimeZone>,
	#[serde(skip_serializing)]
	pub password_reset_token: Option<String>,
	#[serde(skip_serializing)]
	pub password_reset_token_expiry: Option<DateTimeWithTimeZone>,
	#[serde(skip_serializing)]
	pub pending_email: Option<String>,
	pub created_at: DateTimeWithTimeZone,
	pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
