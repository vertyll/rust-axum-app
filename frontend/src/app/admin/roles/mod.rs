pub(crate) mod api;
pub(crate) mod role_name;

use serde::Deserialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::Form;
use topcoat::router::{href, page, route};
use topcoat::runtime::link;
use topcoat::runtime::{Signal, shard, signal};
use topcoat::view::{View, attributes, component, view};

use self::api::NewRole;
use crate::auth::{Permission, require_permission};
use crate::backend::BackendError;
use crate::components::badge::{BadgeVariant, badge};
use crate::components::button::button;
use crate::components::card::{card, card_content, card_header, card_title};
use crate::components::data_table::data_table;
use crate::components::feedback::problem_alert;
use crate::components::field::field_group;
use crate::components::icon::{Icon, IconSize, icon};
use crate::components::page_header::page_header;
use crate::components::pagination::{DEFAULT_PAGE_SIZE, PageInfo, load_page, page_request, pagination};
use crate::components::row_actions::{actions_cell, actions_head, icon_link};
use crate::components::table::{table_body, table_cell, table_head, table_header, table_row};
use crate::components::text_field::text_field;
use crate::forms::{done, refused_with, typed_or};
use crate::i18n::{Texts, texts};

#[page]
pub(crate) async fn roles(cx: &Cx) -> Result<impl View> {
    let current = require_permission(cx, Permission::RolesView).await?;
    let texts = texts(cx).await?;
    let page = signal(cx, || 1_usize);
    let page_size = signal(cx, || DEFAULT_PAGE_SIZE);

    Ok(view! {
        page_header(
            title: texts.t("admin.roles.title"),
            description: Some(texts.t("admin.roles.description"))
        )
        problem_alert()
        role_list(page: page, page_size: page_size)
        if current.may(Permission::RolesManage) {
            new_role(texts: texts)
        }
    })
}

/// One page of the roles; an endpoint of its own, so it checks the reader's
/// permission itself.
#[shard]
pub(crate) async fn role_list(cx: &Cx, page: Signal<usize>, page_size: Signal<usize>) -> Result<impl View> {
    require_permission(cx, Permission::RolesView).await?;
    let texts = texts(cx).await?;
    let request = page_request(page.get(), page_size.get());
    let paged = load_page(request, |request| api::roles(cx, request))
        .await
        .map_err(BackendError::into_page_error)?;
    Ok(view! {
        data_table(
            table_header(
                table_row(
                    table_head((texts.t("admin.roles.fields.name")))
                    table_head((texts.t("admin.roles.fields.description")))
                    table_head((texts.t("admin.roles.fields.grants")))
                    actions_head(label: texts.t("common.actions"))
                )
            )
            table_body(
                #[key(role.name.clone())]
                for role in paged.content.clone() {
                    let count = i64::try_from(role.permissions.len()).expect(
                        "a role grants a handful of permissions",
                    );
                    let page = href!(
                        role_name::role,
                        role_name::RoleName(role.name.clone()),
                    ).resolve(cx);
                    let name_link = page.clone();
                    table_row(
                        attrs: attributes! { id=(format!("role-{}", role.name)) },
                        table_cell(
                            link(
                                href: name_link,
                                attrs: attributes! { class="font-medium underline-offset-4 hover:underline" },
                                (role.name.as_str())
                            )
                        )
                        table_cell((role.description.as_str()))
                        table_cell(
                            if role.unrestricted {
                                badge((texts.t("admin.roles.unrestricted")))
                            } else {
                                badge(
                                    variant: BadgeVariant::Outline,
                                    (texts.t_with(
                                        "admin.roles.permission_count",
                                        vec![("count", count.into())],
                                    ))
                                )
                            }
                        )
                        actions_cell(
                            icon_link(
                                href: page,
                                name: Icon::Pencil,
                                label: texts.t("common.edit")
                            )
                        )
                    )
                }
            )
        )
        pagination(page: &page, page_size: &page_size, shown: PageInfo::from(&paged))
    })
}

#[component]
async fn new_role(cx: &Cx, texts: &Texts) -> Result<impl View> {
    let name = typed_or(cx, "name", "");
    let description = typed_or(cx, "description", "");
    Ok(view! {
        card(
            attrs: attributes! { class="mt-8" },
            card_header(card_title(<h2>(texts.t("admin.roles.create.title"))</h2>))
            card_content(
                <form
                    method="post"
                    action=(href!(create_role))
                    class="flex flex-col gap-4"
                >
                    field_group(
                        text_field(
                            name: "name",
                            label: texts.t("admin.roles.fields.name"),
                            value: name,
                            attrs: attributes! { required="" autocomplete="off" }
                        )
                        text_field(
                            name: "description",
                            label: texts.t("admin.roles.fields.description"),
                            value: description,
                            attrs: attributes! { maxlength="255" }
                        )
                    )
                    <div>
                        button(
                            attrs: attributes! { type="submit" },
                            icon(name: Icon::Plus, size: IconSize::Sm)
                            (texts.t("admin.roles.create.submit"))
                        )
                    </div>
                </form>
            )
        )
    })
}

#[derive(Deserialize)]
struct NewRoleForm {
    name: String,
    description: String,
}

#[route(POST)]
pub(crate) async fn create_role(cx: &Cx, Form(form): Form<NewRoleForm>) -> Result<()> {
    require_permission(cx, Permission::RolesManage).await?;
    let role = NewRole {
        name: form.name.trim(),
        description: form.description.trim(),
    };
    match api::create_role(cx, &role).await {
        Ok(created) => done(href!(role_name::role, role_name::RoleName(created.name)).resolve(cx)),
        Err(err) => Err(refused_with(
            cx,
            href!(roles).resolve(cx),
            err,
            [("name", form.name), ("description", form.description)],
        )),
    }
}
