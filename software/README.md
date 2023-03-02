 Info

The project is configured to be used with a NUCLEO-F411RE Board. If you use a different one, you might have to adjust a few settings in `.cargo/config`, `memory.x` and `Cargo.toml`

This workspace has the following four members:

1. `board` This is the binary that will run on the nucleo-board. It provides all the low level functionality, like reading/writing GPIO-Pins, UART, I2C and so on for `firmware`.

2. `client` This will be the program running on your computer to set up tests and receive the data.

3. `firmware` This will be called by `board`. It implements all the logic for testing the batteries. `firmware` will not talk to the hardware directly. There is another layer of abstraction in between, that way we can easily test all the functionality on the developer's machine.

4. `transmission` This library will implement the protocol used for communication between the `client` and the `board`.

Except for the unit-tests `client`, `firmware` and `transmission` will have to be no_std

# Prerequisites

1) cargo-flash

    See the [install guide](https://crates.io/crates/cargo-flash#installation) for your OS


2) install the build target `thumbv7em-none-eabihf`

        $ rustup target add thumbv7em-none-eabihf

3) if you get an error flashing it, you also need the STM Drivers: https://probe.rs/docs/getting-started/probe-setup

# Build & Run

All aliases you should need are defined in `.cargo/config.toml`.

To flash the code onto the board, all you should need to do is connect the board to your pc and run:

    $ cargo run-board

Info: On my machine, the build sometimes gets stuck. If that happens, kill the process and try it again. Normally after a few tries it works :D

To run the client

    $ cargo run-client

To just build them without flashing/executing

    $ cargo build-board
    $ cargo build-client

To run the tests

    $ cargo test-all
    $ cargo test-client
    $ cargo test-firmware
    $ cargo test-transmission
