pub(crate) mod roles;

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, module_param, page, path_param};
use topcoat::view::{View, attributes, view};

use super::api;
use crate::app::admin::roles::api as roles_api;
use crate::auth::{Permission, require_permission};
use crate::backend::{BackendError, PageRequest};
use crate::components::back_link::back_link;
use crate::components::button::button;
use crate::components::card::{card, card_content, card_description, card_header, card_title};
use crate::components::feedback::problem_alert;
use crate::components::field::{field, field_label};
use crate::components::icon::{Icon, IconSize, icon};
use crate::components::page_header::page_header;
use crate::components::pagination::MAX_PAGE_SIZE;
use crate::components::row_actions::icon_submit;
use crate::components::select::select;
use crate::i18n::texts;

module_param!(pub user_id: i64, error = not_found);

#[page]
pub(crate) async fn user(cx: &Cx) -> Result<impl View> {
    let current = require_permission(cx, Permission::UsersView).await?;
    let texts = texts(cx).await?;
    let id = *path_param::<UserId>(cx)?;
    let account = api::user(cx, id).await.map_err(BackendError::into_page_error)?;
    let may_manage = current.may(Permission::UsersManage);
    // Offering a role to give needs the list of roles, which `ROLES_VIEW` reads.
    let assignable: Vec<String> = if may_manage && current.may(Permission::RolesView) {
        roles_api::roles(
            cx,
            PageRequest {
                page: 0,
                size: MAX_PAGE_SIZE,
            },
        )
        .await
        .map_err(BackendError::into_page_error)?
        .content
        .into_iter()
        .map(|role| role.name)
        .filter(|name| !account.roles.contains(name))
        .collect()
    } else {
        Vec::new()
    };

    Ok(view! {
        page_header(
            title: account.display_name(),
            description: Some(account.email.clone()),
            back_link(
                href: href!(super::users).resolve(cx),
                label: texts.t("admin.users.back")
            )
        )
        problem_alert()
        card(
            card_header(
                card_title(<h2>(texts.t("admin.users.roles.title"))</h2>)
                card_description((texts.t("admin.users.roles.description")))
            )
            card_content(
                <ul class="mb-6 flex flex-wrap gap-2">
                    #[key(role.clone())]
                    for role in account.roles.iter() {
                        <li
                            id=(format!("role-{role}"))
                            class="inline-flex items-center gap-1 rounded-md border border-border bg-background py-0.5 pr-0.5 pl-2.5 text-sm"
                        >
                            <span class="font-medium">(role.as_str())</span>
                            if may_manage {
                                icon_submit(
                                    action: href!(roles::remove::remove_role, UserId(id)).resolve(
                                        cx,
                                    ),
                                    name: Icon::XMark,
                                    label: texts.t_with(
                                        "admin.users.roles.remove",
                                        vec![("role", role.as_str().into())],
                                    ),
                                    <input type="hidden" name="role" value=(role.as_str())>
                                )
                            }
                        </li>
                    }
                </ul>
                if !assignable.is_empty() {
                    <form
                        method="post"
                        action=(href!(roles::assign_role, UserId(id)))
                        class="flex flex-col gap-3 sm:flex-row sm:items-end"
                    >
                        field(
                            attrs: attributes! { class="flex-1" },
                            field_label(
                                attrs: attributes! { for="role" },
                                (texts.t("admin.users.roles.choose"))
                            )
                            select(
                                attrs: attributes! { id="role" name="role" required="" },
                                for name in assignable.iter() {
                                    <option value=(name.as_str())>(name.as_str())</option>
                                }
                            )
                        )
                        button(
                            attrs: attributes! { type="submit" },
                            icon(name: Icon::ShieldCheck, size: IconSize::Sm)
                            (texts.t("admin.users.roles.assign"))
                        )
                    </form>
                }
            )
        )
    })
}
