//! What the identity module checks, and the roles it ships with.

use crate::identity::domain::{Catalogue, Declared, Permission, RoleName, StockRole};

pub struct UsersView;

impl Permission for UsersView {
    const NAME: &'static str = "USERS_VIEW";
    const DESCRIPTION: &'static str = "See every account and the roles it holds";
}

pub struct UsersManage;

impl Permission for UsersManage {
    const NAME: &'static str = "USERS_MANAGE";
    const DESCRIPTION: &'static str = "Give accounts roles and take them away";
}

pub struct RolesView;

impl Permission for RolesView {
    const NAME: &'static str = "ROLES_VIEW";
    const DESCRIPTION: &'static str = "See the roles, the permissions and what each role grants";
}

pub struct RolesManage;

impl Permission for RolesManage {
    const NAME: &'static str = "ROLES_MANAGE";
    const DESCRIPTION: &'static str = "Make, change and remove roles";
}

pub const CATALOGUE: Catalogue = Catalogue {
    module: "identity",
    permissions: &[
        Declared::of::<UsersView>(),
        Declared::of::<UsersManage>(),
        Declared::of::<RolesView>(),
        Declared::of::<RolesManage>(),
    ],
    stock_roles: &[
        StockRole {
            name: RoleName::ADMIN,
            description: "Administrator",
            unrestricted: true,
            grants: &[],
        },
        StockRole {
            name: RoleName::USER,
            description: "Signed-in user",
            unrestricted: false,
            grants: &[],
        },
    ],
};
