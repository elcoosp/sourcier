# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/elcoosp/sourcier/releases/tag/v0.1.0) - 2025-12-19

### Added

- add serde optional
- view poc
- poc RuntimeFeedback (with_feedback fn)
- *(core)* include content in SourceFilesMap create FileEntry underneath, sort using content length after path, preallocate & add `get_content`
- *(core)* add fid, sfm and sfp mods to manage source file position in a compact u64 repr, and mapping path to id in O(1), utils in sfp

### Fixed

- fix remaining warns
- test issues
- dep:memchr for view feat
- missing clo git track & mv SourceFilesMap defaults to self +
- *(core)* mv clo mod & use memchr

### Other

- use exhaustive_test_suite macro
- clippy
- more cases
- use yaml & redactions
- add exhaustive_test_suite macro to automate snapshots
- add test_suite macro with basic multiline view
- add test_utils
- clippy
- add example integration
