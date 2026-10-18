//! The navigation: a bar on top with the modules, the
//! account menu and the language, and a side bar with the sections of the
//! current module. On a phone the modules fold into a menu, and the sections
//! sit under the bar.

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::href;
use topcoat::router::request::uri;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, class, component, view};

use super::{account, admin, files, home, language, sign_out, theme};
use crate::auth::{CurrentUser, Permission, fetch_current_user};
use crate::components::button::{ButtonSize, ButtonVariant, button_variants};
use crate::components::dropdown::{
    DROPDOWN_ITEM, DROPDOWN_ITEM_ACTIVE, DROPDOWN_ITEM_DESTRUCTIVE, DropdownAlign, dropdown,
};
use crate::components::icon::{Icon, IconSize, icon};
use crate::i18n::{Lang, texts};
use crate::keycloak;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Module {
    Start,
    Files,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Home,
    Files,
    Translations,
    Roles,
    Users,
}

impl Module {
    const ALL: [Module; 3] = [Module::Start, Module::Files, Module::Admin];

    fn label_key(self) -> &'static str {
        match self {
            Module::Start => "layout.nav.home",
            Module::Files => "layout.nav.files",
            Module::Admin => "layout.nav.admin",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Module::Start => Icon::Home,
            Module::Files => Icon::Folder,
            Module::Admin => Icon::Cog6Tooth,
        }
    }

    fn sections(self) -> &'static [Section] {
        match self {
            Module::Start => &[Section::Home],
            Module::Files => &[Section::Files],
            Module::Admin => &[Section::Translations, Section::Roles, Section::Users],
        }
    }

    fn visible_sections(self, current: &CurrentUser) -> Vec<Section> {
        self.sections()
            .iter()
            .copied()
            .filter(|section| section.permission().is_none_or(|permission| current.may(permission)))
            .collect()
    }
}

impl Section {
    fn href(self, cx: &Cx) -> String {
        match self {
            Section::Home => href!(home).resolve(cx),
            Section::Files => href!(files::files).resolve(cx),
            Section::Translations => href!(admin::translations::translations).resolve(cx),
            Section::Roles => href!(admin::roles::roles).resolve(cx),
            Section::Users => href!(admin::users::users).resolve(cx),
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            Section::Home => "layout.nav.home",
            Section::Files => "layout.nav.files",
            Section::Translations => "layout.nav.translations",
            Section::Roles => "layout.nav.roles",
            Section::Users => "layout.nav.users",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Section::Home => Icon::Home,
            Section::Files => Icon::Folder,
            Section::Translations => Icon::Language,
            Section::Roles => Icon::ShieldCheck,
            Section::Users => Icon::Users,
        }
    }

    fn description_key(self) -> &'static str {
        match self {
            Section::Home => "home.signed_in",
            Section::Files => "home.sections.files",
            Section::Translations => "home.sections.translations",
            Section::Roles => "home.sections.roles",
            Section::Users => "home.sections.users",
        }
    }

    fn permission(self) -> Option<Permission> {
        match self {
            Section::Home => None,
            Section::Files => Some(Permission::FilesView),
            Section::Translations => Some(Permission::TranslationsView),
            Section::Roles => Some(Permission::RolesView),
            Section::Users => Some(Permission::UsersView),
        }
    }

    fn is_current(self, cx: &Cx) -> bool {
        let section = self.href(cx);
        let path = uri(cx).path();
        if section == "/" {
            return path == "/";
        }
        path == section || path.starts_with(&format!("{section}/"))
    }
}

fn modules(current: &CurrentUser) -> Vec<(Module, Vec<Section>)> {
    Module::ALL
        .into_iter()
        .map(|module| (module, module.visible_sections(current)))
        .filter(|(_, sections)| !sections.is_empty())
        .collect()
}

/// The module whose section is open, else the first, so the side bar always
/// has something to show.
fn current_module(cx: &Cx, modules: &[(Module, Vec<Section>)]) -> Option<(Module, Vec<Section>)> {
    modules
        .iter()
        .find(|(_, sections)| sections.iter().any(|section| section.is_current(cx)))
        .or_else(|| modules.first())
        .cloned()
}

const ITEM: &str =
    "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors hover:bg-foreground/5";
const ITEM_ACTIVE: &str = "flex items-center gap-2 rounded-md bg-foreground/5 px-3 py-2 text-sm font-medium";

