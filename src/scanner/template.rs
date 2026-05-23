//! Aggregation-key template parser and evaluator.
//!
//! Templates are small expressions that extract a stable string key from an
//! artist or album, used to identify the same entity across multiple upstream
//! servers.
//!
//! # Syntax
//!
//! | Pattern                  | Meaning                                                    |
//! |--------------------------|------------------------------------------------------------|
//! | `{field}`                | Value of `field`; empty string if null/absent              |
//! | `{field\|t1\|t2}`       | Value of `field` with transforms `t1`, `t2` applied        |
//! | `{field:-fallback_expr}` | Value of `field` if non-empty, otherwise `fallback_expr`   |
//! | any other text           | Literal characters                                         |
//!
//! # Transforms
//! * `lowercase`        — convert to ASCII lower case
//! * `trim`             — strip leading/trailing whitespace
//! * `ascii_normalize`  — decompose Unicode, drop combining marks, keep ASCII
//! * `strip_articles`   — remove leading "The ", "A ", "An " (case-insensitive)
//!
//! # Example
//! ```
//! let t = Template::parse("{music_brainz_id:-{name|lowercase|trim}}");
//! ```

use std::fmt::Write as _;

// ── Public API ────────────────────────────────────────────────────────────────

/// A compiled aggregation-key template.
#[derive(Debug, Clone)]
pub struct Template {
    segments: Vec<Segment>,
}

impl Template {
    /// Parse a template string.  Never returns an error — unknown field names
    /// simply evaluate to an empty string so users get a usable (if surprising)
    /// result rather than a hard failure.
    pub fn parse(src: &str) -> Self {
        Self { segments: parse_segments(src) }
    }

    /// Evaluate the template against a set of named fields.
    ///
    /// `lookup` is called with a field name and should return the raw value or
    /// `None` if the field is absent.
    pub fn eval(&self, lookup: &impl Fn(&str) -> Option<String>) -> String {
        eval_segments(&self.segments, lookup)
    }
}

// ── Context helpers ───────────────────────────────────────────────────────────

/// Evaluate an artist template against an `opensubsonic::data::ArtistId3`.
pub fn eval_artist(template: &Template, artist: &opensubsonic::data::ArtistId3) -> String {
    template.eval(&|field| match field {
        "music_brainz_id" => artist.music_brainz_id.clone(),
        "name" => Some(artist.name.clone()),
        "sort_name" => artist.sort_name.clone(),
        _ => None,
    })
}

/// Evaluate an album template against an `opensubsonic::data::AlbumId3`.
///
/// `artist_key` is the already-computed aggregation key for this album's artist.
pub fn eval_album(
    template: &Template,
    album: &opensubsonic::data::AlbumId3,
    artist_key: &str,
) -> String {
    template.eval(&|field| match field {
        "music_brainz_id" => album.music_brainz_id.clone(),
        "name" => Some(album.name.clone()),
        "sort_name" => album.sort_name.clone(),
        "year" => album.year.map(|y| y.to_string()),
        "display_artist" => album.display_artist.clone(),
        "artist_key" => Some(artist_key.to_owned()),
        _ => None,
    })
}

// ── Internal representation ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Segment {
    Literal(String),
    Field {
        name: String,
        transforms: Vec<Transform>,
        fallback: Vec<Segment>,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum Transform {
    Lowercase,
    Trim,
    AsciiNormalize,
    StripArticles,
}

// ── Parser ────────────────────────────────────────────────────────────────────

fn parse_segments(src: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut literal = String::new();
    let mut chars = src.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Flush any accumulated literal text
            if !literal.is_empty() {
                segments.push(Segment::Literal(std::mem::take(&mut literal)));
            }
            // Collect everything up to the matching '}'
            let mut inner = String::new();
            let mut depth = 1usize;
            for c in chars.by_ref() {
                match c {
                    '{' => { depth += 1; inner.push(c); }
                    '}' => {
                        depth -= 1;
                        if depth == 0 { break; }
                        inner.push(c);
                    }
                    _ => inner.push(c),
                }
            }
            segments.push(parse_field(&inner));
        } else {
            literal.push(ch);
        }
    }
    if !literal.is_empty() {
        segments.push(Segment::Literal(literal));
    }
    segments
}

/// Parse the content between `{` and `}`.
fn parse_field(inner: &str) -> Segment {
    // Split on `:-` (fallback separator) — but only at the top level (not inside nested `{}`)
    if let Some(pos) = find_fallback_separator(inner) {
        let field_part = &inner[..pos];
        let fallback_expr = &inner[pos + 2..]; // skip ":-"
        let (name, transforms) = parse_field_and_transforms(field_part);
        Segment::Field {
            name,
            transforms,
            fallback: parse_segments(fallback_expr),
        }
    } else {
        let (name, transforms) = parse_field_and_transforms(inner);
        Segment::Field { name, transforms, fallback: vec![] }
    }
}

