use topcoat::{
    Result,
    view::{Attributes, Child, StaticClass, View, class, component, view},
};

const LABEL: StaticClass = class!(
    "flex items-center gap-2 text-sm leading-none font-medium select-none \
     peer-disabled:pointer-events-none peer-disabled:opacity-50 \
     peer-has-[:disabled]:pointer-events-none peer-has-[:disabled]:opacity-50 \
     has-[+:disabled]:pointer-events-none has-[+:disabled]:opacity-50 \
     has-[:disabled]:pointer-events-none has-[:disabled]:opacity-50",
);

/// A label for a form control.
///
/// Wrap the control or pass a `for` attribute matching its `id`. Pass the label text as children.
/// `attrs` are forwarded to the `<label>`, with extra classes added to its classes.
#[component]
pub async fn label(#[default] mut attrs: Attributes, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <label class=(class!(LABEL, attrs.remove("class"))) (attrs)>(child)</label>
    })
}
