use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, page};
use topcoat::view::{View, view};

use super::{Section, legal_document};

const SECTIONS: &[Section] = &[
    Section::plain("legal.terms.p1"),
    Section::plain("legal.terms.p2"),
    Section::plain("legal.terms.p3"),
    Section::plain("legal.terms.p4"),
    Section::plain("legal.terms.p5"),
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
