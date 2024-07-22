# Changelog

All notable changes to `socksx` will be documented in this file.

## [2.0.0] - 2024-07-22
This project now uses [semantic versioning](https://semver.org). As such, **(BREAKING CHANGES)** will be indicated as such.

### Added
- Docker (Compose) files for containerization of example functions.
- Missing documentation, also in README.
- Unit tests.

### Changed
- Bumped dependencies to more recent versions.

### Fixed
- The crate not compiling on Windows.
- Broken badges in the README.


## [0.1.2] - 2021-12-14
### Added
- Automated coverage and release workflows.

### Fixed 
- Breaking changes from the `clap` library.

## [0.1.1] - 2021-07-29
### Added
- Experimental support for chaining (SOCKS6).
- Python package (`socksx-py`) with interface to `socksx`.

### Changed
- Use `tokio::io::copy_bidirectional` instead of local copy.

## [0.1.0] - 2021-03-16
### Added
- Initial implementation.