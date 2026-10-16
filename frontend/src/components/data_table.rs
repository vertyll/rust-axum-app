use topcoat::Result;
use topcoat::view::{Child, View, component, view};

use super::table::table;

/// A table in a bordered frame: every column shown, the rows
/// scrolling sideways on a narrow screen under the actions column, which
/// [`actions_head`](super::row_actions::actions_head) and
/// [`actions_cell`](super::row_actions::actions_cell) keep at the right edge.
#[component]
pub async fn data_table(#[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <div class="overflow-hidden rounded-lg border border-border bg-background">
            table((child))
        </div>
    })
}
