# Uuid

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

Uuid library for Rust

A `no_std` library to create and use RFC 4122 UUID's in Rust.

## Security

UUID's can be used without requiring a central authority,
but are not, strictly speaking, guaranteed to be unique, collisions may be possible.

Do not assume they are hard to guess, they should not be used as security capabilities.

Do not assume people can tell if they've been altered at a glance. They can't.

## Install

```toml
[dependencies]
uuid = "0.1.0"
```

### Dependencies

Depends on [`getrandom`](https://crates.io/crates/getrandom) by default,
which is `no_std` but, depending on target, requires OS system libraries.

## Usage

<!-- TODO: Usage -->

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
