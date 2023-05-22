# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- `Version::Reserved`

### Changed

- `Uuid::version` now returns `Version::Reserved` instead of `Version::Nil`
- `Uuid::from_str` and `Uuid::to_str` are now significantly faster

### Breaking

- `Uuid::version` now returns `Version::Reserved` instead of `Version::Nil`

## [0.4.7] - 2023-05-22

- Yanked for backwards compatibility

## [0.4.6] - 2022-08-04

### Changed

- Fixed github link in Cargo.toml

## [0.4.5] - 2022-08-04

### Changed

- Documentation details
- Docs rs config
- Updated README

## [0.4.4] - 2022-08-04

## [0.4.3] - 2022-08-04

## [0.4.2] - 2022-08-04

### Added

- `Uuid::timestamp`, `Uuid::clock_sequence`, `Uuid::node`
- UUID Version 1 support, `Uuid::new_v1`
- Experimental UUID Version 6, 7, and 8 support, `Uuid::new_v6|7|8`, behind the `experimental_uuid` cargo feature
- impl `Display` on `Version` and `Variant`

### Changed

- `Uuid` alternate `Debug` representation now includes version number
- Clarified `Uuid::version` documentation

### Fixed

- `Uuid` `Debug` representation of `Version`
- `Uuid` `Debug` representation on `no_std`
- `Uuid` `Debug` representation in general..

## [0.4.1] - 2022-08-03

### Added

- `Uuid::parse_me`, to parse mixed-endian UUID strings.

### Changed

- Documented how `Uuid::from_bytes_me` works better
- Documented `Uuid` `Debug` representation
- `Uuid::version`, `Uuid::variant`, `Uuid::from_bytes_me`, `Uuid::is_nil`, and `Uuid::to_bytes_me` are now `const fn`

## [0.4.0] - 2022-08-02

### Added

- `Uuid::parse` now supports braced UUIDs and hyphen-less UUIDs

### Changed

- Improved Performance
- Dependencies updated
- Documentation improved
- `Uuid::version` now returns `Version::Nil` for invalid versions
- `rand` dependency replaced with `rand_chacha`
- `Variant` and `Version` are now `non_exhaustive`

### Removed

- `Version::Invalid`

## [0.3.2] - 2021-12-08

## [0.3.1] - 2021-05-02

### Changed

- Updated RustCrypto crates to 0.10.0

### Fixed

- Minor typo in README.

## [0.3.0] - 2021-05-02

### Changed

- Removed bitvec as a dependency
- `Uuid::to_(str|urn)(_upper)` now take arrays as arguments, not slices.

### Fixed

- Documentation typos

### Removed

- `impl From<[u8; 16]> for Uuid`

## [0.2.1] - 2021-03-04

## [0.2.0] - 2021-03-04

### Changed

- `Uuid::version` no longer panics, instead returns `Version::Invalid`
- Improved documentation
- Updated dependencies

## [0.1.1] - 2020-12-06

### Changed

- Typos, readme

## [0.1.0] - 2020-12-06

### Added

- Initial release
- no_std UUID's

<!-- next-url -->
[Unreleased]: https://github.com/DianaNites/nuuid/compare/v0.4.7...HEAD
[0.4.7]: https://github.com/DianaNites/nuuid/compare/v0.4.6...v0.4.7
[0.4.6]: https://github.com/DianaNites/nuuid/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/DianaNites/nuuid/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/DianaNites/nuuid/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/DianaNites/nuuid/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/DianaNites/nuuid/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/DianaNites/nuuid/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/DianaNites/nuuid/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/DianaNites/nuuid/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/DianaNites/nuuid/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/DianaNites/nuuid/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/DianaNites/nuuid/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/DianaNites/nuuid/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/DianaNites/nuuid/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/DianaNites/nuuid/releases/tag/v0.1.0
