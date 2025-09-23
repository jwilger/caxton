# Tasks: Health Check Endpoint

**Input**: Design documents from `/specs/001-health-check-endpoint/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/

## Execution Flow (main)

```
1. Load plan.md from feature directory
   → Found: axum HTTP framework, nutype domain types, TOML config
   → Extract: Rust 2024, axum/tokio/serde/nutype stack
2. Load optional design documents:
   → data-model.md: HealthResponse, HealthStatus, ServerConfig entities
   → contracts/: health-api.yaml → contract test task
   → research.md: axum selection, extensible HTTP foundation
3. Generate tasks by category:
   → Setup: dependencies, project structure, linting
   → Tests: contract tests, property tests, integration tests
   → Core: domain types, HTTP server, health endpoint
   → Integration: configuration, middleware, error handling
   → Polish: performance tests, documentation, benchmarks
4. Apply task rules:
   → Different files = mark [P] for parallel
   → Same file = sequential (no [P])
   → Tests before implementation (TDD)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency graph
7. Create parallel execution examples
8. Validate task completeness:
   → health-api.yaml has contract tests ✓
   → All entities have model implementations ✓
   → Health endpoint implemented ✓
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root (per plan.md)
- Paths shown below follow Rust single project structure

## Phase 3.1: Setup

- [ ] **T001** Add HTTP server dependencies to Cargo.toml (axum, tokio, serde, nutype, toml)
- [ ] **T002** Create project structure directories: `src/{types,http,api,config}`, `tests/{unit,integration,contract}`
- [ ] **T003** Configure development environment: update .gitignore for Rust, ensure cargo fmt/clippy

## Phase 3.2: Domain Types & Property Tests [P]

- [ ] **T004 [P]** Create health domain types in `src/types/health.rs` (HealthResponse, HealthStatus with serde)
- [ ] **T005 [P]** Create server config types in `src/types/config.rs` (ServerConfig with nutype: HostAddress, ServerPort, EndpointPath)
- [ ] **T006 [P]** Add property tests for nutype validation in `tests/unit/config_types_test.rs`
- [ ] **T007 [P]** Add unit tests for health response serialization in `tests/unit/health_types_test.rs`

## Phase 3.3: Contract Tests [P]

- [ ] **T008 [P]** Create health endpoint contract test in `tests/contract/health_endpoint_test.rs` (validate OpenAPI spec compliance)
- [ ] **T009 [P]** Create integration test scenarios in `tests/integration/health_integration_test.rs` (GET/HEAD methods, error cases)

## Phase 3.4: HTTP Foundation

- [ ] **T010** Create axum HTTP server setup in `src/http/server.rs` (tokio runtime, graceful shutdown)
- [ ] **T011** Create route registration system in `src/http/router.rs` (extensible for future endpoints)
- [ ] **T012** Create middleware pipeline in `src/http/middleware.rs` (observability, error handling)

## Phase 3.5: Configuration Management

- [ ] **T013** Create TOML configuration loading in `src/config/server.rs` (parse ServerConfig with validation)
- [ ] **T014** Create configuration defaults and environment overrides in `src/config/defaults.rs`
- [ ] **T015** Add configuration validation tests in `tests/unit/config_test.rs`

## Phase 3.6: Health Endpoint Implementation

- [ ] **T016** Create health endpoint handler in `src/api/health.rs` (GET/HEAD methods, JSON response)
- [ ] **T017** Integrate health endpoint with HTTP router in `src/http/router.rs`
- [ ] **T018** Add error handling for unsupported methods and malformed requests

## Phase 3.7: Main Application Integration

- [ ] **T019** Update main.rs to initialize HTTP server with configuration
- [ ] **T020** Add server lifecycle management (startup, shutdown, error handling)
- [ ] **T021** Create library module exports in `src/lib.rs`

## Phase 3.8: Performance & Polish [P]

- [ ] **T022 [P]** Add response time benchmarks in `benches/health_performance.rs` (validate sub-100ms requirement)
- [ ] **T023 [P]** Create load testing script per quickstart.md examples
- [ ] **T024 [P]** Add comprehensive integration tests from quickstart.md scenarios (curl examples, error cases)

## Dependency Graph

```
Setup Phase (T001-T003)
├── Types Phase [P] (T004-T007)
├── Contract Tests [P] (T008-T009)
└── Foundation Phase (T010-T012)
    ├── Config Phase (T013-T015)
    └── Implementation Phase (T016-T018)
        └── Integration Phase (T019-T021)
            └── Polish Phase [P] (T022-T024)
```

## Parallel Execution Examples

### Phase 3.2: Types Development (Parallel)

```bash
# All these can run in parallel (different files):
Task agent: T004 Create health domain types
Task agent: T005 Create server config types
Task agent: T006 Add property tests for config validation
Task agent: T007 Add unit tests for health serialization
```

### Phase 3.3: Contract Testing (Parallel)

```bash
# Contract tests can run in parallel (different test files):
Task agent: T008 Create health endpoint contract test
Task agent: T009 Create integration test scenarios
```

### Phase 3.8: Performance & Polish (Parallel)

```bash
# Final polish tasks can run in parallel:
Task agent: T022 Add response time benchmarks
Task agent: T023 Create load testing script
Task agent: T024 Add comprehensive integration tests
```

## Task Completion Criteria

### TDD Requirements

- All contract tests (T008-T009) must be written and **failing** before implementation
- Property tests (T006-T007) must validate nutype domain constraints
- Integration tests (T024) must cover quickstart.md scenarios

### Performance Requirements

- Response time benchmarks (T022) must validate sub-100ms requirement
- Load testing (T023) must demonstrate production readiness

### Constitutional Compliance

- Zero external service dependencies ✓ (only Rust crates)
- Type-driven safety ✓ (nutype domain types, axum type safety)
- Observability ✓ (middleware pipeline for tracing/logging)
- ADR documentation ✓ (001-http-framework-selection.md completed)

## Implementation Notes

### File Structure Created

```
src/
├── types/
│   ├── health.rs       # T004
│   └── config.rs       # T005
├── http/
│   ├── server.rs       # T010
│   ├── router.rs       # T011, T017
│   └── middleware.rs   # T012
├── api/
│   └── health.rs       # T016
├── config/
│   ├── server.rs       # T013
│   └── defaults.rs     # T014
├── main.rs             # T019, T020
└── lib.rs              # T021

tests/
├── unit/
│   ├── config_types_test.rs    # T006
│   ├── health_types_test.rs    # T007
│   └── config_test.rs          # T015
├── integration/
│   └── health_integration_test.rs  # T009, T024
└── contract/
    └── health_endpoint_test.rs     # T008

benches/
└── health_performance.rs        # T022
```

### Testing Strategy

- **Property Tests**: nutype validation rules (ports 1-65535, paths start with '/')
- **Contract Tests**: OpenAPI spec compliance (GET/HEAD methods, JSON schema)
- **Integration Tests**: End-to-end HTTP scenarios from quickstart.md
- **Performance Tests**: Sub-100ms response time validation

### Dependencies Added (T001)

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
nutype = { version = "0.4", features = ["serde"] }
toml = "0.8"

[dev-dependencies]
proptest = "1.0"
```

## Ready for Execution

All tasks are specific, ordered by dependencies, and marked for parallel execution where appropriate. Each task includes exact file paths and clear completion criteria. The implementation follows TDD principles with tests written before implementation code.
