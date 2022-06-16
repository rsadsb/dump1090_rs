# dump1090_rs
[![Actions Status](https://github.com/rsadsb/dump1090_rs/workflows/CI/badge.svg)](https://github.com/rsadsb/dump1090_rs/actions)
[![dependency status](https://deps.rs/repo/github/rsadsb/dump1090_rs/status.svg)](https://deps.rs/repo/github/rsadsb/dump1090_rs)

Demodulate a ADS-B signal from a software defined radio device tuned at 1090mhz and
forward the bytes to applications such as [adsb_deku/radar](https://github.com/rsadsb/adsb_deku).

See [quickstart-guide](https://rsadsb.github.io/quickstart.html) for a quick installation guide.

See [rsadsb-v0.5.0](https://rsadsb.github.io/v0.5.0.html) for latest major release details.

## Tested Support

Through the use of the [rust-soapysdr](https://github.com/kevinmehall/rust-soapysdr) project,
we support [many different](https://github.com/pothosware/SoapySDR/wiki) software defined radio devices.
If you have tested this project on devices not listed below, let me know!
(you will need to add gain settings to [config.toml](dump1090_rs/config.toml) or use `--custom-config`)

| Device    | Supported/Tested | Recommend | argument          |
| --------- | :--------------: | :-------: | ----------------- |
| rtlsdr    |        x         |     x     | `--driver rtlsdr` |
| HackRF    |        x         |           | `--driver hackrf` |
| uhd(USRP) |        x         |           | `--driver uhd` |


## Usage
**Minimum Supported Rust Version**: 1.59.0

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
These images already have `soapysdr` installed.
```
> cargo install cross
> cross build --workspace --target x86_64-unknown-linux-gnu --relese
> cross build --workspace --target armv7-unknown-linux-gnueabihf --release
```

### Release Builds
Check the [latest release](https://github.com/rsadsb/dump1090_rs/releases) for binaries built from the CI.

## Run
Run the software using the default rtlsdr.
```
> cargo r --release
```

### help

See `--help` for detailed information.
```
dump1090_rs 0.5.1
wcampbell0x2a
ADS-B Demodulator and Server

USAGE:
    dump1090_rs [OPTIONS]

OPTIONS:
        --custom-config <CUSTOM_CONFIG>    Filepath for config.toml file overriding or adding sdr config values for soapysdr
        --driver <DRIVER>                  Soapysdr driver name (sdr device) from default `config.toml` or `--custom-config` [default: rtlsdr]
    -h, --help                             Print help information
        --host <HOST>                      Ip Address to bind with for client connections [default: 127.0.0.1]
        --port <PORT>                      Port to bind with for client connections [default: 30002]
    -V, --version                          Print version information
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

Reading from a 512KB iq sample to ADS-B bytes takes ~3.3 ms, but feel free to run benchmarks on your computer.
```
> RUSTFLAGS="-C target-cpu=native" cargo bench --workspace
```

### Faster hardware: Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz
```
01                      time:   [3.4230 ms 3.4274 ms 3.4322 ms]
02                      time:   [3.3413 ms 3.3452 ms 3.3492 ms]
03                      time:   [3.2562 ms 3.2597 ms 3.2635 ms]
```


### Slower hardware: Intel(R) Core(TM) i5-6300U CPU @ 2.40GHz
```
01                      time:   [5.7163 ms 5.7744 ms 5.8373 ms]
02                      time:   [5.5845 ms 5.6405 ms 5.7018 ms]
03                      time:   [5.4486 ms 5.5052 ms 5.5655 ms]
```

# Changes
See [CHANGELOG.md](https://github.com/rsadsb/dump1090_rs/blob/master/CHANGELOG.md)
