# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2022-12-31
### Added
- Add a progress bar for upload
  - Now you can track the upload status for big files!

![demo](https://user-images.githubusercontent.com/24392180/210139218-7c309398-1e4c-4323-ace7-ba30baf3c9d2.gif)

### Updated
- Upgrade dependencies

## [0.2.0] - 2022-10-04
### Added
- Add `--server-version` flag
  - With the last release of `rustypaste`, it is now possible to retrieve the server version via `/version` endpoint.
  - You can print the server version with using `-V`/`--server-version` flag with `rustypaste-cli`.

### Updated
- Upgrade dependencies
- Enable [GitHub Sponsors](https://github.com/sponsors/orhun) for funding
  - Consider supporting me for my open-source work 💖

## [0.1.8 ... 0.1.11] - 2022-06-18
### Updated
- Build/release for more platforms (MacOS & Windows)
  - (0.1.9) Upgrade transitive dependencies
  - (0.1.9) Fix deployment workflow (remove `x86_64-pc-windows-gnu` target)
  - (0.1.10) Fix deployment workflow (use compatible commands for MacOS & Windows)
  - (0.1.11) Fix deployment workflow (set the correct artifact name for Windows assets)

## [0.1.7] - 2022-05-29
### Updated
- Upgrade dependencies

## [0.1.6] - 2022-03-31
### Updated
- Fix typo in the manpage identifier
- Use `url::Url` for parsing URLs

## [0.1.5] - 2022-03-15
### Added
- Allow specifying `prettify` in config
- Add a manpage

### Changed
- Respect `XDG_CONFIG_HOME` as global config location
- Exit with a more informative message if no address is given

## [0.1.4] - 2022-03-13
### Added
- Add instructions for installing on Arch Linux

### Updated
- Update license copyright years
- Upgrade dependencies

### Fixed
- Apply clippy::map_flatten suggestion

## [0.1.3] - 2021-11-07
### Added
- Add argument for uploading files from remote URL

## [0.1.2] - 2021-09-19
### Fixed
- Read raw bytes from stdin.
  - Fixes "stream did not contain valid UTF-8" error

## [0.1.1] - 2021-09-19
Initial release.
