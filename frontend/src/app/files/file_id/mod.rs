mod delete;
pub(crate) mod download;
mod soft_delete;

use serde::Deserialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::Form;
use topcoat::router::{href, module_param, page, path_param, route};
use topcoat::view::{View, attributes, component, view};

use super::api::{self, FileChanges, StoredFile};
use crate::auth::{Permission, require_permission};
use crate::backend::BackendError;
use crate::components::back_link::back_link;
use crate::components::button::{ButtonSize, ButtonVariant, button, button_variants};
use crate::components::card::{card, card_content, card_description, card_footer, card_header, card_title};
use crate::components::confirm::confirm_action;
use crate::components::feedback::problem_alert;
use crate::components::field::field_group;
use crate::components::icon::{Icon, IconSize, icon};
use crate::components::page_header::page_header;
use crate::components::text_field::text_field;
use crate::forms::{done, refused_with, typed_or};
use crate::i18n::{Texts, texts};

module_param!(pub file_id: i64, error = not_found);

#[page]
pub(crate) async fn file(cx: &Cx) -> Result<impl View> {
    let current = require_permission(cx, Permission::FilesView).await?;
    let texts = texts(cx).await?;
    let id = *path_param::<FileId>(cx)?;
    let stored = api::get(cx, id).await.map_err(BackendError::into_page_error)?;
    let may_manage = current.may(Permission::FilesManage);

    Ok(view! {
        page_header(
            title: stored.original_name.clone(),
            description: Some(
                texts.t_with(
                    "files.detail.uploaded",
                    vec![("at", texts.datetime(stored.created_at).into())],
                ),
            ),
            <a
                href=(href!(download::download_file, FileId(id)))
                class=(button_variants(ButtonVariant::Primary, ButtonSize::Md))
            >
                icon(name: Icon::ArrowDownTray, size: IconSize::Sm)
                (texts.t("files.detail.download"))
            </a>
            back_link(
                href: href!(super::files).resolve(cx),
                label: texts.t("files.detail.back")
            )
        )
        problem_alert()
        <div class="grid items-start gap-6 md:grid-cols-2">
            file_details(texts: texts, stored: &stored)
            if may_manage {
                file_changes(texts: texts, stored: &stored)
            }
        </div>
    })
}

#[component]
async fn file_details(texts: &Texts, stored: &StoredFile) -> Result<impl View> {
    let rows = [
        ("files.fields.type", stored.mime_type.clone()),
        ("files.fields.encoding", stored.encoding.clone()),
        (
            "files.fields.size",
            texts.t_with("common.bytes", vec![("size", stored.size.into())]),
        ),
        ("files.fields.storage", stored.storage_type.clone()),
        ("files.fields.updated", texts.datetime(stored.updated_at)),
    ];
    Ok(view! {
        card(
            card_header(card_title(<h2>(texts.t("files.detail.title"))</h2>))
            card_content(
                <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
                    for (key, value) in rows {
                        <dt class="text-muted-foreground">(texts.t(key))</dt>
                        <dd class="break-all">(value)</dd>
                    }
                </dl>
            )
        )
    })
}

#[component]
async fn file_changes(cx: &Cx, texts: &Texts, stored: &StoredFile) -> Result<impl View> {
    let id = stored.id;
    let original_name = typed_or(cx, "original_name", &stored.original_name);
    let mime_type = typed_or(cx, "mime_type", &stored.mime_type);
    let encoding = typed_or(cx, "encoding", &stored.encoding);

    Ok(view! {
        card(
            card_header(
                card_title(<h2>(texts.t("files.edit.title"))</h2>)
                card_description((texts.t("files.edit.description")))
            )
            card_content(
                <form
                    id="file-details"
                    method="post"
                    action=(href!(update_file, FileId(id)))
                >
                    field_group(
                        text_field(
                            name: "original_name",
                            label: texts.t("files.fields.name"),
                            value: original_name,
                            attrs: attributes! { required="" }
                        )
                        text_field(
                            name: "mime_type",
                            label: texts.t("files.fields.type"),
                            value: mime_type,
                            attrs: attributes! { required="" }
                        )
                        text_field(
                            name: "encoding",
                            label: texts.t("files.fields.encoding"),
                            value: encoding,
                            attrs: attributes! { required="" }
                        )
                    )
                </form>
            )
            card_footer(
                attrs: attributes! { class="flex-wrap" },
                button(
                    attrs: attributes! { type="submit" form="file-details" },
                    icon(name: Icon::Check, size: IconSize::Sm)
                    (texts.t("common.save"))
                )
                <form
                    method="post"
                    action=(href!(soft_delete::soft_delete_file, FileId(id)))
                >
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { type="submit" },
                        icon(name: Icon::Trash, size: IconSize::Sm)
                        (texts.t("files.edit.soft_delete"))
                    )
                </form>
                confirm_action(
                    action: href!(delete::delete_file, FileId(id)).resolve(cx),
                    label: texts.t("files.edit.delete"),
                    question: texts.t_with(
                        "files.edit.delete_question",
                        vec![("name", stored.original_name.clone().into())],
                    )
                )
            )
        )
    })
}

#[derive(Deserialize)]
struct FileDetails {
    original_name: String,
    mime_type: String,
    encoding: String,
}

#[route(POST)]
pub(crate) async fn update_file(cx: &Cx, Form(details): Form<FileDetails>) -> Result<()> {
    require_permission(cx, Permission::FilesManage).await?;
    let id = *path_param::<FileId>(cx)?;
    let changes = FileChanges {
        original_name: details.original_name.trim(),
        mime_type: details.mime_type.trim(),
        encoding: details.encoding.trim(),
    };
    match api::update(cx, id, &changes).await {
        Ok(_) => done(href!(file, FileId(id)).resolve(cx)),
        Err(err) => Err(refused_with(
            cx,
            href!(file, FileId(id)).resolve(cx),
            err,
            [
                ("original_name", details.original_name),
                ("mime_type", details.mime_type),
                ("encoding", details.encoding),
            ],
        )),
    }
}
