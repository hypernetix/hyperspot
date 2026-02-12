# Changelog

All notable changes to this repository are documented in this file.

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

release-plz updates this file in the Release PR.

## [Unreleased]

## [0.2.8](https://github.com/cyberfabric/cyberfabric-core/compare/types-sdk-v0.2.7...types-sdk-v0.2.8) - 2026-02-12

### Other

- update Cargo.toml dependencies

## [0.2.7](https://github.com/cyberfabric/cyberfabric-core/compare/cf-modkit-v0.2.1...cf-modkit-v0.2.7) - 2026-02-12

### Added

- *(modkit-http)* introduce HttpClient (hyper+tower), replace reqwest and harden JWKS handling (by @MikeFalcon77)

### Fixed

- update module names to use kebab-case convention (by @yoskini) - #459
- *(dylint/de1301)* allow proc-macro impl methods via visit_assoc_item (by @Artifizer)

### Other

- Merge branch 'main' into shutdown-nl (by @Artifizer) - #475
- Merge branch 'main' into rescue-docs-standards (by @Artifizer) - #478

### Contributors

* @yoskini
* @MikeFalcon77
* @Artifizer

## [0.1.0] - 2026-01-23

### Added

- Initial release.
