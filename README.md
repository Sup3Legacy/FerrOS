# OS-test

## Dependencies
- cargo
- rustup
- qemu

## Installation
Install the dependencies.

Then run these three commands in the root directory of the project:
- `rustup override set nightly`
- `cargo install bootimage`
- `rustup component add llvm-tools-preview`
- `rustup component add rust-src`

## Usage
- Build and open documentation: `cargo doc --open`
- Build: `cargo build`
- Build and run: `cargo run`
- Clean: `cargo clean`
