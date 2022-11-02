# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.3.0

### Changed

- Dependency updates:
  - Move crossbeam-channel to the 0.5.X series
  - Move serde to any 1.0 version, for flexibility of dependent crates

## 1.2.0

### Added

- Added a `serde` cargo feature that, when enabled, adds `Serialize` and `Deserialize` implementations for `DeferDrop`.

## 1.1.0

### Changed

- `DeferDrop` no longer sends its value to the background thread if it's already in the background thread (that is, if you manage to send a `DeferDrop` value to the background thread, the inner value will be dropped eagerly, rather than being sent through the channel, since it's already in the background thread.)

## 1.0.1

### Changed

- Replaced our local OnceSlot with the once_cell crate
- Slimmed the crossbeam dependency to just crossbeam_channel
- Thank you to @cuviper for suggesting these changes!

## 1.0.0

### Added

- Initial release!
