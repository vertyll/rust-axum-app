use topcoat::Result;
use topcoat::view::{Attributes, View, attributes, component, view};

use crate::components::feedback::field_errors;
use crate::components::field::{field, field_label};
use crate::components::input::input;

/// A labelled text input, with the messages a refused form gives it. `attrs`
/// go on the `<input>`, such as `required` or `maxlength`.
#[component]
pub async fn text_field(
    name: &'static str,
    #[into] label: String,
    #[into] value: String,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    Ok(view! {
        field(
            field_label(attrs: attributes! { for=(name) }, (label))
            input(attrs: attributes! { id=(name) name=(name) value=(value) (attrs) })
            field_errors(name: name)
        )
    })
}
