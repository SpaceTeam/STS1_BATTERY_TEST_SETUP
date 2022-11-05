# Info

The project is configured to be used with a NUCLEO-F411RE Board. If you use a different one, you might have to adjust a few settings in `.cargo/config`, `memory.x` and `Cargo.toml`

# Prerequisites

cargo-flash

	> cargo install cargo-flash

install the build target `thumbv7em-none-eabihf` for your toolchain

# Build & Run

You can use `cargo build` and `cargo check` as usual.

To upload it to the board use:

	> cargo flash --chip STM32F411RETx --release

To get a list of all the available chips use:

	> cargo flash --list-chips