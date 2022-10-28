# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.2.1

### Updated

- Updated `crossbeam-channel` to v0.5.6
- Updated `once_cell` to v0.13.1
- Updated `serde` to v1.0.444

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
