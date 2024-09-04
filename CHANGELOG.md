# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [v0.8.1] 2024-09-03
- Restore performance for recent rustc versions, force more functions to be inlined [!135](https://github.com/rsadsb/dump1090_rs/pull/135)

## [v0.8.0] 2024-09-02
- Update MSRV to `1.74` to [!130](https://github.com/rsadsb/dump1090_rs/pull/130)
- Properly decode short ADSB messages, thanks [@gariac](https://github.com/gariac) for finding. [!130](https://github.com/rsadsb/dump1090_rs/pull/130)
- Add `--quiet` to remove hex output of decoded message bytes [!130](https://github.com/rsadsb/dump1090_rs/pull/130)

## [v0.7.1] 2024-02-10
- Add `--driver-extra` to specify additional sopaysdr device options [!108](https://github.com/rsadsb/dump1090_rs/pull/108)

## [v0.7.0] 2023-11-22
- Inline `calculate_bit`, giving 5% performance boost
- Bump MSRV to 1.70, for new workspace packages, required libc version, and clap updates
- Update `--host` to support IPv6 [!67](https://github.com/rsadsb/dump1090_rs/pull/67)
  Thanks [@daviessm](https://github.com/daviessm)
- Add bladeRF 2.0 micro xA4 support [!21](https://github.com/rsadsb/dump1090_rs/pull/87)
  Thanks [@tjmullicani](https://github.com/tjmullicani)

## [v0.6.1]
- Update clap to v4
- Update MSRV to 1.64

## [v0.6.0] 2022-08-19
### Features
- Bump MSRV for using new const Mutex, removing `once_cell`.
- Fix error when running `cargo test` including the shared library `soapysdr`
- Bump `soapysdr-rs` to `v0.3.2`, enabling the use of read/write settings. For example, enabling bias-t on the rtlsdr is now allowed!
  [#16](https://github.com/rsadsb/dump1090_rs/issues/16) [!17](https://github.com/rsadsb/dump1090_rs/pull/17).
  Thanks [@Cherenkov11](https://github.com/Cherenkov11) for the feature suggestion
- Add `--custom-config` for providing custom configs for SDRs. See `--help` for examples
- Improve performance by 2% by using compiler aided `mul_add` [!21](https://github.com/rsadsb/dump1090_rs/pull/21/files)
- Improve performance by 38% by limiting slice size [!41](https://github.com/rsadsb/dump1090_rs/pull/41)
- Add support for `aarm64-unknown-linux-gnu`, for Raspberry Pi 64 bit
- Updated other docker images to `0.2.0`: [docker hub](https://hub.docker.com/repository/docker/rsadsb/ci/tags?page=1&ordering=last_updated&name=0.2.0)

### Breaking
- Stripped release binaries, requires bump of MSRV to `1.59`. This reduces the size of the generated binary from ~800KB to ~400KB.
- Added `overflow-checks`, tested without errors.

## [v0.5.1] 2022-02-13

## [v0.5.0] 2022-02-12
- Support Multi SDRs with the help of soapysdr: [!10](https://github.com/rsadsb/dump1090_rs/pull/10)
- Add CI builds for releases
- Docker images for cross compiling are available at [hub.docker](https://hub.docker.com/r/rsadsb/ci/tags).
- Binaries are available in Github Releases from the CI.

## [v0.4.0] 2021-12-08
- 9% speed increases in benchmarks.
- Add `--host` and `--port` for control of TCP server.

## [v0.2.0] 2021-10-31
- Add tests
- Add benchmarks

## [v0.1.1] 2021-10-12
- Add `Phase` for holding state matchine of current phase, as well as functionalize the huge match/if statement
  from the original dump1090_rs fork.
- Handle ConnectionReset from client applications

## [v0.1.0] 2021-09-21
- Initial Release
