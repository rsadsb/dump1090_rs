# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
- Support Multi SDRs with the help of soapysdr: [!10](https://github.com/rsadsb/dump1090_rs/pull/10)
- Add CI builds for releases

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
