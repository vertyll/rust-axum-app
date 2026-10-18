//! What the files module checks: reading files, and changing them.

use crate::identity::contracts::{Catalogue, Declared, Permission};

pub struct FilesView;

impl Permission for FilesView {
    const NAME: &'static str = "FILES_VIEW";
    const DESCRIPTION: &'static str = "See and download every file";
}

pub struct FilesManage;

impl Permission for FilesManage {
    const NAME: &'static str = "FILES_MANAGE";
    const DESCRIPTION: &'static str = "Upload files, change their details, and delete them";
}

pub const CATALOGUE: Catalogue = Catalogue {
    module: "files",
    permissions: &[Declared::of::<FilesView>(), Declared::of::<FilesManage>()],
    stock_roles: &[],
};
