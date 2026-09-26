//! ICU MessageFormat syntax checks. Messages are rendered by clients; the
//! server only makes sure an edited message parses and asks for no argument
//! the code never supplies.

use std::collections::BTreeSet;

use swc_icu_messageformat_parser::{AstElement, Parser, ParserOptions};

/// The argument names `message` refers to, or `None` when it does not parse.
pub fn placeholders(message: &str) -> Option<BTreeSet<String>> {
	let options = ParserOptions {
		requires_other_clause: true,
		..ParserOptions::default()
	};
	let mut parser = Parser::new(message, &options);
	let ast = parser.parse().ok()?;
	let mut names = BTreeSet::new();
	collect(&ast, &mut names);
	Some(names)
}

fn collect(elements: &[AstElement<'_>], names: &mut BTreeSet<String>) {
	for element in elements {
		match element {
			AstElement::Argument { value, .. }
			| AstElement::Number { value, .. }
			| AstElement::Date { value, .. }
			| AstElement::Time { value, .. } => {
				names.insert(value.clone());
			}
			AstElement::Select { value, options, .. } | AstElement::Plural { value, options, .. } => {
				names.insert(value.clone());
				for (_, option) in &options.0 {
					collect(&option.value, names);
				}
			}
			AstElement::Tag { children, .. } => collect(children, names),
			AstElement::Literal { .. } | AstElement::Pound(_) => {}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeSet;

	use super::placeholders;

	fn names(list: &[&str]) -> Option<BTreeSet<String>> {
		Some(list.iter().map(ToString::to_string).collect())
	}

	#[test]
	fn collects_arguments_including_nested_plural_branches() {
		let message = "{min, plural, one {# znak} few {# znaki} many {# znaków} other {# znaku}} dla {name}";

		assert_eq!(placeholders(message), names(&["min", "name"]));
		assert_eq!(placeholders("Plain text"), names(&[]));
	}

	#[test]
	fn rejects_broken_syntax_and_plural_without_other() {
		assert_eq!(placeholders("{min, plural, one {# znak}"), None);
		assert_eq!(placeholders("{min, plural, one {# znak}}"), None);
	}
}
