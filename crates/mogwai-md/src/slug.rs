//! Tiny slugifier for heading `id` attributes.
//!
//! Lowercases, replaces whitespace with `-`, and keeps only alphanumerics and
//! `-`. Matches GitHub's heading-anchor behavior closely enough for v0.1
//! (GitHub also strips punctuation; we keep it simple).

/// Convert `s` into a URL-safe slug suitable for an HTML `id` attribute.
pub fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = false;
    for ch in s.trim().chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            prev_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn punctuation_stripped() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
    }

    #[test]
    fn underscores_become_dashes() {
        assert_eq!(slugify("hello_world"), "hello-world");
    }

    #[test]
    fn leading_trailing_dashes_trimmed() {
        assert_eq!(slugify("  hi  "), "hi");
    }

    #[test]
    fn empty_stays_empty() {
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn unicode_lowercase() {
        assert_eq!(slugify("Über æø"), "über-æø");
    }
}
