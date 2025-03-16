use crate::i18n::setup::translate;
use std::borrow::Cow;
use validator::ValidationError;

pub fn validate_storage_type(storage_type: &str) -> Result<(), ValidationError> {
	match storage_type {
		"local" => Ok(()),
		_ => {
			let mut err = ValidationError::new("invalid_storage_type");
			err.message = Some(Cow::Owned(translate("files.validators.storage_type.invalid")));
			Err(err)
		}
	}
}

pub fn validate_mime_type(mime_type: &str) -> Result<(), ValidationError> {
	if !mime_type.contains('/') {
		let mut err = ValidationError::new("invalid_mime_type");
		err.message = Some(Cow::Owned(translate("files.validators.mime_type.invalid_format")));
		return Err(err);
	}
	Ok(())
}

pub fn validate_file_size(size: i32) -> Result<(), ValidationError> {
	if size <= 0 {
		let mut err = ValidationError::new("invalid_size");
		err.message = Some(Cow::Owned(translate("files.validators.size.must_be_positive")));
		return Err(err);
	}
	Ok(())
}
