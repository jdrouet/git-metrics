# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
