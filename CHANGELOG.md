# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/jwilger/caxton/compare/v0.1.3...v0.1.4) - 2025-08-11

### Bug Fixes

- remove claude-flow metrics from git and add to .gitignore
- update Jekyll workflow to include all documentation
- remove distracting animations from hero section
- make debugging tools tags consistent with other feature cards
- increase spacing between section titles and subtitles
- improve Getting Started section layout for better readability
- make Getting Started and Example sections visually consistent
- improve ADR and documentation pages with proper sidebars
- move release YAML generation to separate script
- simplify release YAML generation in workflow
- use line-by-line approach for YAML generation
- improve release notes handling and update license info
- remove theme toggle button and fix UI issues
- unify step number styling across all examples
- show foundation phase description instead of release notes
- make Multi-Agent Example section identical to Getting Started
- enhance documentation page styling and visual appeal
- load documentation CSS for documentation pages
- remove sidebar scrolling and apply documentation layout to all docs
- make ADR sidebars flexible to fit content
- remove sticky positioning from all sidebars
- constrain sidebar widths to reasonable limits
- align ADR card header elements horizontally
- properly layout ADR card header with all elements inline
- align ADR header elements vertically
- fine-tune ADR header element alignment
- align ADR header elements using baseline alignment
- correct ADR dates from January to July/August
- apply comprehensive styling to documentation pages
- use documentation layout for all docs pages in Jekyll workflow
- resolve code block formatting and add comprehensive UI enhancements
- prevent duplicate Table of Contents on documentation pages
- eliminate all dead links and add comprehensive documentation
- correct ADR-0012 links to GitHub repository
- correct Jekyll workflow to properly publish ADRs
- add Jekyll front matter to ADRs for proper site publication
- rename _adr to _adrs to match Jekyll collection config
- add Jekyll front matter to SOURCE ADRs in docs/adr/
- use consistent ADR title format in front matter
- add logo to documentation pages and fix README logo path
- replace README logo path with Jekyll relative URL during build
- resolve Security Monitoring workflow failures
- add security-events write permission for CodeQL
- simplify CodeQL configuration for Rust
- add missing CSS styling for distributed feature card tags
- improve code block rendering in ADRs
- resolve clippy warnings and establish code quality enforcement

### Documentation

- properly structure dual licensing files
- reorganize documentation structure for clarity and cohesion
- clarify capability registration pattern - single source of truth
- simplify agent manifest to essential deployment config only
- replace negative documentation with positive JSON Schema
- move architectural rationale to ADR, focus docs on how-to
- add ADR-0012 explaining pragmatic FIPA subset
- make documentation pragmatic and accessible
- remove work-in-progress disclaimers from README
- address architecture review recommendations
- implement coordination-first distributed architecture
- complete architecture documentation with ADRs 16-18 and comprehensive guides
- clean up ADRs and move procedures to operational runbook
- add comprehensive type system analysis and performance metrics
- add CI/CD pipeline and security stories to planning backlog
- update formatting via pre-commit hooks

### Features

- complete website redesign with enhanced UX and accessibility
- dynamically fetch and display GitHub release information
- add comprehensive development planning with 50 user stories
- implement Story 001 - WebAssembly runtime with sandboxing ([#001](https://github.com/jwilger/caxton/pull/001))
- implement Story 002 - Core Message Router with 236K msgs/sec performance
- implement Story 003 - Agent Lifecycle Management with comprehensive WASM validation
- implement comprehensive SPARC workflow with GitHub PR integration
- add hook to prevent git commit --no-verify
- implement comprehensive code quality enforcement ([#8](https://github.com/jwilger/caxton/pull/8))

### Miscellaneous Tasks

- remove ADR-0017 performance requirements
- release v0.1.4 ([#9](https://github.com/jwilger/caxton/pull/9))

### Refactoring

- eliminate primitive obsession using nutype domain types
- leverage Rust type system to eliminate test failures
- eliminate remaining primitive obsession with comprehensive domain types
- eliminate remaining primitive obsession with comprehensive domain types
- remove GitHub MCP dependencies and enhance gh CLI documentation
- fix SPARC orchestration architecture and model specifications
- restructure SPARC commands to use subdirectories

### Devex

- Remove current claude-flow and claude files

## [0.1.3](https://github.com/jwilger/caxton/compare/v0.1.2...v0.1.3) - 2025-08-06

### Miscellaneous Tasks

- remove Homebrew formula update from release workflow

## [0.1.2](https://github.com/jwilger/caxton/compare/v0.1.1...v0.1.2) - 2025-08-06

### Bug Fixes

- update build-artifacts workflow to attach to existing release

## [0.1.1] - 2025-08-06

### Changed
- Complete project restart with simplified foundation
- Removed all previous implementation code
- Established clean base for new development approach

### Added
- Basic Rust project structure with Cargo.toml
- Minimal main.rs entry point
- Development environment configuration (flake.nix)
- Claude Code and Claude Flow tools integration for AI-assisted development

### Fixed
- Corrected rust toolchain target parameter in build workflow

## [0.1.0] - 2025-08-06

### Added
- Initial release with comprehensive build and release automation
- GitHub Actions workflows for CI/CD
- Release automation with release-plz
- Multi-platform build artifacts (Linux, macOS, Windows)
- Automated Homebrew formula updates
- Security scanning and dependency management
- Comprehensive project documentation structure

### Documentation
- README with project vision
- SECURITY.md with vulnerability reporting process
- Basic ARCHITECTURE.md placeholder
- CONTRIBUTING.md guidelines

[Unreleased]: https://github.com/jwilger/caxton/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/jwilger/caxton/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jwilger/caxton/releases/tag/v0.1.0
