pub(crate) mod api;
pub(crate) mod user_id;

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, page};
use topcoat::runtime::link;
use topcoat::runtime::{Signal, shard, signal};
use topcoat::view::{View, attributes, view};

use crate::auth::{Permission, require_permission};
use crate::backend::BackendError;
use crate::components::badge::{BadgeVariant, badge};
use crate::components::data_table::data_table;
use crate::components::icon::Icon;
use crate::components::page_header::page_header;
use crate::components::pagination::{DEFAULT_PAGE_SIZE, PageInfo, load_page, page_request, pagination};
use crate::components::row_actions::{actions_cell, actions_head, icon_link};
use crate::components::table::{table_body, table_cell, table_head, table_header, table_row};
use crate::i18n::texts;

#[page]
pub(crate) async fn users(cx: &Cx) -> Result<impl View> {
    require_permission(cx, Permission::UsersView).await?;
    let texts = texts(cx).await?;
    let page = signal(cx, || 1_usize);
    let page_size = signal(cx, || DEFAULT_PAGE_SIZE);

    Ok(view! {
        page_header(
            title: texts.t("admin.users.title"),
            description: Some(texts.t("admin.users.description"))
        )
        user_list(page: page, page_size: page_size)
    })
}

/// One page of the accounts; an endpoint of its own, so it checks the
/// reader's permission itself.
#[shard]
pub(crate) async fn user_list(cx: &Cx, page: Signal<usize>, page_size: Signal<usize>) -> Result<impl View> {
    require_permission(cx, Permission::UsersView).await?;
    let texts = texts(cx).await?;
    let request = page_request(page.get(), page_size.get());
    let paged = load_page(request, |request| api::users(cx, request))
        .await
        .map_err(BackendError::into_page_error)?;

    Ok(view! {
        data_table(
            table_header(
                table_row(
                    table_head((texts.t("admin.users.fields.name")))
                    table_head((texts.t("admin.users.fields.email")))
                    table_head((texts.t("admin.users.fields.roles")))
                    table_head((texts.t("admin.users.fields.since")))
                    actions_head(label: texts.t("common.actions"))
                )
            )
            table_body(
                #[key(account.id)]
                for account in paged.content.clone() {
                    let page = href!(user_id::user, user_id::UserId(account.id)).resolve(
                        cx,
                    );
                    let name_link = page.clone();
                    table_row(
                        attrs: attributes! { id=(format!("user-{}", account.id)) },
                        table_cell(
                            link(
                                href: name_link,
                                attrs: attributes! { class="font-medium underline-offset-4 hover:underline" },
                                (account.display_name())
                            )
                        )
                        table_cell((account.email.as_str()))
                        table_cell(
                            <div class="flex gap-1">
                                for role in account.roles.iter() {
                                    badge(variant: BadgeVariant::Secondary, (role.as_str()))
                                }
                            </div>
                        )
                        table_cell((texts.datetime(account.created_at)))
                        actions_cell(
                            icon_link(
                                href: page,
                                name: Icon::Eye,
                                label: texts.t("common.details")
                            )
                        )
                    )
                }
            )
        )
        pagination(page: &page, page_size: &page_size, shown: PageInfo::from(&paged))
    })
}
