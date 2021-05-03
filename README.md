# Nuuid

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg)](https://github.com/RichardLitt/standard-readme)
[![nuuid crates.io version and link](https://img.shields.io/crates/v/nuuid.svg)](https://crates.io/crates/nuuid)
![nuuid Crates.io license](https://img.shields.io/crates/l/nuuid)
[![nuuid docs.rs badge](https://docs.rs/nuuid/badge.svg)](https://docs.rs/nuuid)

A New Uuid(nuuid) library for Rust

A `no_std` library to create and use RFC 4122 UUID's in Rust.

## Security

UUID's can be used without requiring a central authority,
but are not, strictly speaking, guaranteed to be unique, collisions may be possible.

Do not assume they are hard to guess, they should not be used as security capabilities.

Do not assume people can tell if they've been altered at a glance. They can't.

## Install

```toml
[dependencies]
nuuid = "0.3.0"
```

`no_std` support:

```toml
[dependencies]
nuuid = { version = "0.3.0", default-features = false }
```

### Dependencies

Depends on [`getrandom`](https://crates.io/crates/getrandom) by default,
which is `no_std` but, depending on target, requires OS system libraries.

This crate is only tested on the latest *stable* Rust.

## Usage

See the documentation for details

## Changelog

Please see [CHANGELOG](CHANGELOG.md) for version history

## See Also

The other [uuid](https://crates.io/crates/uuid) crate.

## Contributing

Feel free to ask questions on the [Github repo](https://github.com/DianaNites/uuid).

[See CONTRIBUTING.md](CONTRIBUTING.md) for details on code contributions.

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.
