# dump1090_rs
[![Actions Status](https://github.com/wcampbell0x2a/dump1090_rs/workflows/CI/badge.svg)](https://github.com/wcampbell0x2a/dump1090_rs/actions)

Fork of https://github.com/johnwstanford/dump1090_rs, without parsing messages.
This project is meant to just forward bytes from the the demodulated iq stream from a rtlsdr to my own [adsb_deku](https://github.com/wcampbell0x2a/adsb_deku) library/apps.

## Usage

```
cargo r --release
```

## Testing
```
cargo t --release
```

## Benchmark

Reading from a 256KB iq sample to ADS-B bytes takes ~3.3 ms, but feel free to run benchmarks on your computer.
```
cargo bench
```

# Changes
See [CHANGELOG.md](https://github.com/wcampbell0x2a/dump1090_rs/blob/master/CHANGELOG.md)
