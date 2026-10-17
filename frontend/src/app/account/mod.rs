use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::page;
use topcoat::view::{View, attributes, component, view};

use crate::auth::require_auth;
use crate::backend::BackendError;
use crate::components::badge::{BadgeVariant, badge};
use crate::components::button::{ButtonSize, ButtonVariant, button_variants};
use crate::components::card::{card, card_content, card_description, card_header, card_title};
use crate::components::page_header::page_header;
use crate::i18n::texts;
use crate::keycloak::{self, AccountAction, account_action};

#[page]
pub(crate) async fn account(cx: &Cx) -> Result<impl View> {
    let current = require_auth(cx).await?;
    let texts = texts(cx).await?;
    let user = &current.user;
    let grants = &current.grants;

    Ok(view! {
        page_header(
            title: texts.t("account.title"),
            description: Some(texts.t("account.description"))
        )
        <div class="grid items-start gap-6 md:grid-cols-2">
            card(
                card_header(card_title(<h2>(texts.t("account.profile.title"))</h2>))
                card_content(
                    <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
                        <dt class="text-muted-foreground">
                            (texts.t("account.profile.name"))
                        </dt>
                        <dd>(user.display_name())</dd>
                        <dt class="text-muted-foreground">
                            (texts.t("account.profile.email"))
                        </dt>
                        <dd class="break-all">(user.email.as_str())</dd>
                        <dt class="text-muted-foreground">
                            (texts.t("account.profile.roles"))
                        </dt>
                        <dd class="flex flex-wrap gap-1">
                            for role in user.roles.iter() {
                                badge(variant: BadgeVariant::Secondary, (role.as_str()))
                            }
                        </dd>
                        <dt class="text-muted-foreground">
                            (texts.t("account.profile.since"))
                        </dt>
                        <dd>(texts.datetime(user.created_at))</dd>
                    </dl>
                )
            )
            card(
                card_header(
                    card_title(<h2>(texts.t("account.grants.title"))</h2>)
                    card_description((texts.t("account.grants.description")))
                )
                card_content(
                    if grants.unrestricted {
                        badge((texts.t("account.grants.unrestricted")))
                    } else if grants.permissions.is_empty() {
                        <p class="text-sm text-muted-foreground">
                            (texts.t("account.grants.none"))
                        </p>
                    } else {
                        <ul class="flex flex-wrap gap-1">
                            for permission in grants.permissions.iter() {
                                <li>
                                    badge(
                                        variant: BadgeVariant::Outline,
                                        (permission.as_str())
                                    )
                                </li>
                            }
                        </ul>
                    }
                )
            )
            security_card()
        </div>
    })
}

/// Changing the password and turning the second factor on or off, each on
/// Keycloak's own pages.
#[component]
async fn security_card(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let second_factor = keycloak::second_factor(cx)
        .await
        .map_err(BackendError::into_page_error)?;
    let totp_action = second_factor.toggle();
    let (totp_label, totp_status) = if second_factor.is_on() {
        ("account.security.totp_disable", "account.security.totp_on")
    } else {
        ("account.security.totp", "account.security.totp_off")
    };
    Ok(view! {
        card(
            attrs: attributes! { class="md:col-span-2" },
            card_header(
                card_title(<h2>(texts.t("account.security.title"))</h2>)
                card_description((texts.t("account.security.description")))
            )
            card_content(
                <p class="mb-4 text-sm">(texts.t(totp_status))</p>
                <div class="flex flex-col gap-2 sm:flex-row">
                    <a
                        href=(account_action(cx, &AccountAction::UpdatePassword))
                        class=(button_variants(ButtonVariant::Outline, ButtonSize::Md))
                    >
                        (texts.t("account.security.password"))
                    </a>
                    <a
                        href=(account_action(cx, &totp_action))
                        class=(button_variants(ButtonVariant::Outline, ButtonSize::Md))
                    >
                        (texts.t(totp_label))
                    </a>
                </div>
            )
        )
    })
}
