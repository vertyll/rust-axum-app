use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, page};
use topcoat::view::{View, view};

use super::{Section, legal_document};

const SECTIONS: &[Section] = &[
    Section::titled("legal.privacy.h1", "legal.privacy.p1"),
    Section::titled("legal.privacy.h2", "legal.privacy.p2"),
    Section::titled("legal.privacy.h3", "legal.privacy.p3"),
    Section::titled("legal.privacy.h4", "legal.privacy.p4"),
    Section::titled("legal.privacy.h5", "legal.privacy.p5"),
    Section::titled("legal.privacy.h6", "legal.privacy.p6"),
    Section::titled("legal.privacy.h7", "legal.privacy.p7"),
    Section::titled("legal.privacy.h8", "legal.privacy.p8"),
];

#[page]
pub(crate) async fn privacy(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        legal_document(
            title_key: "legal.privacy.title",
            sections: SECTIONS,
            other_href: href!(super::terms::terms).resolve(cx),
            other_key: "legal.privacy.other"
        )
    })
}