/// Find the position of `:-` that isn't inside a `{…}` pair.
fn find_fallback_separator(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            b':' if depth == 0 && bytes.get(i + 1) == Some(&b'-') => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

fn parse_field_and_transforms(s: &str) -> (String, Vec<Transform>) {
    let mut parts = s.split('|');
    let name = parts.next().unwrap_or("").trim().to_owned();
    let transforms = parts
        .map(|t| match t.trim() {
            "lowercase" => Transform::Lowercase,
            "trim" => Transform::Trim,
            "ascii_normalize" => Transform::AsciiNormalize,
            "strip_articles" => Transform::StripArticles,
            _ => Transform::Trim, // unknown transform → no-op (trim is safe)
        })
        .collect();
    (name, transforms)
}

// ── Evaluator ─────────────────────────────────────────────────────────────────

fn eval_segments(segments: &[Segment], lookup: &impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    for seg in segments {
        match seg {
            Segment::Literal(s) => out.push_str(s),
            Segment::Field { name, transforms, fallback } => {
                let raw = lookup(name);
                let value = raw.map(|v| apply_transforms(&v, transforms));
                if let Some(v) = value.filter(|v| !v.is_empty()) {
                    out.push_str(&v);
                } else {
                    // Use fallback expression
                    let _ = write!(out, "{}", eval_segments(fallback, lookup));
                }
            }
        }
    }
    out
}

fn apply_transforms(s: &str, transforms: &[Transform]) -> String {
    let mut current = s.to_owned();
    for t in transforms {
        current = match t {
            Transform::Trim => current.trim().to_owned(),
            Transform::Lowercase => current.to_lowercase(),
            Transform::AsciiNormalize => ascii_normalize(&current),
            Transform::StripArticles => strip_leading_articles(&current),
        };
    }
    current
}

/// Remove Unicode combining characters and keep only ASCII printable chars.
fn ascii_normalize(s: &str) -> String {
    // A simple approximation: Unicode normalization requires a crate; instead
    // we map common Latin extended chars to their ASCII base and drop the rest.
    s.chars()
        .filter_map(|c| {
            if c.is_ascii() {
                Some(c)
            } else {
                // Map a few very common accented chars; others are dropped.
                Some(match c {
                    'à'|'á'|'â'|'ã'|'ä'|'å'|'À'|'Á'|'Â'|'Ã'|'Ä'|'Å' => 'a',
                    'è'|'é'|'ê'|'ë'|'È'|'É'|'Ê'|'Ë' => 'e',
                    'ì'|'í'|'î'|'ï'|'Ì'|'Í'|'Î'|'Ï' => 'i',
                    'ò'|'ó'|'ô'|'õ'|'ö'|'ø'|'Ò'|'Ó'|'Ô'|'Õ'|'Ö'|'Ø' => 'o',
                    'ù'|'ú'|'û'|'ü'|'Ù'|'Ú'|'Û'|'Ü' => 'u',
                    'ý'|'ÿ'|'Ý' => 'y',
                    'ñ'|'Ñ' => 'n',
                    'ç'|'Ç' => 'c',
                    'ß' => return Some('s'), // expands to 'ss' ideally, but 's' is fine
                    'æ'|'Æ' => 'a',
                    'œ'|'Œ' => 'o',
                    _ => return None, // drop unknown non-ASCII
                })
            }
        })
        .collect()
}

/// Remove common leading articles ("The ", "A ", "An ") case-insensitively.
fn strip_leading_articles(s: &str) -> String {
    for article in &["the ", "a ", "an "] {
        if s.len() > article.len() && s[..article.len()].eq_ignore_ascii_case(article) {
            return s[article.len()..].to_owned();
        }
    }
    s.to_owned()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn field<'a>(name: &'a str, value: &'a str) -> impl Fn(&str) -> Option<String> + 'a {
        move |f| if f == name { Some(value.to_owned()) } else { None }
    }

    #[test]
    fn literal_only() {
        let t = Template::parse("hello world");
        assert_eq!(t.eval(&|_| None), "hello world");
    }

    #[test]
    fn simple_field() {
        let t = Template::parse("{name}");
        assert_eq!(t.eval(&field("name", "Daft Punk")), "Daft Punk");
    }

    #[test]
    fn field_with_transforms() {
        let t = Template::parse("{name|lowercase|trim}");
        assert_eq!(t.eval(&field("name", "  Daft Punk  ")), "  daft punk  ".trim());
        // lowercase first, then trim
        let t2 = Template::parse("{name|trim|lowercase}");
        assert_eq!(t2.eval(&field("name", "  Daft Punk  ")), "daft punk");
    }

    #[test]
    fn fallback_when_empty() {
        let t = Template::parse("{music_brainz_id:-{name|lowercase|trim}}");
        // No MBID → falls back to normalized name
        assert_eq!(
            t.eval(&|f| if f == "name" { Some("Daft Punk".to_owned()) } else { None }),
            "daft punk"
        );
        // With MBID → uses it directly
        assert_eq!(
            t.eval(&|f| match f {
                "music_brainz_id" => Some("abc-123".to_owned()),
                "name" => Some("Daft Punk".to_owned()),
                _ => None,
            }),
            "abc-123"
        );
    }

    #[test]
    fn different_case_same_key() {
        let t = Template::parse("{name|lowercase|trim}");
        let k1 = t.eval(&field("name", "Daft Punk"));
        let k2 = t.eval(&field("name", "daft punk"));
        assert_eq!(k1, k2, "case variants should produce the same key");
    }

    #[test]
    fn ascii_normalize_transform() {
        let t = Template::parse("{name|ascii_normalize|lowercase}");
        assert_eq!(t.eval(&field("name", "Björk")), "bjork");
    }

    #[test]
    fn strip_articles_transform() {
        let t = Template::parse("{name|strip_articles|lowercase}");
        assert_eq!(t.eval(&field("name", "The Beatles")), "beatles");
        assert_eq!(t.eval(&field("name", "A-ha")), "a-ha"); // "A " needs trailing space
        assert_eq!(t.eval(&field("name", "A Ha")), "ha");
    }

    #[test]
    fn album_template_uses_artist_key() {
        let t = Template::parse("{music_brainz_id:-{artist_key}:{name|lowercase|trim}}");
        let key = t.eval(&|f| match f {
            "artist_key" => Some("daft punk".to_owned()),
            "name" => Some("Discovery".to_owned()),
            _ => None,
        });
        assert_eq!(key, "daft punk:discovery");
    }
}
