mod delete;

use std::collections::BTreeSet;

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::RawForm;
use topcoat::router::{href, module_param, page, path_param, route};
use topcoat::view::{View, attributes, component, view};

use super::api::{self, Module, Role, RoleChanges};
use crate::auth::{Permission, require_permission};
use crate::backend::BackendError;
use crate::components::back_link::back_link;
use crate::components::button::button;
use crate::components::card::{card, card_content, card_description, card_footer, card_header, card_title};
use crate::components::checkbox::checkbox;
use crate::components::confirm::confirm_action;
use crate::components::feedback::problem_alert;
use crate::components::field::{field_description, field_legend, field_set};
use crate::components::icon::{Icon, IconSize, icon};
use crate::components::label::label;
use crate::components::page_header::page_header;
use crate::components::text_field::text_field;
use crate::forms::{done, form_failure, refused_with, typed_or};
use crate::i18n::{Texts, texts};

module_param!(pub role_name);

const PERMISSION_FIELD: &str = "permission:";

#[page]
pub(crate) async fn role(cx: &Cx) -> Result<impl View> {
    let current = require_permission(cx, Permission::RolesView).await?;
    let texts = texts(cx).await?;
    let name = path_param::<RoleName>(cx).to_owned();
    let (found, modules) = tokio::join!(api::role(cx, &name), api::permissions(cx));
    let found = found.map_err(BackendError::into_page_error)?;
    let modules = modules.map_err(BackendError::into_page_error)?;

    Ok(view! {
        page_header(
            title: found.name.clone(),
            description: Some(
                texts.t_with(
                    "admin.roles.updated",
                    vec![("at", texts.datetime(found.updated_at).into())],
                ),
            ),
            back_link(
                href: href!(super::roles).resolve(cx),
                label: texts.t("admin.roles.back")
            )
        )
        problem_alert()
        role_form(
            texts: texts,
            found: &found,
            modules: &modules,
            may_manage: current.may(Permission::RolesManage)
        )
    })
}

/// The role's description and grants, which a reader with `ROLES_MANAGE`
/// changes; a refused form shows what the reader chose, not what the role
/// holds.
#[component]
async fn role_form(cx: &Cx, texts: &Texts, found: &Role, modules: &[Module], may_manage: bool) -> Result<impl View> {
    let failure = form_failure(cx);
    let description = typed_or(cx, "description", &found.description);
    let unrestricted = failure.map_or(found.unrestricted, |failure| failure.value("unrestricted").is_some());
    let granted: BTreeSet<String> = failure.map_or_else(
        || found.permissions.clone(),
        |failure| {
            failure
                .values
                .iter()
                .filter(|(field_name, _)| field_name.starts_with(PERMISSION_FIELD))
                .map(|(_, value)| value.clone())
                .collect()
        },
    );

    Ok(view! {
        card(
            card_header(
                card_title(<h2>(texts.t("admin.roles.edit.title"))</h2>)
                card_description((texts.t("admin.roles.edit.description")))
            )
            card_content(
                <form
                    id="role"
                    method="post"
                    action=(href!(update_role, RoleName(found.name.clone())))
                >
                    <fieldset disabled=(!may_manage) class="flex flex-col gap-6">
                        text_field(
                            name: "description",
                            label: texts.t("admin.roles.fields.description"),
                            value: description,
                            attrs: attributes! { maxlength="255" }
                        )
                        <div class="flex items-start gap-3">
                            checkbox(
                                attrs: attributes! {
                                    id="unrestricted"
                                    name="unrestricted"
                                    value="true"
                                    checked=(unrestricted)
                                }
                            )
                            <div class="flex flex-col gap-1">
                                label(
                                    attrs: attributes! { for="unrestricted" },
                                    (texts.t("admin.roles.unrestricted"))
                                )
                                field_description(
                                    (texts.t("admin.roles.edit.unrestricted_hint"))
                                )
                            </div>
                        </div>
                        #[key(module.module.clone())]
                        for module in modules {
                            module_permissions(module: module, granted: &granted)
                        }
                    </fieldset>
                </form>
            )
            if may_manage {
                card_footer(
                    attrs: attributes! { class="flex-wrap justify-between" },
                    button(
                        attrs: attributes! { type="submit" form="role" },
                        icon(name: Icon::Check, size: IconSize::Sm)
                        (texts.t("common.save"))
                    )
                    confirm_action(
                        action: href!(delete::delete_role, RoleName(found.name.clone())).resolve(
                            cx,
                        ),
                        label: texts.t("admin.roles.delete.label"),
                        question: texts.t_with(
                            "admin.roles.delete.question",
                            vec![("role", found.name.clone().into())],
                        )
                    )
                )
            }
        )
    })
}

