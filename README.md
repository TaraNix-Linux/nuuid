# Nuuid

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg)](https://github.com/RichardLitt/standard-readme)
[![nuuid crates.io version and link](https://img.shields.io/crates/v/nuuid.svg)](https://crates.io/crates/nuuid)
![nuuid Crates.io license](https://img.shields.io/crates/l/nuuid)
[![nuuid docs.rs badge](https://docs.rs/nuuid/badge.svg)](https://docs.rs/nuuid)

A New Uuid(nuuid) library for Rust

A `no_std` library to create and use UUID's in pure Rust.

## Specifications

This library follows [RFC 9562] (formerly: [RFC 4122]), with the following errata taken note of

- N/A

### Why RFC 9562?

Despite its current at time of writing status as a "proposed standard", other standards bodies are also adopting it as the canonical reference for what a "UUID" is.
It is compatible with the old [RFC 4122], but with clearer text and more versions defined.
As such, as such we also treat it as the canonical reference.

## Features / Design Goals

- `no_std` and no `alloc`
- Pure Rust
- Strict compliance to the RFC
  - Reasoning and justifications should be explained in documentation and/or source comments
- Easy to use correctly
- Fast
- Small memory and stack footprint
- Support for zero-copy reading, writing, and modification
- No Panics
- No Overflows

## Usage

See the documentation for details

## Changelog

Please see [CHANGELOG](CHANGELOG.md) for version history

## See Also

The other [uuid](https://crates.io/crates/uuid) crate.

## Contributing

Feel free to ask questions on the [Github repo](https://github.com/TaraNix-Linux/uuid).

[See CONTRIBUTING.md](CONTRIBUTING.md) for details on code contributions.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.

[RFC 4122]: https://www.rfc-editor.org/rfc/rfc4122
[RFC 9562]: https://www.rfc-editor.org/info/rfc9562