#[component]
pub async fn navbar(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let current = fetch_current_user(cx).await?;
    let modules = current.map(modules).unwrap_or_default();
    let open = current_module(cx, &modules).map(|(module, _)| module);

    Ok(view! {
        <nav
            class="sticky top-0 z-40 h-16 border-b border-border bg-background"
            aria-label=(texts.t("layout.nav.label"))
        >
            <div class="flex h-full items-center justify-between gap-2 px-3 md:px-4">
                <div class="flex items-center gap-4">
                    link(
                        href: href!(home),
                        attrs: attributes! {
                            class=(class!(
                                "text-lg font-bold whitespace-nowrap",
                                "hidden md:block" if current.is_some(),
                            ))
                        },
                        (texts.t("app.name"))
                    )
                    <div class="hidden items-center gap-1 md:flex">
                        #[key(module.label_key())]
                        for (module, sections) in modules.iter() {
                            link(
                                href: sections[0].href(cx),
                                attrs: attributes! {
                                    class=(if open == Some(*module) {
                                        ITEM_ACTIVE
                                    } else {
                                        ITEM
                                    })
                                    aria-current=((open == Some(*module)).then_some("page"))
                                },
                                icon(name: module.icon())
                                <span>(texts.t(module.label_key()))</span>
                            )
                        }
                    </div>
                    if current.is_some() {
                        dropdown(
                            label: texts.t("layout.nav.toggle"),
                            trigger: view! {
                                icon(name: Icon::Bars3)
                                <span>(texts.t("layout.nav.menu"))</span>
                            }.into(

                            ),
                            align: DropdownAlign::Start,
                            class: Some("md:hidden"),
                            #[key(module.label_key())]
                            for (module, sections) in modules.iter() {
                                link(
                                    href: sections[0].href(cx),
                                    attrs: attributes! {
                                        class=(if open == Some(*module) {
                                            DROPDOWN_ITEM_ACTIVE
                                        } else {
                                            DROPDOWN_ITEM
                                        })
                                    },
                                    icon(name: module.icon())
                                    <span>(texts.t(module.label_key()))</span>
                                )
                            }
                            <div class="mt-1 border-t border-border pt-1">
                                account_items()
                            </div>
                        )
                    }
                </div>
                <div class="flex items-center gap-1">
                    if current.is_some() {
                        dropdown(
                            label: texts.t("layout.nav.toggle"),
                            trigger: view! { <span>(texts.t("layout.nav.menu"))</span> }.into(

                            ),
                            class: Some("hidden md:block"),
                            account_items()
                        )
                    }
                    theme_menu()
                    language_menu()
                    if current.is_none() {
                        <a
                            href=(keycloak::sign_in(cx))
                            class=(button_variants(
                                ButtonVariant::Primary,
                                ButtonSize::Sm,
                            ))
                        >
                            (texts.t("auth.sign_in"))
                        </a>
                    }
                </div>
            </div>
        </nav>
    })
}

#[component]
async fn account_items(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    Ok(view! {
        link(
            href: href!(account::account),
            attrs: attributes! { class=(DROPDOWN_ITEM) },
            icon(name: Icon::UserCircle, size: IconSize::Sm)
            <span>(texts.t("layout.nav.account"))</span>
        )
        <form method="post" action=(href!(sign_out::sign_out))>
            <button type="submit" class=(DROPDOWN_ITEM_DESTRUCTIVE)>
                icon(name: Icon::ArrowRightOnRectangle, size: IconSize::Sm)
                <span>(texts.t("layout.sign_out"))</span>
            </button>
        </form>
    })
}

/// Each choice posts the form, so the language changes without scripts.
#[component]
async fn language_menu(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let back = uri(cx).path_and_query().map_or("/", |path| path.as_str()).to_owned();
    Ok(view! {
        dropdown(
            label: texts.t("layout.language.label"),
            trigger: view! { <span class="uppercase">(texts.lang().code())</span> }.into(

            ),
            <form method="post" action=(href!(language::change_language))>
                <input type="hidden" name="back" value=(back)>
                for lang in Lang::ALL {
                    <button
                        type="submit"
                        name="lang"
                        value=(lang.code())
                        lang=(lang.code())
                        aria-pressed=(if lang == texts.lang() {
                            "true"
                        } else {
                            "false"
                        })
                        class=(if lang == texts.lang() {
                            DROPDOWN_ITEM_ACTIVE
                        } else {
                            DROPDOWN_ITEM
                        })
                    >
                        (texts.t(language_name_key(lang)))
                    </button>
                }
            </form>
        )
    })
}

fn language_name_key(lang: Lang) -> &'static str {
    match lang {
        Lang::Pl => "layout.language.pl",
        Lang::En => "layout.language.en",
    }
}

