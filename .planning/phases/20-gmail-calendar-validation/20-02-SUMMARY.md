---
phase: 20-gmail-calendar-validation
plan: 02
subsystem: cli
tags: [rust, plugins, cli, nested-detection, multi-plugin, npm]

# Dependency graph
requires:
  - phase: 15-plugin-cli-install
    provides: "V2 plugin install from npm/github/local with plugin.json manifest"
provides:
  - "Nested plugin detection in install_from_npm, install_from_github, install_from_local"
  - "Vec<(String, String)> return type for multi-plugin packages"
  - "Transactional cleanup on failed multi-plugin registration"
  - "detect_nested_plugins() reusable helper function"
affects: [20-gmail-calendar-validation, plugin-ecosystem]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Nested plugin detection: scan immediate subdirs for plugin.json", "Root-first check before nested scan (backward compat)"]

key-files:
  modified:
    - "crates/amux-cli/src/plugins.rs"
    - "crates/amux-cli/src/main.rs"

key-decisions:
  - "Combined Tasks 1+2 into single commit: binary crate requires both plugins.rs and main.rs to compile together"
  - "Extracted detect_nested_plugins() as pure helper for testability without filesystem side effects on ~/.tamux/"
  - "Root plugin.json always checked before nested scan (Pitfall 5: backward compat preserved)"

patterns-established:
  - "Nested plugin detection: check root plugin.json first, only scan subdirs if absent"
  - "Transactional multi-plugin cleanup: on any registration failure, remove ALL files and deregister already-registered plugins"

requirements-completed: [GMAI-07]

# Metrics
duration: 6min
completed: 2026-03-25
---

# Phase 20 Plan 02: CLI Nested Plugin Detection Summary

**Nested plugin subdirectory detection in CLI install functions with Vec return types and transactional multi-plugin registration cleanup**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-25T08:20:39Z
- **Completed:** 2026-03-25T08:27:08Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- install_from_npm, install_from_github, install_from_local all detect nested plugin subdirectories when no root plugin.json exists
- Return types changed from `(String, String)` to `Vec<(String, String)>` across all install functions
- PluginAction::Add handler iterates over multiple plugins, registers each with daemon, performs transactional cleanup on failure
- 4 new unit tests for nested detection: multi-plugin, root compat, non-plugin dirs ignored, name from manifest

## Task Commits

Each task was committed atomically:

1. **Task 1: Add nested plugin detection to install functions and update return types** - `b5776e4` (test: RED), `7b81e1c` (feat: GREEN + Task 2 combined)

_Note: Tasks 1 and 2 were combined in a single commit because the binary crate requires both plugins.rs and main.rs changes to compile._

**Plan metadata:** (pending)

## Files Created/Modified
- `crates/amux-cli/src/plugins.rs` - Added detect_nested_plugins() helper, updated all install function return types to Vec, nested scan logic in install_from_npm/github/local/tarball, 4 new tests
- `crates/amux-cli/src/main.rs` - Updated PluginAction::Add to iterate Vec results, transactional cleanup on registration failure with deregister rollback

## Decisions Made
- Combined Tasks 1+2 into single commit: binary crate plugins.rs + main.rs must compile together; cannot have intermediate state
- Extracted detect_nested_plugins() as a pure helper function that only scans immediate subdirs (no recursion into nested dirs, no side effects on plugins_root)
- Root plugin.json always checked BEFORE nested scan in all four install paths (Pitfall 5: backward compat)
- Transactional cleanup removes ALL installed files and deregisters already-registered plugins on any registration failure (Pitfall 7)
- Daemon-unreachable case: files stay installed with warning (daemon can load on startup), not treated as failure

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Combined Tasks 1+2 due to binary crate compilation requirement**
- **Found during:** Task 1 (TDD GREEN phase)
- **Issue:** Binary crate requires main.rs to compile alongside plugins.rs; cannot run tests with partial changes
- **Fix:** Implemented Task 2 (main.rs update) as part of Task 1's GREEN phase
- **Files modified:** crates/amux-cli/src/main.rs
- **Verification:** cargo test + cargo check --workspace both pass
- **Committed in:** 7b81e1c

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Task merge necessary for compilation; no scope creep. All planned functionality implemented.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- CLI now supports multi-plugin npm packages (e.g., tamux-plugin-gmail-calendar with gmail/ and calendar/ subdirs)
- Ready for Phase 20 Plan 03 (actual gmail-calendar plugin manifests)
- All existing single-plugin installs work identically (backward compat verified)

---
## Self-Check: PASSED

- All created/modified files verified present on disk
- All commit hashes verified in git log
- Tests pass: 9/9 (5 existing + 4 new)
- Workspace compiles cleanly

---
*Phase: 20-gmail-calendar-validation*
*Completed: 2026-03-25*
