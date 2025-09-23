# Tasks: Health Check Endpoint

**Input**: Design documents from `/home/jwilger/projects/caxton/specs/001-health-check-endpoint/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

## Execution Flow (main)

```
1. Load plan.md from feature directory
   ✓ Extracted: Rust 2024, Axum framework, single project structure
2. Load optional design documents:
   ✓ data-model.md: HealthResponse, HealthStatus, ServerConfig entities
   ✓ contracts/health-api.yaml: /health endpoint contract
   ✓ research.md: Axum framework selection, constitutional testing
3. Generate tasks by category:
   ✓ Setup: project init, dependencies, linting
   ✓ Tests: outside-in black-box integration tests (Constitutional Principle XII)
   ✓ Core: domain types, HTTP server, health endpoint
   ✓ Integration: configuration, observability, error handling
   ✓ Polish: performance tests, documentation updates
4. Apply task rules:
   ✓ Different files = marked [P] for parallel
   ✓ Same file = sequential (no [P])
   ✓ Outside-in tests before implementation (Constitutional mandate)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency graph
7. Create parallel execution examples
8. Validate task completeness
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths assume single project structure per plan.md

## Phase 3.1: Setup

- [x] T001 Add Axum HTTP framework dependencies to Cargo.toml (axum, tokio, serde, serde_json, nutype)
- [x] T002 [P] Configure cargo fmt and clippy settings for constitutional compliance
- [x] T003 [P] Set up basic project structure (src/lib.rs exports for testing)

## Phase 3.2: Outside-In Black-Box Integration Tests ⚠️ MUST COMPLETE BEFORE 3.3

**CONSTITUTIONAL PRINCIPLE XII: Outside-In Black-Box Testing Methodology**

**CRITICAL: These tests MUST follow the 11-Step Process and MUST FAIL before ANY implementation**

- [x] T004 Write black-box integration test for GET /health endpoint in tests/health_endpoint_test.rs
- [x] T005 Write black-box integration test for HEAD /health endpoint in tests/health_endpoint_test.rs
- [x] T006 Write black-box integration test for unsupported HTTP methods (POST/PUT/DELETE) in tests/health_endpoint_test.rs
- [x] T007 [P] Write black-box performance test for <100ms response requirement in tests/health_performance_test.rs

**Required for each test**: Run immediately after writing with `cargo test`, fix compilation/linting with smallest changes, maintain single failing test rule

## Phase 3.3: Core Implementation (ONLY after tests are failing)

- [ ] T008 [P] Create HealthResponse and HealthStatus types in src/health/types.rs
- [ ] T009 [P] Create ServerConfig with HostAddress, ServerPort, EndpointPath newtypes in src/config.rs
- [ ] T010 Create health endpoint handler function in src/health/handler.rs
- [ ] T011 Create Axum router with /health route registration in src/server.rs
- [ ] T012 Implement HTTP server initialization and startup in src/main.rs
- [ ] T013 Export public types through src/lib.rs for testing
- [ ] T014 Add JSON response serialization with proper Content-Type headers

## Phase 3.4: Integration

- [ ] T015 Add OpenTelemetry tracing middleware for observability (Constitutional Principle IV)
- [ ] T016 Add structured logging for HTTP requests and responses
- [ ] T017 Add graceful shutdown handling for HTTP server
- [ ] T018 Add configuration loading with environment variable support
- [ ] T019 Add error handling for server startup failures

## Phase 3.5: Polish

- [ ] T020 [P] Add property-based tests for domain types validation in tests/property_tests.rs
- [ ] T021 [P] Add benchmarks for response time verification in benches/health_benchmark.rs
- [ ] T022 [P] Update API documentation in docs/api.md
- [ ] T023 [P] Update website API reference per Constitutional Principle VIII
- [ ] T024 Create ADR for HTTP framework selection per Constitutional Principle VI
- [ ] T025 Run complete integration test suite and validate all requirements

## Dependencies

- Setup (T001-T003) before everything else
- Outside-in integration tests (T004-T007) before implementation (T008-T019)
- T008, T009 (types) before T010-T014 (handlers and server)
- T010 (handler) before T011 (router)
- T011 (router) before T012 (server startup)
- T013 (exports) needed for T004-T007 (integration tests)
- Integration (T015-T019) after core implementation
- Polish (T020-T025) after everything else

## Parallel Example

```
# Phase 3.1 Setup (can run T002-T003 in parallel after T001)
Task: "Configure cargo fmt and clippy settings for constitutional compliance"
Task: "Set up basic project structure (src/lib.rs exports for testing)"

# Phase 3.2 Integration Tests (T007 can run parallel to T004-T006)
Task: "Write black-box performance test for <100ms response requirement in tests/health_performance_test.rs"

# Phase 3.3 Core Types (T008-T009 can run in parallel)
Task: "Create HealthResponse and HealthStatus types in src/health/types.rs"
Task: "Create ServerConfig with HostAddress, ServerPort, EndpointPath newtypes in src/config.rs"

# Phase 3.5 Polish (T020-T024 can run in parallel)
Task: "Add property-based tests for domain types validation in tests/property_tests.rs"
Task: "Add benchmarks for response time verification in benches/health_benchmark.rs"
Task: "Update API documentation in docs/api.md"
Task: "Update website API reference per Constitutional Principle VIII"
Task: "Create ADR for HTTP framework selection per Constitutional Principle VI"
```

## Notes

- [P] tasks = different files, no dependencies
- MUST verify tests fail before implementing (Constitutional Principle XII)
- Follow 11-step TDD process for each failing test
- Commit after each task completion
- Use `cargo test`, `cargo fmt`, `cargo clippy` after each change

## Task Generation Rules

_Applied during main() execution_

**MANDATORY AGENT DELEGATION**: Test strategy planning delegated to test-designer agent per Constitutional Principle XI.

**CONSTITUTIONAL PRINCIPLE XII**: All implementation follows Outside-In Black-Box Testing Methodology with 11-Step Process.

After reading agent outputs:

1. **From Contracts**:
   - health-api.yaml → integration test for GET /health (T004)
   - health-api.yaml → integration test for HEAD /health (T005)
   - health-api.yaml → error handling tests for unsupported methods (T006)

2. **From Data Model**:
   - HealthResponse, HealthStatus → type creation task [P] (T008)
   - ServerConfig, newtypes → configuration type task [P] (T009)

3. **From User Stories (quickstart.md)**:
   - GET /health usage example → integration test (T004)
   - HEAD /health usage example → integration test (T005)
   - Performance requirement → performance test [P] (T007)

4. **Ordering**:
   - Setup → Outside-In Black-Box Integration Tests → Types → Handlers → Server → Integration → Polish
   - Integration tests MUST be written BEFORE any implementation code
   - Dependencies block parallel execution
   - Follow 11-Step Process for each test

## Validation Checklist

_GATE: Checked by main() before returning_

- [x] All contracts have corresponding black-box integration tests
- [x] All entities have type creation tasks
- [x] All integration tests come before implementation
- [x] Tests follow Constitutional Principle XII (Outside-In Black-Box Testing)
- [x] Parallel tasks truly independent
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task

## Constitutional Compliance Notes

- **Outside-In Black-Box Testing**: T004-T007 test only HTTP interface behavior
- **11-Step TDD Process**: Each integration test must fail, then implement minimal changes
- **Single Failing Test Rule**: Only one test failing at any time
- **Agent Delegation**: Test strategy developed by test-designer agent
- **Type-Driven Safety**: All domain types use nutype validation (T008-T009)
- **Observability First**: OpenTelemetry tracing added (T015-T016)
