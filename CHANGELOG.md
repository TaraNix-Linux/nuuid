# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

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
[Unreleased]: https://github.com/DianaNites/nuuid/compare/v0.3.2...HEAD
[0.3.2]: https://github.com/DianaNites/nuuid/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/DianaNites/nuuid/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/DianaNites/nuuid/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/DianaNites/nuuid/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/DianaNites/nuuid/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/DianaNites/nuuid/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/DianaNites/nuuid/releases/tag/v0.1.0
