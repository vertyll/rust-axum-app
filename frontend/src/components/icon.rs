use topcoat::Result;
use topcoat::view::{View, component, view};

use super::icons;

/// An icon the pages draw, named as in Heroicons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Home,
    Folder,
    Cog6Tooth,
    Language,
    Sun,
    Moon,
    ComputerDesktop,
    ShieldCheck,
    Users,
    UserCircle,
    ArrowRightOnRectangle,
    ChevronDown,
    ChevronLeft,
    ChevronDoubleLeft,
    ChevronDoubleRight,
    ChevronRight,
    Bars3,
    Pencil,
    ArrowPath,
    MagnifyingGlass,
    Trash,
    ArrowDownTray,
    Eye,
    Check,
    XMark,
    ArrowUpTray,
    Plus,
}

impl Icon {
    fn data(self) -> topcoat::icon::IconData {
        match self {
            Icon::Home => icons::HOME,
            Icon::Folder => icons::FOLDER,
            Icon::Cog6Tooth => icons::COG_6_TOOTH,
            Icon::Language => icons::LANGUAGE,
            Icon::Sun => icons::SUN,
            Icon::Moon => icons::MOON,
            Icon::ComputerDesktop => icons::COMPUTER_DESKTOP,
            Icon::ShieldCheck => icons::SHIELD_CHECK,
            Icon::Users => icons::USERS,
            Icon::UserCircle => icons::USER_CIRCLE,
            Icon::ArrowRightOnRectangle => icons::ARROW_RIGHT_ON_RECTANGLE,
            Icon::ChevronDown => icons::CHEVRON_DOWN,
            Icon::ChevronLeft => icons::CHEVRON_LEFT,
            Icon::ChevronDoubleLeft => icons::CHEVRON_DOUBLE_LEFT,
            Icon::ChevronDoubleRight => icons::CHEVRON_DOUBLE_RIGHT,
            Icon::ChevronRight => icons::CHEVRON_RIGHT,
            Icon::Bars3 => icons::BARS_3,
            Icon::Pencil => icons::PENCIL,
            Icon::ArrowPath => icons::ARROW_PATH,
            Icon::MagnifyingGlass => icons::MAGNIFYING_GLASS,
            Icon::Trash => icons::TRASH,
            Icon::ArrowDownTray => icons::ARROW_DOWN_TRAY,
            Icon::Eye => icons::EYE,
            Icon::Check => icons::CHECK,
            Icon::XMark => icons::X_MARK,
            Icon::ArrowUpTray => icons::ARROW_UP_TRAY,
            Icon::Plus => icons::PLUS,
        }
    }
}

/// How large an icon is drawn: beside small text, in a button, or alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl IconSize {
    fn pixels(self) -> u16 {
        match self {
            IconSize::Sm => 16,
            IconSize::Md => 20,
            IconSize::Lg => 24,
        }
    }
}

/// An icon, hidden from screen readers: the text or label beside it names it.
#[component]
pub async fn icon(name: Icon, #[default] size: IconSize) -> Result<impl View> {
    Ok(view! { topcoat::icon::icon(data: name.data(), size: size.pixels()) })
}
