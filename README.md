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
- Build (release): `make`
- Build (release) and run: `make run`
- Build release: `make ferros_release`
- Build without optimizations: `make ferros`
- Build and open documentation: `make doc`
- Clean: `make clean`
- Format: `make fmt`
- Count the number of lines: `make count`
