# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3](https://github.com/jdrouet/git-metrics/compare/v0.2.2...v0.2.3) - 2024-08-06

### Added
- create init command

## [0.2.2](https://github.com/jdrouet/git-metrics/compare/v0.2.1...v0.2.2) - 2024-08-03

### Added
- create import command ([#66](https://github.com/jdrouet/git-metrics/pull/66))

### Fixed
- add missing checkout for releasing binaries

### Other
- fix readme example
- update readme with current import command state
- execute git check even on push
- replace old actions locations to new ones
- use action for checking metrics ([#65](https://github.com/jdrouet/git-metrics/pull/65))

## [0.2.1](https://github.com/jdrouet/git-metrics/compare/v0.2.0...v0.2.1) - 2024-07-20

### Added
- introduce absolute changes ([#63](https://github.com/jdrouet/git-metrics/pull/63))
- format numbers with unit ([#60](https://github.com/jdrouet/git-metrics/pull/60))
- add colors to output ([#53](https://github.com/jdrouet/git-metrics/pull/53))
- create a check command ([#46](https://github.com/jdrouet/git-metrics/pull/46))

### Other
- add format option to check command
- cross build linux binaries ([#61](https://github.com/jdrouet/git-metrics/pull/61))
- update project goals
- update readme with check command
- move format option to shared module
- make linux-amd64 build run outside of docker ([#56](https://github.com/jdrouet/git-metrics/pull/56))
- update dependencies ([#57](https://github.com/jdrouet/git-metrics/pull/57))
- move writing code out of service ([#52](https://github.com/jdrouet/git-metrics/pull/52))
- move display logic in cmd ([#51](https://github.com/jdrouet/git-metrics/pull/51))
- build deb files for linux ([#50](https://github.com/jdrouet/git-metrics/pull/50))
- make sure the check command answers as expected ([#49](https://github.com/jdrouet/git-metrics/pull/49))
- send a notification after release
- increase coverage for add command
- cover init command
- add license section to readme

## [0.2.0](https://github.com/jdrouet/git-metrics/compare/v0.1.4...v0.2.0) - 2024-06-30

### Added
- [**breaking**] hide previous metrics by default and add option ([#43](https://github.com/jdrouet/git-metrics/pull/43))

### Other
- add image to readme
- update readme to display report
- extract build workflow ([#41](https://github.com/jdrouet/git-metrics/pull/41))
- fix commit lint rules
- Add the repository field to Cargo.toml ([#39](https://github.com/jdrouet/git-metrics/pull/39))
- update condition for triggering deployment

## [0.1.4](https://github.com/jdrouet/git-metrics/compare/v0.1.3...v0.1.4) - 2024-06-19

### Added
- improve error message ([#35](https://github.com/jdrouet/git-metrics/pull/35))

## [0.1.3](https://github.com/jdrouet/git-metrics/compare/v0.1.2...v0.1.3) - 2024-06-18

### Added
- allow to set verbosity with env variable

### Other
- configure to print the metrics diff ([#33](https://github.com/jdrouet/git-metrics/pull/33))
- auto release binaries
- update goals on readme
- update usage in readme

## [0.1.2](https://github.com/jdrouet/git-metrics/compare/v0.1.1...v0.1.2) - 2024-06-04

### Added
- create diff command ([#31](https://github.com/jdrouet/git-metrics/pull/31))

### Fixed
- update metric format to follow prometheus

### Other
- update local refs and service ([#30](https://github.com/jdrouet/git-metrics/pull/30))
- move cli logic to service
- rename Bakend proto
- remove stderr from executor
- rename repository to backend

## [0.1.1](https://github.com/jdrouet/git-metrics/compare/v0.1.0...v0.1.1) - 2024-05-27

### Added
- create log command ([#24](https://github.com/jdrouet/git-metrics/pull/24))
- handle note conflicts ([#23](https://github.com/jdrouet/git-metrics/pull/23))
- make git2 work on github ([#16](https://github.com/jdrouet/git-metrics/pull/16))
- allow to pick backend implementation ([#12](https://github.com/jdrouet/git-metrics/pull/12))

### Other
- limit release when code changes
- limit code testing when code changes
- build git-metrics for windows ([#27](https://github.com/jdrouet/git-metrics/pull/27))
- build binary for apple-darwin ([#26](https://github.com/jdrouet/git-metrics/pull/26))
- build binaries using docker ([#25](https://github.com/jdrouet/git-metrics/pull/25))
- update readme with instructions
- update readme.md
- apply clippy suggestion
- parse tag as a struct
- use macro to handle errors
- push using command backend
- only release if event is workflow dispatch
- add some integration tests ([#21](https://github.com/jdrouet/git-metrics/pull/21))
- rename protocol option to backend ([#15](https://github.com/jdrouet/git-metrics/pull/15))
- remove extra command
- create configuration to release binaries ([#13](https://github.com/jdrouet/git-metrics/pull/13))
- release ([#10](https://github.com/jdrouet/git-metrics/pull/10))

## [0.1.0](https://github.com/jdrouet/git-metrics/releases/tag/v0.1.0) - 2024-05-19

### Added
- create command protocol ([#9](https://github.com/jdrouet/git-metrics/pull/9))
- create pull command ([#7](https://github.com/jdrouet/git-metrics/pull/7))
- add tracing logs ([#8](https://github.com/jdrouet/git-metrics/pull/8))
- create push command ([#5](https://github.com/jdrouet/git-metrics/pull/5))
- add command to remove metrics
- create command to add metric

### Fixed
- apply clippy suggestions

### Other
- create release-plz config
- add some integration testing ([#6](https://github.com/jdrouet/git-metrics/pull/6))
- check pushed binary size
- add some future commands to readme
- create config to monitor binary size ([#2](https://github.com/jdrouet/git-metrics/pull/2))
- create configuration files
- create readme
- rename context to tag, add tests and license
- simply show current metrics
- first commit
