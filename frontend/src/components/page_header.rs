use topcoat::Result;
use topcoat::view::{Child, View, component, view};

#[component]
pub async fn page_header(
    #[into] title: String,
    #[default] description: Option<String>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div
            class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between"
        >
            <div class="flex flex-col gap-1">
                <h1 class="text-2xl font-semibold tracking-tight">(title)</h1>
                if let Some(description) = description {
                    <p class="text-sm text-muted-foreground">(description)</p>
                }
            </div>
            <div class="flex flex-wrap gap-2">(child)</div>
        </div>
    })
}
