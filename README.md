# encodeurl

[![crates.io](https://img.shields.io/crates/v/encodeurl.svg)](https://crates.io/crates/encodeurl)
[![docs.rs](https://docs.rs/encodeurl/badge.svg)](https://docs.rs/encodeurl)
[![CI](https://github.com/trananhtung/encodeurl/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/encodeurl/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/encodeurl.svg)](#license)

**Encode a URL to a percent-encoded form — without double-encoding it.**

`encodeurl` percent-encodes every non-URL code point in a string while **leaving
already-encoded sequences intact**: `%20` stays `%20`, but a stray `%` (as in `%foo`)
becomes `%25foo`. It is the right tool for safely encoding a URL you were *handed* (e.g. a
`Location` header or a request target) that may already be partially encoded.

It is a faithful Rust port of the widely-used
[`encodeurl`](https://www.npmjs.com/package/encodeurl) npm package (v2) — used by Express,
`send`, and `serve-static` — which has no Rust equivalent.

- **Safe** — never panics, always produces valid output
- **Zero dependencies**
- **`#![no_std]`** (needs only `alloc`)
- Returns `Cow<str>` — no allocation when the input is already clean
- Differential-tested against the reference `encodeurl` implementation

## Install

```toml
[dependencies]
encodeurl = "0.1"
```

## Usage

```rust
use encodeurl::encode_url;

// Encode unsafe characters and non-ASCII.
assert_eq!(encode_url("http://example.com/foo bar"), "http://example.com/foo%20bar");
assert_eq!(encode_url("/search?q=café"), "/search?q=caf%C3%A9");

// Already-encoded input is not double-encoded.
let already = "http://example.com/%E2%9C%93/path?x=%20";
assert_eq!(encode_url(already), already);

// A stray `%` is escaped, but valid `%XX` escapes are preserved.
assert_eq!(encode_url("100%done"), "100%25done");
assert_eq!(encode_url("a%2Fb"), "a%2Fb");
```

Because the result is a [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html), an
input that needs no changes is returned without allocating:

```rust
use std::borrow::Cow;
use encodeurl::encode_url;

assert!(matches!(encode_url("/already/clean?x=1"), Cow::Borrowed(_)));
```

## What is and isn't encoded

The characters left **unencoded** are the URL-significant set:

```text
! # $ % & ' ( ) * + , - . / 0-9 : ; = ? @ A-Z [ \ ] ^ _ a-z | ~
```

Everything else — spaces, `"`, `<`, `>`, `` ` ``, `{`, `}`, control characters, and all
non-ASCII — is percent-encoded using UTF-8 (uppercase hex), exactly as JavaScript's
`encodeURI` would. A `%` is left alone only when it begins a valid two-hex-digit escape;
otherwise it is encoded to `%25`.

This crate does **not** decode, and it does not encode the URL-significant characters above
— so it is *not* a replacement for component-level encoding (e.g. `encodeURIComponent`).
Use it to make a whole, possibly-already-encoded URL safe to emit.

## Note on surrogates

The npm package replaces unpaired UTF-16 surrogates with the Unicode replacement character
before encoding. A Rust `&str` is always well-formed UTF-8 and cannot contain unpaired
surrogates, so that step is unnecessary here and the behavior is identical for every valid
input.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
