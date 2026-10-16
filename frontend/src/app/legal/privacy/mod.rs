use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, page};
use topcoat::view::{View, view};

use super::legal_document;

const PARAGRAPHS: &[&str] = &[
    "legal.privacy.p1",
    "legal.privacy.p2",
    "legal.privacy.p3",
    "legal.privacy.p4",
    "legal.privacy.p5",
    "legal.privacy.p6",
    "legal.privacy.p7",
];

#[page]
pub(crate) async fn privacy(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        legal_document(
            title_key: "legal.privacy.title",
            paragraph_keys: PARAGRAPHS,
            other_href: href!(super::terms::terms).resolve(cx),
            other_key: "legal.privacy.other"
        )
    })
}
