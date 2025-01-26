# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Semantic Versioning].

## [Unreleased]

## [0.6.1] - 2025-01-26

### Changed

- Git filters now should respect the `exclude` and `extend-exclude` options
- `--stdin-file-name` allows specifying the file name in `check` and `clean` if the content is piped into stdin
- `--respect-exclusions` an argument for `clean` that disables cleaning if the file is excluded

## [0.6.0] - 2025-01-14

Users should be aware that the behaviour of `--drop-id` has changed.
Use `--sequential-id` to retain the previous behaviour.

### Changed

- Terminal help output now has colors
- Rename `--drop-id` to `--sequential-id`: which replaces cell ids with sequential ids
- `--drop-id` now removes cell ids instead of replacing with sequential ids.
- Add `--id-action` command to cover `--drop-id`, `--sequential-id` and `--keep-id`

## [0.5.1] - 2024-12-04

### Security

- update dependencies for security

## [0.5.0] - 2024-10-17

### Added

- Add convenience option `--strip-kernel-info` to remove metadata related to python kernels
- Add `record` command to record kernel info to non-version-controlled local file
- Add hidden `smudge` command to be used as a git filter, that restores the stored kernel info to checked-out notebooks

## [0.4.0] - 2024-08-27

### Added

- exclude and extend-exclude for excluding files and directories
- add a subcommand hooks, to be used with e.g. pre-commit
- add a hook check-large-files to check for large files after cleaning notebooks
- add `--isolated` flag to ignore configuration files
- add lib.rs to the crate

## [0.3.7] - 2024-06-11

## Added

- Enabled configuration via `nbwipers.toml` and `.nbwipers.toml` files.

## [0.3.6] - 2024-05-22

### Security

- Fixed security advisory #13 gix package

### Fixed

- Fixed clippy lint #13

## [0.3.5] - 2024-03-17

### Changed

- Expand documentation in README
- improve coverage

### Fixed

- ignore whitespace lines in git attribute files

## [0.3.4] - 2024-03-05

### Security

- Updated `mio` version in lockfile per security alert

## [0.3.3] - 2024-02-05

### Added

- Added flag `--allow-no-notebooks` to suppress errors if no notebooks were found in the path.

## [0.3.2] - 2024-01-06

### Added

- Add `--exit-zero` flag and `NBWIPERS_CHECK_INSTALL_EXIT_ZERO` envvar to force `nbwipers check-install` to pass.

## [0.3.1] - 2023-12-17

### Fixed

- fix incorrect error messages

## [0.3.0] - 2023-12-07

### Added

- add subcommand to check install status
- add pre-commit hook to check install status

### Fixed

- create parent directories when creating config/attribute files
- skip parsing empty lines uninstalling attributes

## [0.2.0] - 2023-11-29

### Added

- Allow output in JSON format

### Changed

- Add uninstall command to reverse install command
- Allow specifying path to git config file in install and uninstall command

## [0.1.1] - 2023-11-25

### Fixed

- `install` now creates the attribute and config files if they do not exist, instead of erroring
- git filter commands registered by `install` are now correct

## [0.1.0] - 2023-11-25

- initial release

<!-- Links -->
[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html

<!-- Versions -->
[unreleased]: https://github.com/felixgwilliams/nbwipers/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/felixgwilliams/nbwipers/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/felixgwilliams/nbwipers/releases/tag/v0.1.0
