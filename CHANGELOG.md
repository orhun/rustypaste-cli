# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.2] - 2024-12-11

### Changed

- Bump dependencies

### Fixed

- Omit explicit lifetime by @tessus in [#156](https://github.com/orhun/rustypaste-cli/pull/156)

## [0.9.1] - 2024-08-01

### Added

- Show creation date in list output by @tessus in [#117](https://github.com/orhun/rustypaste-cli/pull/117)

See the latest rustypaste server release ([0.15.1](https://github.com/orhun/rustypaste/releases/tag/v0.15.1)) for more information.

- Add @tessus as a maintainer

### Changed

- Update license copyright years by @orhun
- Bump dependencies

## [0.9.0] - 2024-03-27

### Added

- Add argument to specify filename by @tessus in [#88](https://github.com/orhun/rustypaste-cli/pull/88)

`rustypaste` >=0.15.0 supports overriding the file name by using `filename` header. You can also use this functionality in `rpaste` as follows:

```sh
rpaste -n filename-on-server.txt awesome.txt
```

### Changed

- Simplify reading content from stdin by @tranzystorekk in [#96](https://github.com/orhun/rustypaste-cli/pull/96)
- Split workflow into stable and beta/nightly by @tessus in [#99](https://github.com/orhun/rustypaste-cli/pull/99)
- Get rid of the unmaintained actions by @tessus in [#102](https://github.com/orhun/rustypaste-cli/pull/102)
- Add Mergify config by @orhun
- Bump dependencies by @orhun

### Removed

- Remove deprecated actions by @tessus in [#101](https://github.com/orhun/rustypaste-cli/pull/101)

### New Contributors

- @tranzystorekk made their first contribution in [#96](https://github.com/orhun/rustypaste-cli/pull/96)

## [0.8.0] - 2023-09-05

### Added

- Add option to delete file(s) from server (#54)

`rustpaste` now supports deleting files starting from [`0.14.0`](https://github.com/orhun/rustypaste/releases/tag/v0.14.0) thus a new flag (`-d`) is added to `rpaste`.

```sh
rpaste -d file.txt
```

To use this feature, set tokens for both `rustypaste` and `rustypaste-cli` in the configuration file via `delete_tokens` / `delete_token` option.

### Changed

- Use IsTerminal from stdlib (#55)
- Disable Rust beta builds
- Upgrade dependencies

## [0.7.0] - 2023-08-12

### Added

- Added `-l` flag for retrieving file list from the server (#45)

For example:

```sh
rpaste -l  # JSON output
rpaste -lp # Table output (pretty)
```

`[server].expose_list` option should be set to `true` on `rustypaste` server for this flag to work.

### Removed

- Remove extra newline from version output (#36)

## [0.6.0] - 2023-07-08

### Changed

- Automatically detect if the data is piped (#28)

Now when data is piped into `rpaste`, there is no reason to add `-` as a file.

Before:

```
cat whatever.txt | rpaste -
```

After:

```
cat whatever.txt | rpaste
```

- Upgrade dependencies

## [0.5.0] - 2023-07-01

### Added

- Support using the OS TLS trust store (#18)
  - Added `use-native-certs` feature flag for enabling the default TLS implementation.

### Changed

- Mention the platform-specific configuration directory in the documentation (#10)
- Upgrade dependencies

### Fixed

- Fix the server version retrieval (#17)

## [0.4.0] - 2023-05-31

### Added

- Support uploading one shot URLs

`rustypaste` supports one shot URL uploads since [`0.10.0`](https://github.com/orhun/rustypaste/releases/tag/v0.10.0). To use this feature:

```sh
rpaste -ou https://example.com/some/long/url
```

- Add example for using the stdin
- Add installation instructions for Alpine Linux

### Changed

- Update funding options
  - [Buy me a coffee to support my open source endeavours!](https://www.buymeacoffee.com/orhun) ☕

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
