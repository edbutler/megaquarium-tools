# Check/Report Separation Design

## Summary

This design separates validation logic from output formatting in the `check` module by extracting all printing functions into a new `report` module. Currently, `check.rs` is marked as "Mixed" because it contains both pure validation functions (calculating minimum viable tanks, finding constraint violations) and I/O functions (printing results to stdout). This violates the project's established Functional Core, Imperative Shell (FCIS) pattern used throughout the codebase.

The refactoring introduces `AquariumCheckResult` as a new result type that wraps exhibit-level validation results, mirroring the existing `ExhibitCheckResult` (renamed from `CheckResult`). The mixed function `check_for_viable_aquarium` is split into a pure `validate_aquarium()` function that returns data and a `print_aquarium_result()` function in the new `report.rs` module that handles output. This separation makes validation logic testable without I/O dependencies while maintaining the existing `debug` flag pattern for output formatting control.

## Definition of Done
- **check.rs** contains only pure functions and the structs they return
- Rename `CheckResult` → `ExhibitCheckResult` for naming consistency with new `AquariumCheckResult`
- `AquariumCheckResult` wraps exhibit results with an `is_okay()` method (derived, not stored) for consistency with `ExhibitCheckResult`
- **report.rs** (new file) contains all printing/formatting functions that depend on check.rs types
- The mixed function `check_for_viable_aquarium` is split into pure `validate_aquarium()` → `AquariumCheckResult` and a print function in report.rs
- The existing `debug` flag pattern continues to control output formatting
- The pure validation logic is testable without I/O
- main.rs is updated to use the new module structure

## Glossary

- **FCIS (Functional Core, Imperative Shell)**: Architectural pattern separating pure logic (functional core) from side effects like I/O (imperative shell). The core contains testable functions that return data; the shell handles interactions with the outside world.
- **Exhibit**: A single aquarium tank configuration containing fish, fixtures, and environmental parameters. The game term for what users validate.
- **Aquarium**: A collection of exhibits. In save files, an aquarium can contain multiple tanks/exhibits.
- **Violations**: Constraint failures detected by the rules engine (e.g., temperature conflicts, predation risks, insufficient shoaling group size).
- **Minimum viable tank**: The smallest environment configuration (size, temperature range, fixtures) that satisfies all fish requirements in an exhibit.
- **S-expression**: Lisp-style serialization format used for aquarium I/O in this project.
- **Debug flag**: CLI option controlling output format — when enabled, shows Rust's debug representation; when disabled, uses s-expression format.

## Architecture

Separate pure validation logic from I/O (printing) following the project's established FCIS pattern.

**check.rs (Functional Core)** contains:
- `ExhibitCheckResult` — renamed from `CheckResult`, holds violations, food requirements, and minimum viable environment for a single exhibit
- `AquariumCheckResult` — new struct wrapping `Vec<(String, ExhibitCheckResult)>` with derived `is_okay()` method
- `validate_aquarium()` — new pure function replacing the mixed `check_for_viable_aquarium`
- All existing pure functions: `check_for_viable_tank`, `try_expand_tank`, `create_check_query`, `environment_for_exhibit`, and private helpers

**report.rs (Imperative Shell)** contains:
- `print_violations()` — moved from check.rs
- `print_exhibit_result()` — renamed from `print_check_result` to match struct name
- `print_environment_differences()` — moved from check.rs
- `print_aquarium_result()` — new function handling output for `AquariumCheckResult`

**Dependency flow:** `main.rs` → `report.rs` → `check.rs`

## Existing Patterns

Investigation confirmed the codebase uses explicit `// pattern:` comments at the top of each file:
- `// pattern: Functional Core` for pure modules (rules.rs, animal.rs, tank.rs, etc.)
- `// pattern: Imperative Shell` for I/O modules (main.rs, data.rs)
- `// pattern: Mixed` currently on check.rs (acknowledging the issue this design fixes)

This design follows the existing pattern:
- check.rs will be updated to `// pattern: Functional Core`
- report.rs will use `// pattern: Imperative Shell`

Result struct pattern from existing code:
- Functions return data structs (e.g., `CheckResult`, `Violation`)
- Separate print functions consume those structs
- `debug` flag controls output format (debug vs s-expression)

## Implementation Phases

<!-- START_PHASE_1 -->
### Phase 1: Create report.rs with moved functions

**Goal:** Create the new imperative shell module with printing functions moved from check.rs

**Components:**
- New `src/report.rs` with `// pattern: Imperative Shell` header
- Move `print_violations`, `print_check_result`, `print_environment_differences` from check.rs
- Add `mod report;` and `use report::*;` to main.rs
- Update check.rs header to `// pattern: Functional Core`

**Dependencies:** None (first phase)

**Done when:** Project builds, existing CLI commands work unchanged
<!-- END_PHASE_1 -->

<!-- START_PHASE_2 -->
### Phase 2: Rename CheckResult to ExhibitCheckResult

**Goal:** Rename the existing result struct for naming consistency

**Components:**
- Rename `CheckResult` → `ExhibitCheckResult` in check.rs
- Rename `print_check_result` → `print_exhibit_result` in report.rs
- Update all references in main.rs and tests

**Dependencies:** Phase 1 (report.rs exists)

**Done when:** Project builds, all tests pass, CLI commands work unchanged
<!-- END_PHASE_2 -->

<!-- START_PHASE_3 -->
### Phase 3: Add AquariumCheckResult and validate_aquarium

**Goal:** Create pure validation function for aquariums

**Components:**
- New `AquariumCheckResult` struct in check.rs with `is_okay()` method
- New `validate_aquarium()` pure function in check.rs
- Remove `check_for_viable_aquarium` from check.rs (mixed function)

**Dependencies:** Phase 2 (ExhibitCheckResult exists with correct name)

**Done when:** New structs and function compile, existing tests still pass
<!-- END_PHASE_3 -->

<!-- START_PHASE_4 -->
### Phase 4: Add print_aquarium_result and update main.rs

**Goal:** Complete the separation by adding the print function and wiring main.rs

**Components:**
- New `print_aquarium_result()` function in report.rs
- Update Validate command in main.rs to use `validate_aquarium()` + `print_aquarium_result()`
- Pass `debug` flag explicitly to print function

**Dependencies:** Phase 3 (validate_aquarium exists)

**Done when:** Validate command works correctly with new code path
<!-- END_PHASE_4 -->

<!-- START_PHASE_5 -->
### Phase 5: Update tests for pure validation

**Goal:** Update existing test and add new tests for pure validation

**Components:**
- Update `test_happy_path_single_tank_compatible_fish` to call `validate_aquarium()` and assert on `AquariumCheckResult`
- Add test for aquarium with violations (assert `is_okay() == false`)
- Add test verifying specific violations are captured

**Dependencies:** Phase 4 (full integration complete)

**Done when:** All tests pass, pure validation logic is covered
<!-- END_PHASE_5 -->

## Additional Considerations

**Backwards compatibility:** Not a concern per user confirmation — main.rs is the only consumer of check.rs functions.
