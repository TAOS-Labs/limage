# Limage: Limine Boot Imager

Limage is a command line tool designed to assist in building a Rust-based kernel with the Limine bootloader. This tool automates the process of downloading necessary files, copying them to the appropriate directories, building the kernel, and creating an ISO image for booting.

Importantly, allows support for `cargo run` and `cargo test` to execute through QEMU.

Inspired by, and partially derived from, the popular [bootimage](https://crates.io/crates/bootimage) crate.

## Basic Usage

**Installation:** `cargo install limage`

**Build:** `limage`

**Run (QEMU):** `cargo run`

**Test (QEMU):** `cargo test`

**Delete Image:** `cargo clean`

NOTE: `run` and `test` commands will always build before their execution.

## Prerequisites
- **Linux:** Required for building the Limine bootloader. WSL for Windows is compatible (tested with MSYS2).
- **Xorriso:** Required for building the *.iso file.
- **Git:** Required for cloning the Limine bootloader repository.
- **Curl:** Required for downloading architecture-specific OVMF files.
- **QEMU:** Required for running the kernel in a virtual environment.

Apologies for so many dependencies; there is a priority to remove these in later versions.

## Features

- Downloads OVMF files required for UEFI booting.
- Copies necessary files into the correct directories for Limine.
- Builds the kernel using Cargo.
- Creates an ISO image for easy booting.
- Includes a Qemu runner for compatibility with `cargo run`

## Supported Architectures

- [❌] X86
- [✔️] X86-64
- [❌] aarch64
- [❌] riscv64

Compatibility with more Limine-supported architectures is planned.

## Installation

To use Limage, ensure you have Rust and Cargo installed on your system. You can install Rust by following the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

Install the command line tool through Cargo:

```
cargo install limage
```

## Usage

### Configuration

Add the following to your .cargo/config.toml to allow `cargo run` support:

```
[unstable]
build-std = ["core", "compiler_builtins", "alloc"]
build-std-features = ["compiler-builtins-mem"]

[build]
target = "<your-target-json>.json"

[target.'cfg(target_os = "none")']
# Limage compatibility with `cargo run`
runner = "limage runner"
# Required for Cargo to pass the correct flags to the linker before running `limage runner`
rustflags = ["-C", "relocation-model=static", "-C", "link-arg=<your-linker>.ld", "-C", "code-model=kernel"]
```

For testing through `cargo test`, add the following configuration to your Cargo.toml:

```
[package.metadata.limage]
test-success-exit-code = 33 # (0x10 << 1) | 1
test-args = ["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-serial", "stdio", "-display", "none"]
```

Also for testing, your kernel should be configured to use the `custom_test_frameworks` feature. For the best experience, your main.rs and lib.rs should both be configured to use this feature. In an effort to keep this README succinct, please refer to the [lib.rs](https://github.com/phillipg14/limage/blob/main/example/src/lib.rs) and [main.rs](https://github.com/phillipg14/limage/blob/main/example/src/main.rs) of the example kernel.

### Build

Run the command line tool with the following command:

```
cargo limage
```

This will initiate the kernel building process. You may need to provide additional command-line arguments depending on your specific requirements.

### Run (QEMU)

Run the following command:

```
cargo run
```

This will build the kernel and launch through QEMU.

### Test (QEMU)

```
cargo test
```

This will build the kernel with a test profile, then launch through QEMU. All functions marked with `#[test_case]` will be automatically executed.

**NOTE:** Your kernel project must be configured to use the feature of Rust: `custom_test_frameworks`

## Coming Soon

- More architecture support, starting with aarch64
- Reduction of dependencies (Xorriso, Curl, Git)
- More configuration options
- Bug fixes :-)

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes. Help with support for non-x86 architectures is especially appreciated!

## License

This project is licensed under the MIT License. See the LICENSE file for details.
