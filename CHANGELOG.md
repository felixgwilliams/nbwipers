# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Semantic Versioning].

## [Unreleased]

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
