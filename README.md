# BL602 Wifi Rust

This is work in progress and currently more a proof of concept.
The code is awfully hacked together - just enough to make it work. However it seems to work so far for me.

It uses the NuttX wifi blob from https://github.com/bouffalolab/bl_blob

## Status

|What|Status|
|---|---|
|Wifi Connect to Access Point|Works|
|Wifi Scan|Works|
|Wifi Start Access Point|Doesn't work|
|Wifi Automatic Reconnect|Not implemented yet|
|BLE|HCI Communication works and there is a minimal BLE stack|

## Examples

See the `examples` folder.

### Example: _simple_

This connects to an access point and provides a minimal TCP server on port 4321.
You can ping the BL602 and telnet to port 4321.

Sometimes it fails to connect (there is no retry currently).

In `examples/simple/wifi_config.rs` change the SSID and PSK for your access point. 

Maybe you need to change the IP address (currently 192.168.2.191) and the IP of the default gateway (192.168.2.1) in `examples/simple/main.rs`.

Compile with `cargo build -Z build-std --target riscv32imfc-unknown-none-elf.json --example simple` and flash the resulting binary.

### Example: _scan_

This scans for wifi networks.

### Example: _dhcp_

Similar to _simple_ but uses DHCP - so no need to configure an IP address or gateway.

### Example: _ble_advertising_

This starts BLE advertising. It should show up as _BL-602 BLE_ when scanning for Bluetooth devices.
It's possible to connect to it and discover two services. One is read and writeable and one is just writeable. Read and write should also work.

## Implementation Notes

This needs some modifications to the following crates (done in my forks referenced in `Cargo.toml`)
- _riscv-rt_ - needs support for ILP32F and initialization of the FPU

Also it needs a very special linker script.

It uses one of the timers which can't be used for other things.

You'll need nightly Rust. (I use `rustc 1.57.0-nightly (9bb77da74 2021-09-13)` currently)

## Things to change / improve

- [ ] especially the code in `compat` can be improved
- [ ] make it more stable
- [ ] use a queue for tx for improved performance
- [ ] implement more BLE stack functionality

and many more ...
