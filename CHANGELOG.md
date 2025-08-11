# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Cross-domain workflow orchestration with event subscriptions
- NATS event publishing with correlation/causation tracking
- Distributed transaction coordination across domains
- Cross-domain event subscription and filtering
- Comprehensive .claude directory for AI assistance
- Examples for NATS and cross-domain workflows

### Changed
- Updated module documentation structure
- Enhanced test coverage to 93%
- Improved error handling in handlers

## [0.3.0] - 2025-08-02

### Added
- Cross-domain workflow orchestration capabilities
- 10 new cross-domain event types
- CrossDomainHandler for multi-domain coordination
- Event subscription from external domains
- Distributed transaction support
- Filter-based event matching
- NATS integration with full event streaming
- Correlation and causation ID tracking
- NatsEventPublisher for distributed events
- NatsWorkflowCommandHandler for NATS-enabled commands
- Integration tests for cross-domain scenarios

### Changed
- Enhanced workflow architecture for distributed operations
- Updated dependencies to include async-nats and futures
- Improved test structure with 85 total tests

## [0.2.0] - 2025-01-30

### Added
- Complete event sourcing implementation
- CQRS command and query handlers
- Workflow and step state machines
- ContextGraph integration for visualization
- Comprehensive example scenarios
- User story tests (25 scenarios)

### Changed
- Refactored to use cim-domain from GitHub
- Improved state transition validation
- Enhanced error handling

### Fixed
- State machine transition guards
- Circular dependency detection

## [0.1.0] - 2025-01-15

### Added
- Initial workflow domain implementation
- Basic workflow aggregate
- Step management functionality
- Value objects for workflow concepts
- Command and event definitions
- State machine foundations

### Security
- No security vulnerabilities reported

---

## Version History Summary

- **0.3.0** - Cross-domain orchestration and NATS integration
- **0.2.0** - Event sourcing and CQRS implementation
- **0.1.0** - Initial release with basic workflow functionality

[Unreleased]: https://github.com/thecowboyai/cim-domain-workflow/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/thecowboyai/cim-domain-workflow/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/thecowboyai/cim-domain-workflow/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/thecowboyai/cim-domain-workflow/releases/tag/v0.1.0