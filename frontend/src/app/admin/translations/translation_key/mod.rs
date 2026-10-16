pub(crate) mod reset;

use serde::Deserialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::Form;
use topcoat::router::error::not_found;
use topcoat::router::{href, module_param, page, path_param, route};
use topcoat::view::{View, attributes, component, view};

use super::api::{self as catalog, TextChanges, Translation};
use crate::auth::{Permission, require_permission};
use crate::backend::{BackendError, PageRequest};
use crate::components::back_link::back_link;
use crate::components::button::{ButtonVariant, button};
use crate::components::card::{card, card_content, card_description, card_footer, card_header, card_title};
use crate::components::feedback::{field_errors, problem_alert};
use crate::components::field::{field, field_description, field_group, field_label};
use crate::components::icon::{Icon, IconSize, icon};
use crate::components::page_header::page_header;
use crate::components::pagination::MAX_PAGE_SIZE;
use crate::components::textarea::textarea;
use crate::forms::{done, refused_with, typed_or};
use crate::i18n::{Lang, Texts, texts};

module_param!(pub translation_key);

#[page]
pub(crate) async fn translation(cx: &Cx) -> Result<impl View> {
    let current = require_permission(cx, Permission::TranslationsView).await?;
    let texts = texts(cx).await?;
    let key = path_param::<TranslationKey>(cx).to_owned();
    let message = catalog::list(
        cx,
        &key,
        PageRequest {
            page: 0,
            size: MAX_PAGE_SIZE,
        },
    )
    .await
    .map_err(BackendError::into_page_error)?
    .content
    .into_iter()
    .find(|message| message.key == key)
    .ok_or_else(not_found)?;
    let may_edit = current.may(Permission::TranslationsEdit);

    Ok(view! {
        page_header(
            title: texts.t("admin.translations.edit.title"),
            back_link(
                href: href!(super::translations).resolve(cx),
                label: texts.t("admin.translations.back")
            )
        )
        problem_alert()
        card(
            card_header(
                card_title(<h2 class="font-mono break-all">(message.key.as_str())</h2>)
                card_description((texts.t("admin.translations.edit.description")))
            )
            card_content(
                <form
                    id="translation"
                    method="post"
                    action=(href!(update_translation, TranslationKey(key.clone())))
                >
                    <fieldset disabled=(!may_edit)>
                        field_group(
                            message_field(
                                texts: texts,
                                message: &message,
                                lang: Lang::Pl
                            )
                            message_field(
                                texts: texts,
                                message: &message,
                                lang: Lang::En
                            )
                        )
                    </fieldset>
                </form>
            )
            if may_edit {
                card_footer(
                    attrs: attributes! { class="flex-wrap" },
                    button(
                        attrs: attributes! { type="submit" form="translation" },
                        icon(name: Icon::Check, size: IconSize::Sm)
                        (texts.t("common.save"))
                    )
                    if message.customized {
                        <form
                            method="post"
                            action=(href!(
                                reset::reset_translation,
                                TranslationKey(key.clone()),
                            ))
                        >
                            button(
                                variant: ButtonVariant::Outline,
                                attrs: attributes! { type="submit" },
                                icon(name: Icon::ArrowPath, size: IconSize::Sm)
                                (texts.t("admin.translations.edit.reset"))
                            )
                        </form>
                    }
                )
            }
        )
    })
}

#[component]
async fn message_field(cx: &Cx, texts: &Texts, message: &Translation, lang: Lang) -> Result<impl View> {
    let (name, label_key, current, default) = match lang {
        Lang::Pl => ("pl", "layout.language.pl", &message.message.pl, &message.default.pl),
        Lang::En => ("en", "layout.language.en", &message.message.en, &message.default.en),
    };
    let value = typed_or(cx, name, current);
    Ok(view! {
        field(
            field_label(attrs: attributes! { for=(name) }, (texts.t(label_key)))
            textarea(
                attrs: attributes! {
                    id=(name)
                    name=(name)
                    lang=(name)
                    rows="3"
                    required=""
                    maxlength="2000"
                },
                (value)
            )
            field_description(
                (texts.t_with(
                    "admin.translations.edit.default",
                    vec![("text", default.clone().into())],
                ))
            )
            field_errors(name: name)
        )
    })
}

#[derive(Deserialize)]
struct MessageForm {
    pl: String,
    en: String,
}

#[route(POST)]
pub(crate) async fn update_translation(cx: &Cx, Form(form): Form<MessageForm>) -> Result<()> {
    require_permission(cx, Permission::TranslationsEdit).await?;
    let key = path_param::<TranslationKey>(cx).to_owned();
    let page = href!(translation, TranslationKey(key.clone())).resolve(cx);
    let text = TextChanges {
        pl: &form.pl,
        en: &form.en,
    };
    match catalog::update(cx, &key, &text).await {
        Ok(_) => done(page),
        Err(err) => Err(refused_with(cx, page, err, [("pl", form.pl), ("en", form.en)])),
    }
}
