//! What the files module checks. Uploading, listing and reading files needs
//! only a signed-in caller.

use crate::identity::{Catalogue, Declared, Permission};

pub struct FilesManage;

impl Permission for FilesManage {
    const NAME: &'static str = "FILES_MANAGE";
    const DESCRIPTION: &'static str = "Change any file's details, and delete files";
}

pub const CATALOGUE: Catalogue = Catalogue {
    module: "files",
    permissions: &[Declared::of::<FilesManage>()],
    stock_roles: &[],
};
