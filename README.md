# BL602 Wifi Rust

This is work in progress and currently more a proof of concept.
The code is awfully hacked together - just enough to make it work.

It uses the NuttX wifi blob from https://github.com/bouffalolab/bl_blob

## Status

This connects to an access point and provides a minimal TCP server on port 4321.
You can ping the BL602 and telnet to port 4321.

Sometimes it fails to connect (there is no retry currently) and it might panic.

## Implementation Notes

This needs some modifications to the following crates (done in my forks referenced in `Cargo.toml`)
- _riscv_ - needs support for ILP32F
- _riscv-rt_ - needs support for ILP32F and initialization of the FPU

Also it needs a very special linker script.

It uses one of the timers which can't be used for other things.

## Get Started

In `examples/simple/wifi_config.rs` change the SSID and PSK for your access point. 

Maybe you need to change the IP address (currently 192.168.2.191) and the IP of the default gateway (192.168.2.1) in `examples/simple/main.rs`.

Compile with `cargo build -Z build-std --target riscv32imfc-unknown-none-elf.json --example simple` and flash the resulting binary.

## Things to change

- [ ] especially the code in `compat` needs rework
- [ ] make it more stable
- [ ] use a queue for tx
- [ ] update to latest blobs (for me they can not connect to all APs currently)

and many more ...
