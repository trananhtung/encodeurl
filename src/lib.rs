//! # encodeurl — encode a URL to a percent-encoded form, preserving existing escapes
//!
//! Encodes all the non-URL code points in a string while **leaving already-encoded
//! sequences intact**: `%20` stays `%20`, but a stray `%` (e.g. `%foo`) becomes `%25foo`.
//! This is the "encode a URL you were given, without double-encoding it" operation.
//!
//! A faithful Rust port of the widely-used [`encodeurl`](https://www.npmjs.com/package/encodeurl)
//! npm package (v2), used by Express, `send`, and `serve-static`. Safe (never panics),
//! **zero dependencies**, and `#![no_std]` (needs only `alloc`).
//!
//! ```
//! use encodeurl::encode_url;
//!
//! assert_eq!(encode_url("http://example.com/foo bar"), "http://example.com/foo%20bar");
//! assert_eq!(encode_url("/path?q=café"), "/path?q=caf%C3%A9");
//! assert_eq!(encode_url("%20already%20encoded"), "%20already%20encoded"); // kept as-is
//! assert_eq!(encode_url("100%done"), "100%25done"); // stray % is escaped
//! ```
//!
//! The return type is [`Cow`], so an input that needs no changes is returned without any
//! allocation:
//!
//! ```
//! use std::borrow::Cow;
//! use encodeurl::encode_url;
//!
//! assert!(matches!(encode_url("/already/clean?x=1"), Cow::Borrowed(_)));
//! ```
//!
//! ## What is and isn't encoded
//!
//! The characters left unencoded are the URL-significant set
//! `! # $ % & ' ( ) * + , - . / 0-9 : ; = ? @ A-Z [ \ ] ^ _ a-z | ~`. Everything else —
//! spaces, quotes, angle brackets, braces, backticks, control characters, and all
//! non-ASCII — is percent-encoded using UTF-8 (uppercase hex), exactly as JavaScript's
//! `encodeURI` would. A `%` is left
//! alone only when it begins a valid two-hex-digit escape; otherwise it is encoded to
//! `%25`.

#![no_std]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/encodeurl/0.1.0")]

extern crate alloc;

use alloc::borrow::Cow;
use alloc::string::String;

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// Encode a URL to a percent-encoded form, excluding already-encoded sequences.
///
/// Takes a (possibly already partly-encoded) URL and percent-encodes every code point
/// that is not allowed in a URL, while leaving valid `%XX` escapes untouched. It never
/// fails: any input produces a valid output.
///
/// The result borrows the input unchanged when no encoding is necessary (see [`Cow`]).
///
/// ```
/// use encodeurl::encode_url;
///
/// assert_eq!(encode_url("foo bar"), "foo%20bar");
/// assert_eq!(encode_url("a%2Fb"), "a%2Fb");   // valid escape preserved
/// assert_eq!(encode_url("a%2gb"), "a%252gb"); // invalid escape: % encoded
/// ```
#[must_use]
pub fn encode_url(url: &str) -> Cow<'_, str> {
    let bytes = url.as_bytes();
    let mut out: Option<String> = None;
    let mut idx = 0;

    while idx < bytes.len() {
        let end = run_end(url, idx);
        if end > idx {
            // `url[idx..end]` is a maximal run of code points to encode (and/or invalid
            // `%` sequences); pass the whole run through `encodeURI` semantics.
            let acc = out.get_or_insert_with(|| prefix_string(url, idx));
            encode_uri_into(&url[idx..end], acc);
            idx = end;
        } else {
            // A character that is left verbatim.
            let len = char_len(url, idx);
            if let Some(acc) = out.as_mut() {
                acc.push_str(&url[idx..idx + len]);
            }
            idx += len;
        }
    }

    match out {
        Some(s) => Cow::Owned(s),
        None => Cow::Borrowed(url),
    }
}

/// Seed an owned output buffer with the verbatim prefix `url[..idx]`.
fn prefix_string(url: &str, idx: usize) -> String {
    let mut s = String::with_capacity(url.len() + 8);
    s.push_str(&url[..idx]);
    s
}

/// Byte length of the UTF-8 character starting at byte `pos` (0 if past the end).
fn char_len(url: &str, pos: usize) -> usize {
    url[pos..].chars().next().map_or(0, char::len_utf8)
}

