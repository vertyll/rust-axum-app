pub(crate) mod privacy;
pub(crate) mod terms;

use formatjs_icu_messageformat::DateTimeValue;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, component, view};

use crate::i18n::texts;

const CONTACT_EMAIL: &str = "gawrmiko@gmail.com";

const MODIFIED: (i32, u8, u8) = (2026, 10, 3);

#[component]
async fn legal_document(
    cx: &Cx,
    title_key: &'static str,
    paragraph_keys: &'static [&'static str],
    #[into] other_href: String,
    other_key: &'static str,
) -> Result<impl View> {
    let texts = texts(cx).await?;
    let (year, month, day) = MODIFIED;
    let modified = DateTimeValue::try_new(year, month, day, 0, 0, 0, 0).expect("the modification date is a valid date");
    let paragraphs: Vec<Vec<Segment>> = paragraph_keys
        .iter()
        .map(|key| segments(&texts.t_with(key, vec![("email", CONTACT_EMAIL.into())])))
        .collect();

    Ok(view! {
        <article class="mx-auto max-w-3xl">
            <h1 class="text-3xl font-semibold tracking-tight">(texts.t(title_key))</h1>
            <p class="mt-2 mb-8 text-sm text-muted-foreground">
                (texts.t_with("legal.modified", vec![("date", modified.into())]))
            </p>
            <div class="flex flex-col gap-4 leading-relaxed">
                for paragraph in paragraphs {
                    <p>
                        for segment in paragraph {
                            match segment {
                                Segment::Text(text) => (text),
                                Segment::Email => <a
                                    href=(format!("mailto:{CONTACT_EMAIL}"))
                                    class="underline underline-offset-4"
                                >
                                    (CONTACT_EMAIL)
                                </a>,
                            }
                        }
                    </p>
                }
            </div>
            link(
                href: other_href,
                attrs: attributes! {
                    class="mt-8 inline-block text-sm underline underline-offset-4"
                },
                (texts.t(other_key))
            )
        </article>
    })
}

#[derive(Debug, PartialEq, Eq)]
enum Segment {
    Text(String),
    Email,
}

fn segments(paragraph: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    for (index, text) in paragraph.split(CONTACT_EMAIL).enumerate() {
        if index > 0 {
            segments.push(Segment::Email);
        }
        if !text.is_empty() {
            segments.push(Segment::Text(text.to_owned()));
        }
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::{Segment, segments};

    #[test]
    fn the_contact_address_becomes_a_link() {
        let parts = segments("Contact: gawrmiko@gmail.com.");

        assert_eq!(
            parts,
            [
                Segment::Text("Contact: ".to_owned()),
                Segment::Email,
                Segment::Text(".".to_owned())
            ]
        );
    }
}
