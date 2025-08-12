# Embedded Terminal

[![Crate](https://img.shields.io/crates/v/embedded-term.svg)](https://crates.io/crates/embedded-term)
[![Docs](https://docs.rs/embedded-term/badge.svg)](https://docs.rs/embedded-term)
[![Actions Status](https://github.com/rcore-os/embedded-term/workflows/CI/badge.svg)](https://github.com/rcore-os/embedded-term/actions)

A terminal emulator on [embedded-graphics][].

This crate is `no_std` compatible. It is suitable for embedded systems and OS kernels.

[embedded-graphics]: https://github.com/embedded-graphics/embedded-graphics

## Run example

1. rterm

Read and show stdin:

```
htop | cargo run --example rterm
```

2. pty

Spawn a process and show:

```
cargo run --example pty htop
```

TODO: documents and tests

## Optional features

- `log`: Enable built-in logging.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