/// Find the end (exclusive byte index) of a maximal "to-encode" run starting at `start`.
///
/// Mirrors the reference regex
/// `(?:[^safe] | %(?:[^hex] | [hex][^hex] | $))+`, where a run consists of unsafe code
/// points and invalid `%` escapes. Returns `start` itself when the character at `start`
/// is left verbatim (a safe character, or a `%` that begins a valid escape).
fn run_end(url: &str, start: usize) -> usize {
    let bytes = url.as_bytes();
    let mut pos = start;

    while pos < bytes.len() {
        let b = bytes[pos];
        if b == b'%' {
            let b1 = bytes.get(pos + 1).copied();
            let b2 = bytes.get(pos + 2).copied();
            let b1_hex = b1.is_some_and(|x| x.is_ascii_hexdigit());
            let b2_hex = b2.is_some_and(|x| x.is_ascii_hexdigit());

            // Valid `%XX`, or an incomplete `%X` at end-of-string: the `%` is a safe
            // character, so the run stops here.
            if b1_hex && (b2.is_none() || b2_hex) {
                break;
            }

            // Otherwise this is an invalid `%` escape that belongs to the run.
            match b1 {
                None => pos += 1, // `%` at end
                Some(x) if x.is_ascii_hexdigit() => {
                    pos += 2; // `%` + hex digit ...
                    pos += char_len(url, pos); // ... + the trailing non-hex character
                }
                Some(_) => {
                    pos += 1; // `%` ...
                    pos += char_len(url, pos); // ... + the trailing non-hex character
                }
            }
            continue;
        }

        if b < 0x80 && is_url_safe(b) {
            break; // safe ASCII (not `%`) → run stops
        }

        // Any other character (unsafe ASCII or non-ASCII) is encoded.
        pos += char_len(url, pos);
    }

    pos
}

/// Apply `encodeURI` semantics to `run`, appending into `out`.
fn encode_uri_into(run: &str, out: &mut String) {
    for ch in run.chars() {
        if ch.is_ascii() && is_encode_uri_safe(ch as u8) {
            out.push(ch);
        } else {
            let mut buf = [0u8; 4];
            for &byte in ch.encode_utf8(&mut buf).as_bytes() {
                push_percent(out, byte);
            }
        }
    }
}

/// Append `%XX` (uppercase hex) for a single byte.
fn push_percent(out: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(HEX[(byte >> 4) as usize] as char);
    out.push(HEX[(byte & 0x0f) as usize] as char);
}

/// Characters left unencoded by `encodeurl`: `! # $ % & ' ( ) * + , - . / 0-9 : ; = ? @
/// A-Z [ \ ] ^ _ a-z | ~`.
fn is_url_safe(b: u8) -> bool {
    matches!(b,
        0x21 | 0x23..=0x3B | 0x3D | 0x3F..=0x5F | 0x61..=0x7A | 0x7C | 0x7E)
}

/// Characters left unencoded by JavaScript's `encodeURI`.
fn is_encode_uri_safe(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b'-'
                | b'.'
                | b'/'
                | b':'
                | b';'
                | b'='
                | b'?'
                | b'@'
                | b'_'
                | b'~'
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::borrow::Cow;

    fn enc(s: &str) -> String {
        encode_url(s).into_owned()
    }

    #[test]
    fn encodes_spaces_and_non_ascii() {
        assert_eq!(
            enc("http://example.com/foo bar"),
            "http://example.com/foo%20bar"
        );
        assert_eq!(enc("/path?q=café"), "/path?q=caf%C3%A9");
        assert_eq!(enc("π"), "%CF%80");
        assert_eq!(enc("😀"), "%F0%9F%98%80");
    }

    #[test]
    fn preserves_valid_escapes() {
        assert_eq!(enc("%20"), "%20");
        assert_eq!(enc("%3c%3C"), "%3c%3C"); // case preserved, not normalized
        assert_eq!(enc("a%2Fb"), "a%2Fb");
    }

    #[test]
    fn escapes_invalid_percent() {
        assert_eq!(enc("%G"), "%25G");
        assert_eq!(enc("%2G"), "%252G");
        assert_eq!(enc("%"), "%25");
        assert_eq!(enc("100%done"), "100%25done");
        assert_eq!(enc("%zz%41%4"), "%25zz%41%4");
    }

    #[test]
    fn incomplete_escape_at_end_is_kept() {
        // `%X` at end-of-string is not a matched invalid escape, so `%` is kept.
        assert_eq!(enc("%1"), "%1");
        assert_eq!(enc("a%4"), "a%4");
    }

    #[test]
    fn run_consumption_edge_cases() {
        assert_eq!(enc("%%20"), "%25%2520");
        assert_eq!(enc("%["), "%25%5B");
        assert_eq!(enc("%é"), "%25%C3%A9");
        assert_eq!(enc("%😀"), "%25%F0%9F%98%80");
    }

    #[test]
    fn keeps_url_significant_chars() {
        let s = "/path/[id]/?a=1&b=2#frag|x^y\\z";
        assert_eq!(enc(s), s);
    }

    #[test]
    fn encodes_unsafe_ascii() {
        assert_eq!(enc("a<b>c\"d`e{f}g"), "a%3Cb%3Ec%22d%60e%7Bf%7Dg");
    }

    #[test]
    fn borrows_when_unchanged() {
        assert!(matches!(encode_url("/already/clean?x=1"), Cow::Borrowed(_)));
        assert!(matches!(encode_url("foo bar"), Cow::Owned(_)));
        assert!(matches!(encode_url(""), Cow::Borrowed(_)));
    }
}
