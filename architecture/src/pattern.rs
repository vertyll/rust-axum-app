//! Patterns over module paths: `identity::domain`, `*::domain::**`, `crate`.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Name(String),
    One,
    Any,
}

/// A pattern a module path is matched against, segment by segment: a name matches itself, `*` any one segment and `**`
/// any number of them, none included. A leading `crate` is optional, and `crate` alone is the crate root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pattern {
    text: String,
    segments: Vec<Segment>,
}

impl Pattern {
    pub(crate) fn parse(text: &str) -> Self {
        let trimmed = text.trim();
        let relative = trimmed
            .strip_prefix("crate::")
            .unwrap_or(if trimmed == "crate" { "" } else { trimmed });
        let segments = relative
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(|segment| match segment {
                "*" => Segment::One,
                "**" => Segment::Any,
                name => Segment::Name(name.to_owned()),
            })
            .collect();
        Self {
            text: trimmed.to_owned(),
            segments,
        }
    }

    pub(crate) fn matches(&self, path: &[String]) -> bool {
        matches_from(&self.segments, path)
    }

    /// The number of segments every path it matches has, or `None` where a `**` lets that vary.
    pub(crate) fn fixed_len(&self) -> Option<usize> {
        (!self.segments.contains(&Segment::Any)).then_some(self.segments.len())
    }
}

fn matches_from(segments: &[Segment], path: &[String]) -> bool {
    match segments.split_first() {
        None => path.is_empty(),
        Some((Segment::Any, rest)) => (0..=path.len()).any(|skipped| matches_from(rest, &path[skipped..])),
        Some((Segment::One, rest)) => !path.is_empty() && matches_from(rest, &path[1..]),
        Some((Segment::Name(name), rest)) => path.first() == Some(name) && matches_from(rest, &path[1..]),
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    fn path(text: &str) -> Vec<String> {
        text.split("::").filter(|s| !s.is_empty()).map(str::to_owned).collect()
    }

    #[test]
    fn a_name_matches_only_itself() {
        let pattern = Pattern::parse("identity::domain");
        assert!(pattern.matches(&path("identity::domain")));
        assert!(!pattern.matches(&path("identity::domain::user")));
        assert!(!pattern.matches(&path("files::domain")));
    }

    #[test]
    fn a_star_is_one_segment_and_a_double_star_any_number() {
        let pattern = Pattern::parse("*::domain::**");
        assert!(pattern.matches(&path("identity::domain")));
        assert!(pattern.matches(&path("identity::domain::user::email")));
        assert!(!pattern.matches(&path("domain")));
        assert!(!pattern.matches(&path("a::b::domain")));
    }

    #[test]
    fn crate_is_the_root_and_an_optional_prefix() {
        assert!(Pattern::parse("crate").matches(&[]));
        assert!(Pattern::parse("crate::identity").matches(&path("identity")));
        assert!(Pattern::parse("**").matches(&[]));
    }

    #[test]
    fn only_a_pattern_without_a_double_star_has_a_fixed_length() {
        assert_eq!(Pattern::parse("*").fixed_len(), Some(1));
        assert_eq!(Pattern::parse("identity::*").fixed_len(), Some(2));
        assert_eq!(Pattern::parse("identity::**").fixed_len(), None);
    }
}
