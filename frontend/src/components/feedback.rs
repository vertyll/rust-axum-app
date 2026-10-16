//! How a refused form tells the reader what went wrong.

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::view::{View, attributes, component, view};

use crate::components::alert::{AlertVariant, alert, alert_description, alert_title};
use crate::components::field::field_error;
use crate::forms::form_failure;
use crate::i18n::texts;

/// The back-end's refusal of the form just sent, if there was one.
#[component]
pub async fn problem_alert(cx: &Cx) -> Result<impl View> {
    let texts = texts(cx).await?;
    let message = form_failure(cx).map(|failure| texts.problem(&failure.problem));
    Ok(view! {
        if let Some(message) = message {
            alert(
                variant: AlertVariant::Destructive,
                attrs: attributes! { role="alert" class="mb-6" },
                alert_title((texts.t("common.form.refused")))
                alert_description((message))
            )
        }
    })
}

/// The messages a refused form gives one of its fields.
#[component]
pub async fn field_errors(cx: &Cx, name: &'static str) -> Result<impl View> {
    let texts = texts(cx).await?;
    let messages = form_failure(cx)
        .map(|failure| texts.field_errors(&failure.problem, name))
        .unwrap_or_default();
    Ok(view! {
        for message in messages {
            field_error((message))
        }
    })
}