/// Each choice posts the form, so the theme changes without scripts.
#[component]
async fn theme_menu(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let back = uri(cx).path_and_query().map_or("/", |path| path.as_str()).to_owned();
    let current = crate::theme::theme(cx);
    Ok(view! {
        dropdown(
            label: texts.t("layout.theme.label"),
            trigger: view! { icon(name: theme_icon(current), size: IconSize::Sm) }.into(),
            <form method="post" action=(href!(theme::change_theme))>
                <input type="hidden" name="back" value=(back)>
                for choice in Theme::ALL {
                    <button
                        type="submit"
                        name="theme"
                        value=(choice.code())
                        aria-pressed=(if choice == current { "true" } else { "false" })
                        class=(if choice == current {
                            DROPDOWN_ITEM_ACTIVE
                        } else {
                            DROPDOWN_ITEM
                        })
                    >
                        icon(name: theme_icon(choice), size: IconSize::Sm)
                        <span>(texts.t(theme_name_key(choice)))</span>
                    </button>
                }
            </form>
        )
    })
}

fn theme_icon(theme: Theme) -> Icon {
    match theme {
        Theme::System => Icon::ComputerDesktop,
        Theme::Light => Icon::Sun,
        Theme::Dark => Icon::Moon,
    }
}

fn theme_name_key(theme: Theme) -> &'static str {
    match theme {
        Theme::System => "layout.theme.system",
        Theme::Light => "layout.theme.light",
        Theme::Dark => "layout.theme.dark",
    }
}

#[component]
pub async fn sidebar(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let sections = fetch_current_user(cx)
        .await?
        .and_then(|current| current_module(cx, &modules(current)))
        .map(|(_, sections)| sections)
        .unwrap_or_default();
    Ok(view! {
        if !sections.is_empty() {
            <nav
                class="fixed top-16 bottom-0 left-0 hidden w-24 border-r border-border bg-card md:block"
                aria-label=(texts.t("layout.sidebar.label"))
            >
                <ul class="flex h-full flex-col items-center gap-4 py-8">
                    #[key(section.label_key())]
                    for section in sections.iter().copied() {
                        <li>
                            link(
                                href: section.href(cx),
                                attrs: attributes! {
                                    class=(class!(
                                        "flex w-20 flex-col items-center gap-2 rounded-xl py-2 text-xs font-medium transition-colors hover:bg-foreground/5",
                                        "bg-foreground/5" if section.is_current(cx),
                                    ))
                                    aria-current=(section.is_current(cx).then_some("page"))
                                },
                                icon(name: section.icon(), size: IconSize::Lg)
                                <span class="text-center">
                                    (texts.t(section.label_key()))
                                </span>
                            )
                        </li>
                    }
                </ul>
            </nav>
            <nav
                class="border-b border-border bg-background md:hidden"
                aria-label=(texts.t("layout.sidebar.label"))
            >
                #[key(section.label_key())]
                for section in sections.iter().copied() {
                    link(
                        href: section.href(cx),
                        attrs: attributes! {
                            class=(class!(
                                "flex items-center gap-3 px-4 py-2.5 text-sm transition-colors hover:bg-foreground/5",
                                "bg-foreground/5 font-medium" if section.is_current(cx),
                            ))
                            aria-current=(section.is_current(cx).then_some("page"))
                        },
                        icon(name: section.icon())
                        <span>(texts.t(section.label_key()))</span>
                    )
                }
            </nav>
        }
    })
}

#[component]
pub async fn section_cards(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let sections: Vec<Section> = fetch_current_user(cx)
        .await?
        .map(|current| {
            modules(current)
                .into_iter()
                .flat_map(|(_, sections)| sections)
                .collect()
        })
        .unwrap_or_default();
    let card_class =
        "flex h-full items-start gap-4 rounded-xl border border-border p-5 transition-colors hover:bg-foreground/5";
    Ok(view! {
        <ul class="grid gap-4 sm:grid-cols-2">
            #[key(section.label_key())]
            for section in sections
                .into_iter()
                .filter(|section| *section != Section::Home) {
                <li id=(format!("section-{}", section.label_key()))>
                    link(
                        href: section.href(cx),
                        attrs: attributes! { class=(card_class) },
                        <span class="text-primary">
                            icon(name: section.icon(), size: IconSize::Lg)
                        </span>
                        <span>
                            <span class="block font-medium">
                                (texts.t(section.label_key()))
                            </span>
                            <span class="mt-1 block text-sm text-muted-foreground">
                                (texts.t(section.description_key()))
                            </span>
                        </span>
                    )
                </li>
            }
            <li id="section-account">
                link(
                    href: href!(account::account),
                    attrs: attributes! { class=(card_class) },
                    <span class="text-primary">
                        icon(name: Icon::UserCircle, size: IconSize::Lg)
                    </span>
                    <span>
                        <span class="block font-medium">
                            (texts.t("layout.nav.account"))
                        </span>
                        <span class="mt-1 block text-sm text-muted-foreground">
                            (texts.t("home.sections.account"))
                        </span>
                    </span>
                )
            </li>
        </ul>
    })
}
