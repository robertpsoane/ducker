# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/robertpsoane/ducker/compare/v0.1.0...v0.1.1) - 2024-08-14

### Other
- *(deps)* bump clap from 4.5.13 to 4.5.15 ([#69](https://github.com/robertpsoane/ducker/pull/69))
- *(deps)* bump serde from 1.0.204 to 1.0.207 ([#72](https://github.com/robertpsoane/ducker/pull/72))

## [0.1.0](https://github.com/robertpsoane/ducker/compare/v0.0.7...v0.1.0) - 2024-08-14

Over the past few weeks I've had little time to add to Ducker, however I've been trying to keep on top of dependencies.  I've decided to bump to v0.1.0 as Ducker's approaching a near-stable point, and to allow the semver to determine between feature update & changes and dependency patches.

### Other
- *(deps)* bump ratatui to 0.28.0 ([#70](https://github.com/robertpsoane/ducker/pull/70))
- *(deps)* bump crossterm from 0.27.0 to 0.28.1 ([#62](https://github.com/robertpsoane/ducker/pull/62))
- *(deps)* bump clap from 4.5.11 to 4.5.13 ([#63](https://github.com/robertpsoane/ducker/pull/63))
- *(deps)* bump tui-big-text from 0.5.2 to 0.5.3 ([#64](https://github.com/robertpsoane/ducker/pull/64))
- *(deps)* bump tui-big-text from 0.4.5 to 0.5.2 ([#60](https://github.com/robertpsoane/ducker/pull/60))
- *(deps)* bump tokio from 1.38.1 to 1.39.2 ([#58](https://github.com/robertpsoane/ducker/pull/58))
- *(deps)* bump clap from 4.5.9 to 4.5.11 ([#57](https://github.com/robertpsoane/ducker/pull/57))
- *(deps)* bump serde_yml from 0.0.10 to 0.0.11 ([#53](https://github.com/robertpsoane/ducker/pull/53))
- *(deps)* bump ratatui-macros from 0.4.2 to 0.4.3 ([#54](https://github.com/robertpsoane/ducker/pull/54))
- *(deps)* bump tokio from 1.38.0 to 1.38.1 ([#55](https://github.com/robertpsoane/ducker/pull/55))

## [0.0.7](https://github.com/robertpsoane/ducker/compare/v0.0.6...v0.0.7) - 2024-07-17

### Added
- *(tracing)* add initial tracing to ~/.local/share/ducker/ducker.log ([#49](https://github.com/robertpsoane/ducker/pull/49))
- initial network page ([#47](https://github.com/robertpsoane/ducker/pull/47))
- add support for listing volumes ([#44](https://github.com/robertpsoane/ducker/pull/44))

### Other
- *(config)* use arc instead of box for config ([#52](https://github.com/robertpsoane/ducker/pull/52))
- *(deps)* bump clap from 4.5.8 to 4.5.9 ([#51](https://github.com/robertpsoane/ducker/pull/51))
- *(versions)* use ureq instead of reqwest for version checks ([#48](https://github.com/robertpsoane/ducker/pull/48))
- symlink demo.gif to fix old cargo docs
- *(readme)* add a note about using `--locked` to cargo installs
- add an ai generated image of a duck riding a whale ([#45](https://github.com/robertpsoane/ducker/pull/45))

## [0.0.6](https://github.com/robertpsoane/ducker/compare/v0.0.5...v0.0.6) - 2024-07-11

Over the past few days I have added a few small features, as well as bumping some dependency versions where there have been known vulnerabilities.
One of the new features is a visual prompt that there is a new version.  This makes one network request at the beginning of each ducker session, but can be switched off in the config.  The bottom right of the screen shows the current version, and when there is a new version, it will show both.  I hope this is minimally intrusive, and am open to feedback.

The other main feature is the option to choose which command to exec into the shell as.  I'm not entirely happy with how it's presented to the user and once I've got modals sorted I will probably change this up a bit.  Similarly I'm wondering if its worth having two exec actions, one to use the default and one to offer the pop-up?

I have been giving versioning a bit of thought; when I started this I thought I'd start with v0.0.1 to signify the lack of completeness.  I intend to add initial support for volumes and networks, as well as get some unit test coverage (so I have some degree of trust in the stability of the system!) before bumping to v0.1.0.

### Added
- option to display all logs
- initial optional exec command
- add version info to bottom right of screen

### Other
- start using macros for layouts
- update to issues roadmap
- *(deps)* bump zerovec from 0.10.2 to 0.10.4
- *(deps)* bump serde from 1.0.203 to 1.0.204
- *(deps)* bump async-trait from 0.1.80 to 0.1.81

## [0.0.5](https://github.com/robertpsoane/ducker/compare/v0.0.4...v0.0.5) - 2024-07-07

When consolidating the command field to use a common text input widget as part of the plan to add more user options, I came across a regression in `exec`; in essence the exec action fails, which isn't ideal.

This was introduced as part of the changes to the transition payloads.  Unfortunately I'm not yet at a point where there is a test suite for the project (see the [pinned issue](https://github.com/robertpsoane/ducker/issues/2)  for some wider context as to how this project got to where it is) - I think this highlights that that needs prioritising.

### Fixed
- fix!(attach): exec regression fixed

### Other
- use new text input widget
- add brew commands ([#32](https://github.com/robertpsoane/ducker/pull/32))
- add CONTRIBUTING.md
- fix cargo.toml to include license from correct source

## [0.0.4](https://github.com/robertpsoane/ducker/compare/v0.0.3...v0.0.4) - 2024-07-06

### Added
- add ability for more detailed summary of images
- add ability for more detailed container description and stats ([#17](https://github.com/robertpsoane/ducker/pull/17))

### Other
- describe uses a trait object
- *(deps)* bump clap from 4.5.6 to 4.5.8
- *(pages)* simplify page lifecycle removing the redundant concept of visibility

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
