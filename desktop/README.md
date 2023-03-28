# Arduino-as-GPIO Desktop App

this is the desktop app part of Arduino-as-GPIO. It is written in Rust.
Since it uses the [serialport](https://crates.io/crates/serialport) crate, it should work on both Windows and Linux.

## Building

to build the desktop app, you need to have the Rust toolchain installed. You can install it by following the instructions on the [Rust website](https://www.rust-lang.org/tools/install).

after you have installed the Rust toolchain, you can build the desktop app by running the following command in the root directory of this repository:

```bash
$ cargo build --release
```

this will create a binary in the `target/release` directory.
