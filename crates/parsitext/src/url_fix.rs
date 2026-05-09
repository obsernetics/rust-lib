//! URL helpers for Persian text.
//!
//! - [`encode`] — percent-encode a Persian string for use as a URL path
//!   segment or query value (UTF-8 → `%XX`, RFC 3986 unreserved kept).
//! - [`decode`] — reverse the percent-encoding back to readable Persian.
//! - [`fix`] — round-trip-clean a URL: percent-decode the path/query so
//!   Persian characters render legibly, but keep the scheme/authority
//!   untouched.
//!
//! Use [`encode`] when you are *building* URLs and want them spec-compliant.
//! Use [`decode`] / [`fix`] when you are *displaying* URLs to a Persian user
//! and want them to read naturally.

/// Percent-encode `text` per RFC 3986: keep `A-Z a-z 0-9 - _ . ~`,
/// percent-encode every other byte as `%XX`.
///
/// ```
/// use parsitext::url_fix::encode;
///
/// assert_eq!(encode("سلام"), "%D8%B3%D9%84%D8%A7%D9%85");
/// assert_eq!(encode("hello-world"), "hello-world");
/// ```
#[must_use]
pub fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for &b in text.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(hex_nibble(b >> 4));
            out.push(hex_nibble(b & 0x0F));
        }
    }
    out
}

/// Percent-decode `text`.  Invalid `%XX` sequences are passed through
/// verbatim so the function never fails.
///
/// ```
/// use parsitext::url_fix::decode;
///
/// assert_eq!(decode("%D8%B3%D9%84%D8%A7%D9%85"), "سلام");
/// // Mixed input: only valid sequences are decoded.
/// assert_eq!(decode("hello-%D8%A7"), "hello-ا");
/// // Stray % left intact.
/// assert_eq!(decode("100%"), "100%");
/// ```
#[must_use]
pub fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut buf: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                buf.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        buf.push(bytes[i]);
        i += 1;
    }
    // Lossy decoding so malformed UTF-8 doesn't drop the whole string.
    String::from_utf8_lossy(&buf).into_owned()
}

/// Decode the path and query of a URL while leaving the scheme + authority
/// untouched.  Useful for displaying user-facing URLs whose `path` is
/// percent-encoded Persian.
///
/// If `url` does not contain `://`, the entire string is decoded.
///
/// ```
/// use parsitext::url_fix::fix;
///
/// assert_eq!(
///     fix("https://fa.wikipedia.org/wiki/%D8%A7%DB%8C%D8%B1%D8%A7%D9%86"),
///     "https://fa.wikipedia.org/wiki/ایران"
/// );
/// ```
#[must_use]
pub fn fix(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let after = &url[scheme_end + 3..];
        // Authority ends at the first `/` `?` `#` (or end of string).
        let auth_end = after.find(['/', '?', '#']).unwrap_or(after.len());
        let scheme_auth = &url[..scheme_end + 3 + auth_end];
        let rest = &url[scheme_end + 3 + auth_end..];
        format!("{scheme_auth}{}", decode(rest))
    } else {
        decode(url)
    }
}

#[inline]
fn hex_nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'A' + n - 10) as char,
        _ => '?',
    }
}

#[inline]
fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_persian() {
        assert_eq!(encode("سلام"), "%D8%B3%D9%84%D8%A7%D9%85");
    }

    #[test]
    fn encode_keeps_unreserved() {
        assert_eq!(encode("a-b_c.d~e0"), "a-b_c.d~e0");
    }

    #[test]
    fn encode_space_and_punct() {
        assert_eq!(encode(" "), "%20");
        assert_eq!(encode("/"), "%2F");
    }

    #[test]
    fn round_trip() {
        for s in ["سلام", "ایران", "tehran-طهران-2024", "مدرسه ای"] {
            assert_eq!(decode(&encode(s)), s);
        }
    }

    #[test]
    fn decode_passes_through_stray_percent() {
        assert_eq!(decode("100%"), "100%");
        assert_eq!(decode("%ZZ"), "%ZZ");
    }

    #[test]
    fn decode_lowercase_hex() {
        assert_eq!(decode("%d8%a7"), "ا");
    }

    #[test]
    fn fix_keeps_authority() {
        assert_eq!(
            fix("https://fa.wikipedia.org/wiki/%D8%A7%DB%8C%D8%B1%D8%A7%D9%86"),
            "https://fa.wikipedia.org/wiki/ایران"
        );
    }

    #[test]
    fn fix_query_string() {
        assert_eq!(
            fix("https://x.com/?q=%D8%B3%D9%84%D8%A7%D9%85"),
            "https://x.com/?q=سلام"
        );
    }

    #[test]
    fn fix_no_scheme() {
        assert_eq!(fix("/wiki/%D8%A7%DB%8C%D8%B1%D8%A7%D9%86"), "/wiki/ایران");
    }
}
