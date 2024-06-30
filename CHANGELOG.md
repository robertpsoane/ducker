# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/robertpsoane/ducker/compare/v0.0.2...v0.0.3) - 2024-06-30

### Added
- docker socket is configurable
- *(images)* toggle to show dangling images
- *(config)* introduce initial config system providing basic customisability
- *(cli)* add minimal clap commands

### Fixed
- *(history)* don't panic when attempting to view non-existent history in input field
- stop and delete containers in tokio task
- consistent help messaging for jump to top and bottom
- *(images)* g & G jump between top and bottom on image page

### Other
- improve README
- minimal user manual in README.md
- use mod.rs across the codebase
- *(issues)* update issues with feedback
- *(docs)* update issue templates
- *(logs)* simplify logs pane using list selection features introduced in ratatui 0.27.0
- add AUR instructions

## [0.0.2](https://github.com/robertpsoane/ducker/compare/v0.0.1...v0.0.2) - 2024-06-27

### Fixed
- *(ci)* fix ci formatting
- update readme with `--locked`
- add initial ci
- fix clippy

### Other
- fix formatting
- updates to installation instructions
