# zigate-rs

[WIP] Rust driver for the PiZiGate.

## Build

To build for Raspberry Pi Zero, you need the Raspberry Pi tools:

    git clone https://github.com/raspberrypi/tools.git raspberrypi-tools

Then add the following to zigate-rs/.cargo/config:

    [target.arm-unknown-linux-gnueabihf]
    linker = "~/raspberrypi-tools/arm-bcm2708/arm-linux-gnueabihf/bin/arm-linux-gnueabihf-gcc"

And build with:

    cargo build --target=arm-unknown-linux-gnueabihf --release
