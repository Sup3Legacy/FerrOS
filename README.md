# FerrOS

## Build status : ![Rust](https://github.com/Sup3Legacy/FerrOS/workflows/Rust/badge.svg)

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
- Build and open documentation: `make doc`
- Build: `make build`
- Build and run: `make run`
- Clean: `make clean`
