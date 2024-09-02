# dump1090_rs
[<img alt="github" src="https://img.shields.io/badge/github-rsadsb/dump1090_rs-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/rsadsb/dump1090_rs)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/rsadsb/dump1090_rs/main.yml?branch=master&style=for-the-badge" height="20">](https://github.com/rsadsb/dump1090_rs/actions?query=branch%3Amaster)

Demodulate a ADS-B signal from a software defined radio device tuned at 1090mhz and
forward the bytes to applications such as [adsb_deku/radar](https://github.com/rsadsb/adsb_deku).

- See [quickstart-guide](https://rsadsb.github.io/quickstart.html) for a quick installation guide.
- See [rsadsb-2023.11.22](https://rsadsb.github.io/2023.11.22.html) for latest major release details.

## Tested Support

Through the use of the [rust-soapysdr](https://github.com/kevinmehall/rust-soapysdr) project,
we support [many different](https://github.com/pothosware/SoapySDR/wiki) software defined radio devices.
If you have tested this project on devices not listed below, let me know!
(you will need to add gain settings to [config.toml](dump1090_rs/config.toml) or use `--custom-config`)

| Device                | Supported/Tested | Recommend | argument          |
| --------------------- | :--------------: | :-------: | ----------------- |
| rtlsdr                |        x         |     x     | `--driver rtlsdr` |
| HackRF                |        x         |           | `--driver hackrf` |
| uhd(USRP)             |        x         |           | `--driver uhd`    |
| bladeRF 2.0 micro xA4 | x                |           | `--driver bladerf`|


## Usage
**Minimum Supported Rust Version**: 1.74.0

## Build

Install `soapysdr` drivers and library and `libclang-dev`.

### Note
Using `debug` builds will result in SDR overflows, always using `--release` for production.

### Ubuntu
```
> apt install libsoapysdr-dev libclang-dev
```

### Cross Compile
Use [hub.docker.com/r/rsadsb](https://hub.docker.com/r/rsadsb/ci/tags) for cross compiling to the following archs.
These images already have `soapysdr` installed with the correct cross compilers.
This uses [cross-rs](https://github.com/cross-rs/cross).
```
> cargo install cross
> cross build --workspace --target x86_64-unknown-linux-gnu --relese

# Used for example in Raspberry Pi (raspios) 32 bit
> cross build --workspace --target armv7-unknown-linux-gnueabihf --release

# Used for example in Raspberry Pi (raspios) 64 bit
> cross build --workspace --target aarm64-unknown-linux-gnu --release
```

### Release Builds from CI
Check the [latest release](https://github.com/rsadsb/dump1090_rs/releases) for binaries built from the CI.

## Run
Run the software using the default rtlsdr.
```
> cargo r --release
```

### help

See `--help` for detailed information.
```
ADS-B Demodulator and Server

Usage: dump1090_rs [OPTIONS]

Options:
      --host <HOST>                    ip address to bind with for client connections [default: 127.0.0.1]
      --port <PORT>                    port to bind with for client connections [default: 30002]
      --driver <DRIVER>                soapysdr driver name (sdr device) from default `config.toml` or `--custom-config` [default: rtlsdr]
      --driver-extra <DRIVER_EXTRA>    specify extra values for soapysdr driver specification
      --custom-config <CUSTOM_CONFIG>  Filepath for config.toml file overriding or adding sdr config values for soapysdr
  -h, --help                           Print help (see more with '--help')
  -V, --version                        Print version
```

## Performance tricks

To enable maximum performance, instruct rustc to use features specific to your cpu.
```
> RUSTFLAGS="-C target-cpu=native" cargo r --release
```

Always use the latest rust releases including nightly, currently this gives around a 5-10% performance
boost.

## Testing
```
> cargo t --workspace --release
```

## Benchmark

Reading from a 512KB iq sample to ADS-B bytes takes ~3.0 ms, but feel free to run benchmarks on your own computer.
```
> RUSTFLAGS="-C target-cpu=native" cargo bench --workspace
```

### Intel i7-7700K CPU @ 4.20GHz
```
01                      time:   [2.9850 ms 2.9859 ms 2.9868 ms]
02                      time:   [2.9260 ms 2.9279 ms 2.9308 ms]
03                      time:   [2.8449 ms 2.8525 ms 2.8653 ms]
```

# Changes
See [CHANGELOG.md](https://github.com/rsadsb/dump1090_rs/blob/master/CHANGELOG.md)
