# dump1090_rs
[![Actions Status](https://github.com/rsadsb/dump1090_rs/workflows/CI/badge.svg)](https://github.com/rsadsb/dump1090_rs/actions)
[![dependency status](https://deps.rs/repo/github/rsadsb/dump1090_rs/status.svg)](https://deps.rs/repo/github/rsadsb/dump1090_rs)

Demodulate a ADS-B signal from a software defined radio device tuned at 1090mhz and
forward the bytes to applications such as [adsb_deku/radar](https://github.com/rsadsb/adsb_deku).

See [quistart-guide](https://rsadsb.github.io/quickstart.html) for a quick installation guide.

See [rsadsb-blog](https://rsadsb.github.io/v0.5.0.html) for latest release details.

## Tested Support

Through the use of the [rust-soapysdr](https://github.com/kevinmehall/rust-soapysdr) project,
we support [many different](https://github.com/pothosware/SoapySDR/wiki) software defined radio devices.
If you have tested this project on devices not listed below, let me know!
(you will need to add gain settings to `src/bin/dump1090.rs`)

| Device | Supported/Tested | Recommend | argument          |
| ------ | :--------------: | :-------: | ----------------- |
| rtlsdr |        x         |     x     | `--driver rtlsdr` |
| HackRF |        x         |           | `--driver hackrf` |


## Usage
**Minimum Supported Rust Version**: 1.56.1.

## Build

Install `soapysdr` drivers.

### Ubuntu
```
> apt install libsoapysdr-dev
```

### Cross Compile
Use [hub.docker.com/r/rsadsb](https://hub.docker.com/r/rsadsb/ci/tags) for cross compiling to the following archs.
These images already have `soapysdr` installed.
```
> cargo install cross
> cross build --workspace --target x86_64-unknown-linux-gnu
> cross build --workspace --target armv7-unknown-linux-gnueabihf
```

### Release Builds
Check the [latest release](https://github.com/rsadsb/dump1090_rs/releases) for binaries built from the CI.

## Run
Run the software using the default rtlsdr.
```
> cargo r --release
```

### help
```
dump1090_rs 0.5.0
wcampbell0x2a
ADS-B Demodulator and Server

USAGE:
    dump1090 [OPTIONS]

OPTIONS:
        --driver <DRIVER>    soapysdr driver (sdr device)
    -h, --help               Print help information
        --host <HOST>        ip address [default: 127.0.0.1]
        --port <PORT>        port [default: 30002]
    -V, --version            Print version information
```

## Performance tricks

To enable maximum performance, instruct rustc to use features specific to your cpu.
```
> RUSTFLAGS="-C target-cpu=native" cargo r --release
```

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
