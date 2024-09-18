# CHANGELOG

## [Unreleased]

### Added

- We are introducing the `env!()` directive to be able to call the shell from within our scripts. Inherently the language does not have a way to interact with the filesystem or network, so this is a way to get around that limitation.

### Changed

- The translation is lost for types like bool which when say using Swahili, they are parsed as `kweli` and `uongo` but evaluate to `true` and `false` respectively. This is a breaking change.
- The swahili translation changes from `bool` to `bulin`.