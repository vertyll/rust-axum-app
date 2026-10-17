mod account;
mod admin;
mod files;
mod health;
mod language;
mod legal;
mod navigation;
mod sign_out;
mod theme;

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::error::{ForbiddenError, NotFoundError, UnauthorizedError};
use topcoat::router::header::{
    CONTENT_SECURITY_POLICY, HeaderValue, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use topcoat::router::response::Response;
use topcoat::router::{
    Body, Next, RouterBuilder, Slot, StatusCode, href, layer, layout, module_router, not_found, page, query_params,
};
use topcoat::runtime::link;
use topcoat::tailwind;
use topcoat::view::{View, attributes, class, component, error_boundary, view};

use self::navigation::{navbar, section_cards, sidebar};
use crate::auth::fetch_current_user;
use crate::components::button::{ButtonSize, ButtonVariant, button_variants};
use crate::components::card::{card, card_content, card_description, card_header, card_title};
use crate::i18n::texts;
use crate::keycloak;

not_found!();

pub fn router() -> RouterBuilder {
    module_router!()
        .route(admin::translations::message_list)
        .route(admin::users::user_list)
        .route(admin::roles::role_list)
        .route(files::file_list)
}

/// No other site may frame these pages, where a reader could be tricked into
/// clicking a form that acts with their session.
#[layer]
async fn security_headers(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let mut response = next.run(cx, body).await?;
    let headers = response.headers_mut();
    headers.insert(X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers
        .entry(CONTENT_SECURITY_POLICY)
        .or_insert(HeaderValue::from_static("frame-ancestors 'none'"));
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    Ok(response)
}

#[layout]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let texts = texts(cx).await?;
    // Read before the page renders, so the back-end's cookies reach the
    // browser with this response.
    let signed_in = fetch_current_user(cx).await?.is_some();

    Ok(view! {
        <!DOCTYPE html>
        <html lang=(texts.lang().code()) data-theme=(crate::theme::theme(cx).code())>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(texts.t("app.name"))</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::runtime::script()
                topcoat::dev::script()
            </head>
            <body class="flex min-h-screen flex-col">
                navbar()
                sidebar()
                <main
                    class=(class!("flex-1 px-4 py-6 md:px-8", "md:ml-24" if signed_in))
                >
                    <div class="mx-auto w-full max-w-5xl">
                        error_boundary(
                            fallback: |error| {
                                let refusal = if error
                                    .downcast_ref::<UnauthorizedError>()
                                    .is_some() {
                                    Refusal::SignedOut
                                } else if error.downcast_ref::<ForbiddenError>().is_some() {
                                    Refusal::Forbidden
                                } else if error.downcast_ref::<NotFoundError>().is_some() {
                                    Refusal::NotFound
                                } else {
                                    return Err(error);
                                };
                                Ok(view! { refusal_page(refusal: refusal) })
                            },
                            (slot)
                        )
                    </div>
                </main>
                site_footer(signed_in: signed_in)
            </body>
        </html>
    })
}

#[component]
async fn site_footer(cx: &Cx, signed_in: bool) -> Result<impl View> {
    let texts = texts(cx).await?;
    Ok(view! {
        <footer class=(class!("border-t border-border", "md:ml-24" if signed_in))>
            <nav
                class="mx-auto flex max-w-5xl flex-col items-center gap-2 px-4 py-4 text-center text-sm text-muted-foreground sm:px-6"
                aria-label=(texts.t("layout.footer.label"))
            >
                link(
                    href: href!(legal::terms::terms),
                    attrs: attributes! { class="hover:text-foreground" },
                    (texts.t("legal.terms.title"))
                )
                link(
                    href: href!(legal::privacy::privacy),
                    attrs: attributes! { class="hover:text-foreground" },
                    (texts.t("legal.privacy.title"))
                )
            </nav>
        </footer>
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    SignedOut,
    Forbidden,
    NotFound,
}

#[component]
async fn refusal_page(cx: &Cx, refusal: Refusal) -> Result<impl View> {
    let texts = texts(cx).await?;
    let (status, title, body) = match refusal {
        Refusal::SignedOut => (
            StatusCode::UNAUTHORIZED,
            "errors.page.signed_out.title",
            "errors.page.signed_out.body",
        ),
        Refusal::Forbidden => (
            StatusCode::FORBIDDEN,
            "errors.page.forbidden.title",
            "errors.page.forbidden.body",
        ),
        Refusal::NotFound => (
            StatusCode::NOT_FOUND,
            "errors.page.not_found.title",
            "errors.page.not_found.body",
        ),
    };
    Ok(view! {
        (status)
        card(
            attrs: attributes! { class="mx-auto max-w-md" },
            card_header(
                card_title(<h1>(texts.t(title))</h1>)
                card_description((texts.t(body)))
            )
            card_content(
                if refusal == Refusal::SignedOut {
                    <a
                        href=(keycloak::sign_in(cx))
                        class=(button_variants(ButtonVariant::Primary, ButtonSize::Md))
                    >
                        (texts.t("auth.sign_in"))
                    </a>
                } else {
                    link(
                        href: href!(home),
                        attrs: attributes! {
                            class=(button_variants(
                                ButtonVariant::Outline,
                                ButtonSize::Md,
                            ))
                        },
                        (texts.t("errors.page.home"))
                    )
                }
            )
        )
    })
}

#[query_params(error = bad_request)]
struct HomeQuery {
    error: Option<String>,
}

#[page]
pub(crate) async fn home(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let query = topcoat::router::query_params::<HomeQuery>(cx)?;
    let sign_in_error = match query.error.as_deref() {
        Some("state_mismatch") => Some("auth.errors.state_mismatch"),
        Some(_) => Some("auth.errors.sign_in_rejected"),
        None => None,
    };
    let current = fetch_current_user(cx).await?;

    Ok(view! {
        if let Some(key) = sign_in_error {
            <p
                role="alert"
                class="mb-6 rounded-lg border border-destructive/40 px-4 py-3 text-sm text-destructive"
            >
                (texts.t(key))
            </p>
        }
        match current {
            Some(current) => {
                <h1 class="mb-2 text-2xl font-semibold tracking-tight">
                    (texts.t_with(
                        "home.welcome",
                        vec![("name", current.user.display_name().into())],
                    ))
                </h1>
                <p class="mb-8 text-muted-foreground">(texts.t("home.signed_in"))</p>
                section_cards()
            }
            None => {
                <section class="mx-auto max-w-xl py-12 text-center">
                    <h1 class="mb-3 text-3xl font-semibold tracking-tight">
                        (texts.t("app.name"))
                    </h1>
                    <p class="mb-8 text-muted-foreground">(texts.t("home.guest"))</p>
                    <div class="flex flex-col justify-center gap-3 sm:flex-row">
                        <a
                            href=(keycloak::sign_in(cx))
                            class=(button_variants(
                                ButtonVariant::Primary,
                                ButtonSize::Lg,
                            ))
                        >
                            (texts.t("auth.sign_in"))
                        </a>
                        <a
                            href=(keycloak::register(cx))
                            class=(button_variants(
                                ButtonVariant::Outline,
                                ButtonSize::Lg,
                            ))
                        >
                            (texts.t("auth.register"))
                        </a>
                    </div>
                </section>
            }
        }
    })
}
