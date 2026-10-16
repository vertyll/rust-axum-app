pub(crate) mod api;
pub(crate) mod file_id;

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::multipart::Multipart;
use topcoat::router::{StatusCode, href, page, route};
use topcoat::runtime::{Signal, shard, signal};
use topcoat::view::{View, attributes, view};

use self::api::{StoredFile, Upload};
use crate::auth::{Permission, require_permission};
use crate::backend::{BackendError, Problem};
use crate::components::button::button;
use crate::components::data_table::data_table;
use crate::components::feedback::{field_errors, problem_alert};
use crate::components::field::{field_description, field_label};
use crate::components::icon::{Icon, IconSize, icon};
use crate::components::input::input;
use crate::components::page_header::page_header;
use crate::components::pagination::{DEFAULT_PAGE_SIZE, PageInfo, load_page, page_request, pagination};
use crate::components::row_actions::{actions_cell, actions_head, icon_download, icon_link};
use crate::components::table::{table_body, table_cell, table_head, table_header, table_row};
use crate::forms::{done, refused};
use crate::i18n::{Texts, texts};

#[page]
pub(crate) async fn files(cx: &Cx) -> Result<impl View> {
    let current = require_permission(cx, Permission::FilesView).await?;
    let texts = texts(cx).await?;
    let page = signal(cx, || 1_usize);
    let page_size = signal(cx, || DEFAULT_PAGE_SIZE);

    Ok(view! {
        page_header(
            title: texts.t("files.list.title"),
            description: Some(texts.t("files.list.description"))
        )
        problem_alert()
        if current.may(Permission::FilesManage) {
            upload_form()
        }
        file_list(page: page, page_size: page_size)
    })
}

#[topcoat::view::component]
async fn upload_form(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    Ok(view! {
        <form
            method="post"
            action=(href!(upload_file))
            enctype="multipart/form-data"
            class="mb-6 flex flex-col gap-2 rounded-lg border border-border bg-card p-4"
        >
            field_label(
                attrs: attributes! { for="file" },
                (texts.t("files.upload.file"))
            )
            <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
                input(
                    attrs: attributes! {
                        id="file"
                        name="file"
                        type="file"
                        required=""
                        class="sm:flex-1"
                    }
                )
                button(
                    attrs: attributes! { type="submit" class="shrink-0" },
                    icon(name: Icon::ArrowUpTray, size: IconSize::Sm)
                    (texts.t("files.upload.submit"))
                )
            </div>
            field_description((texts.t("files.upload.hint")))
            field_errors(name: "file")
        </form>
    })
}

/// One page of the files; an endpoint of its own, so it checks the reader's
/// permission itself.
#[shard]
pub(crate) async fn file_list(cx: &Cx, page: Signal<usize>, page_size: Signal<usize>) -> Result<impl View> {
    require_permission(cx, Permission::FilesView).await?;
    let texts = texts(cx).await?;
    let request = page_request(page.get(), page_size.get());
    let paged = load_page(request, |request| api::list(cx, request))
        .await
        .map_err(BackendError::into_page_error)?;
    Ok(view! {
        if paged.total_elements == 0 {
            <p
                class="rounded-lg border border-dashed border-border px-4 py-12 text-center text-sm text-muted-foreground"
            >
                (texts.t("files.list.empty"))
            </p>
        } else {
            files_table(texts: texts, rows: paged.content.clone())
            pagination(
                page: &page,
                page_size: &page_size,
                shown: PageInfo::from(&paged)
            )
        }
    })
}

#[topcoat::view::component]
async fn files_table(cx: &Cx, texts: &Texts, rows: Vec<StoredFile>) -> Result<impl View> {
    Ok(view! {
        data_table(
            table_header(
                table_row(
                    table_head((texts.t("files.fields.name")))
                    table_head((texts.t("files.fields.type")))
                    table_head((texts.t("files.fields.size")))
                    table_head((texts.t("files.fields.uploaded")))
                    actions_head(label: texts.t("common.actions"))
                )
            )
            table_body(
                #[key(file.id)]
                for file in rows {
                    let page = href!(file_id::file, file_id::FileId(file.id)).resolve(cx);
                    let name_link = page.clone();
                    table_row(
                        attrs: attributes! { id=(format!("file-{}", file.id)) },
                        table_cell(
                            topcoat::runtime::link(
                                href: name_link,
                                attrs: attributes! { class="font-medium underline-offset-4 hover:underline" },
                                (file.original_name.as_str())
                            )
                        )
                        table_cell((file.mime_type.as_str()))
                        table_cell(
                            (texts.t_with(
                                "common.bytes",
                                vec![("size", file.size.into())],
                            ))
                        )
                        table_cell((texts.datetime(file.created_at)))
                        actions_cell(
                            icon_link(
                                href: page,
                                name: Icon::Eye,
                                label: texts.t("common.details")
                            )
                            icon_download(
                                href: file.url.as_str(),
                                name: Icon::ArrowDownTray,
                                label: texts.t("files.detail.download")
                            )
                        )
                    )
                }
            )
        )
    })
}

#[route(POST)]
pub(crate) async fn upload_file(cx: &Cx, mut multipart: Multipart) -> Result<()> {
    require_permission(cx, Permission::FilesManage).await?;
    let mut upload = None;
    while let Some(part) = multipart.next_field().await? {
        if part.name() != Some("file") {
            continue;
        }
        let name = part.file_name().unwrap_or_default().to_owned();
        let mime_type = part
            .content_type()
            .map_or("application/octet-stream", |mime| mime)
            .to_owned();
        let data = part.bytes().await?.to_vec();
        if !name.is_empty() {
            upload = Some(Upload { name, mime_type, data });
        }
    }
    let page = href!(files).resolve(cx);
    let Some(upload) = upload else {
        let no_file = Problem::new(StatusCode::BAD_REQUEST, "files.errors.no_file");
        return Err(refused(cx, page, BackendError::Refused(no_file)));
    };

    match api::upload(cx, upload).await {
        Ok(file) => done(href!(file_id::file, file_id::FileId(file.id)).resolve(cx)),
        Err(err) => Err(refused(cx, page, err)),
    }
}
