# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
### Features
  Fixes error when running `cargo test` including the shared library `soapysdr`
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