#[component]
async fn module_permissions(cx: &Cx, module: &Module, granted: &BTreeSet<String>) -> Result<impl View> {
    let texts = texts(cx).await?;
    Ok(view! {
        field_set(
            field_legend(<code>(module.module.as_str())</code>)
            <ul class="flex flex-col gap-3">
                #[key(permission.name.clone())]
                for permission in module.permissions.iter() {
                    let id = format!("permission-{}", permission.name);
                    <li id=(format!("{id}-row")) class="flex items-start gap-3">
                        checkbox(
                            attrs: attributes! {
                                id=(id.clone())
                                name=(format!("{PERMISSION_FIELD}{}", permission.name))
                                value=(permission.name.as_str())
                                checked=(granted.contains(&permission.name))
                            }
                        )
                        <div class="flex flex-col gap-0.5">
                            label(
                                attrs: attributes! { for=(id) },
                                <code>(permission.name.as_str())</code>
                            )
                            if let Some(key) = permission_description_key(
                                &permission.name,
                            ) {
                                <span class="text-sm text-muted-foreground">
                                    (texts.t(key))
                                </span>
                            }
                        </div>
                    </li>
                }
            </ul>
        )
    })
}

/// Sets the role's description, whether it is unrestricted, and the
/// permissions it grants. Each permission's checkbox is named after it
/// (`permission:<NAME>`), so no fixed struct can read the form.
#[route(POST)]
pub(crate) async fn update_role(cx: &Cx, RawForm(body): RawForm) -> Result<()> {
    require_permission(cx, Permission::RolesManage).await?;
    let name = path_param::<RoleName>(cx).to_owned();
    let fields: Vec<(String, String)> = form_urlencoded::parse(&body).into_owned().collect();
    let description = fields
        .iter()
        .find(|(field_name, _)| field_name == "description")
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    let unrestricted = fields.iter().any(|(field_name, _)| field_name == "unrestricted");
    let permissions: BTreeSet<String> = fields
        .iter()
        .filter(|(field_name, _)| field_name.starts_with(PERMISSION_FIELD))
        .map(|(_, value)| value.clone())
        .collect();

    let changes = RoleChanges {
        description: description.trim(),
        unrestricted,
        permissions: &permissions,
    };
    let page = href!(role, RoleName(name.clone())).resolve(cx);
    match api::update_role(cx, &name, &changes).await {
        Ok(_) => done(page),
        Err(err) => Err(refused_with(cx, page, err, fields)),
    }
}

/// What a permission allows, in the reader's language; `None` for one a
/// module declared after these pages were written.
fn permission_description_key(permission: &str) -> Option<&'static str> {
    Some(match permission {
        "USERS_VIEW" => "admin.permissions.users_view",
        "USERS_MANAGE" => "admin.permissions.users_manage",
        "ROLES_VIEW" => "admin.permissions.roles_view",
        "ROLES_MANAGE" => "admin.permissions.roles_manage",
        "FILES_VIEW" => "admin.permissions.files_view",
        "FILES_MANAGE" => "admin.permissions.files_manage",
        "TRANSLATIONS_VIEW" => "admin.permissions.translations_view",
        "TRANSLATIONS_EDIT" => "admin.permissions.translations_edit",
        _ => return None,
    })
}
