use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, page};
use topcoat::view::{View, view};

use super::legal_document;

const PARAGRAPHS: &[&str] = &[
    "legal.terms.p1",
    "legal.terms.p2",
    "legal.terms.p3",
    "legal.terms.p4",
    "legal.terms.p5",
];

#[page]
pub(crate) async fn terms(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        legal_document(
            title_key: "legal.terms.title",
            paragraph_keys: PARAGRAPHS,
            other_href: href!(super::privacy::privacy).resolve(cx),
            other_key: "legal.terms.other"
        )
    })
}
