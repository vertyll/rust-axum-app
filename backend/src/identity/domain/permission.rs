//! Permissions are declared by the module that checks them, in code: a name
//! nobody checks would be a lie in the list an administrator grants from.

/// Something a module checks before it acts. Each permission is a type, so a
/// handler names the one it needs in its extractor, `Permitted<FilesManage>`,
/// and a typo is a compile error rather than an endpoint nobody may call.
pub trait Permission: Send + Sync + 'static {
    /// The name roles grant it by, in capitals: `FILES_MANAGE`.
    const NAME: &'static str;
    /// What it allows, as the list of permissions shows it.
    const DESCRIPTION: &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Declared {
    pub name: &'static str,
    pub description: &'static str,
}

impl Declared {
    #[must_use]
    pub const fn of<P: Permission>() -> Self {
        Self {
            name: P::NAME,
            description: P::DESCRIPTION,
        }
    }
}

/// A role a module ships with, and what it starts out granting. The role is
/// made only where none of that name exists, so an administrator's changes
/// survive every start; an existing role gains only permissions never seen
/// before.
#[derive(Debug, Clone, Copy)]
pub struct StockRole {
    pub name: &'static str,
    pub description: &'static str,
    pub unrestricted: bool,
    pub grants: &'static [&'static str],
}

/// What one module declares: the permissions it checks and the roles it ships with.
#[derive(Debug, Clone, Copy)]
pub struct Catalogue {
    pub module: &'static str,
    pub permissions: &'static [Declared],
    pub stock_roles: &'static [StockRole],
}

/// A permission as the registry holds it, with the module that declared it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisteredPermission {
    pub module: String,
    pub name: String,
    pub description: String,
}

impl RegisteredPermission {
    #[must_use]
    pub fn declared_in(catalogues: &[Catalogue]) -> Vec<Self> {
        catalogues
            .iter()
            .flat_map(|catalogue| {
                catalogue.permissions.iter().map(|permission| Self {
                    module: catalogue.module.to_owned(),
                    name: permission.name.to_owned(),
                    description: permission.description.to_owned(),
                })
            })
            .collect()
    }
}
