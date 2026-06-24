# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0]

### Added

- Initial release: `encode_url(&str) -> Cow<str>` — a faithful, safe, zero-dependency,
  `no_std` port of the `encodeurl` npm package (v2).
- Percent-encodes non-URL code points with `encodeURI` semantics (UTF-8, uppercase hex),
  preserves valid `%XX` escapes, and encodes stray `%` to `%25`.
- Borrows the input (no allocation) when no encoding is necessary.
