use topcoat::{
    Result,
    view::{Attributes, Child, Class, StaticClass, View, class, component, view},
};

/// The visual style of a [`badge`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BadgeVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Destructive,
}

impl BadgeVariant {
    /// Each variant sets its own border color, kept out of the shared base to avoid conflicting
    /// classes.
    fn classes(self) -> StaticClass {
        match self {
            Self::Primary => class!("border-transparent bg-primary text-primary-foreground"),
            Self::Secondary => class!("border-transparent bg-foreground/5 text-foreground"),
            Self::Outline => class!("border-border text-foreground"),
            Self::Destructive => {
                class!("border-transparent bg-destructive text-destructive-foreground")
            }
        }
    }
}

/// Classes shared by badge variants. A border reserves the same space in every variant.
const BASE: StaticClass = class!(
    "inline-flex w-fit shrink-0 items-center justify-center gap-1 rounded-md \
     border px-2 py-0.5 text-xs font-medium whitespace-nowrap",
);

/// Builds the full class list for a badge of the given `variant`.
#[must_use]
#[expect(dead_code, reason = "installed with the badge; no page styles a link as a badge yet")]
pub fn badge_variants(variant: BadgeVariant) -> Class<(StaticClass, StaticClass)> {
    class!(BASE, variant.classes())
}

/// A small label for a status or count.
///
/// `variant` defaults to `Primary`. Pass the label as children and extra attributes through
/// `attrs`. Attributes go on the `<span>`, with classes added to its classes. Use
/// [`badge_variants`] to apply the same styling to another element.
#[component]
pub async fn badge(
    #[default] variant: BadgeVariant,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <span class=(class!(BASE, variant.classes(), attrs.remove("class"))) (attrs)>
            (child)
        </span>
    })
}
