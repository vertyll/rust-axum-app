use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, page};
use topcoat::view::{View, view};

use super::{Section, legal_document};

const SECTIONS: &[Section] = &[
    Section::titled("legal.terms.h1", "legal.terms.p1"),
    Section::titled("legal.terms.h2", "legal.terms.p2"),
    Section::titled("legal.terms.h3", "legal.terms.p3"),
    Section::titled("legal.terms.h4", "legal.terms.p4"),
    Section::titled("legal.terms.h5", "legal.terms.p5"),
];

#[page]
pub(crate) async fn terms(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        legal_document(
            title_key: "legal.terms.title",
            sections: SECTIONS,
            other_href: href!(super::privacy::privacy).resolve(cx),
            other_key: "legal.terms.other"
        )
    })
}
