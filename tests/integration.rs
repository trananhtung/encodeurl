//! Integration tests exercising the public API of `encodeurl`.

use encodeurl::encode_url;

#[test]
fn encodes_a_realistic_url() {
    assert_eq!(
        encode_url("http://example.com/path with spaces/é?name=John Doe&q=π"),
        "http://example.com/path%20with%20spaces/%C3%A9?name=John%20Doe&q=%CF%80"
    );
}

#[test]
fn does_not_double_encode() {
    // An already-encoded URL passes through unchanged.
    let already = "http://example.com/%E2%9C%93/path?x=%20";
    assert_eq!(encode_url(already), already);
}

#[test]
fn escapes_stray_percent_but_keeps_valid_escapes() {
    assert_eq!(
        encode_url("50%25 off + %20 = %2g"),
        "50%25%20off%20+%20%20%20=%20%252g"
    );
}

#[test]
fn keeps_url_structure_intact() {
    let url = "https://user:pass@host:8080/a/b/c?d=1&e=2#frag";
    assert_eq!(encode_url(url), url);
}

#[test]
fn control_characters_are_encoded() {
    assert_eq!(encode_url("a\tb\nc"), "a%09b%0Ac");
}

#[test]
fn empty_input() {
    assert_eq!(encode_url(""), "");
}
