---
phase: 01-memory-foundation
plan: 01
subsystem: database
tags: [sqlite, fts5, episodic-memory, worm-ledger, pii-scrubbing, serde]

# Dependency graph
requires: []
provides:
  - "Episodic memory data types (Episode, EpisodeType, EpisodeOutcome, CausalStep, EpisodeLink, LinkType, NegativeConstraint, ConstraintType, CounterWhoState, TriedApproach, CorrectionPattern, EpisodicStore, EpisodicConfig)"
  - "SQLite schema for 5 episodic tables with FTS5 index and sync triggers"
  - "Episode CRUD operations (record_episode, get_episode, list_episodes_for_goal_run)"
  - "Episode link management (create_episode_link, get_episode_links, find_linked_episodes)"
  - "Privacy controls (PII scrubbing, TTL expiration, session suppression)"
  - "WORM episodic ledger append with SHA-256 hash chain"
  - "EpisodicStore field on AgentEngine, EpisodicConfig on AgentConfig"
affects: [01-02-PLAN, 01-03-PLAN, episodic-retrieval, counter-who, negative-knowledge]

# Tech tracking
tech-stack:
  added: []
  patterns: [episodic-episode-record, worm-hash-chain-append, pii-scrub-before-store]

key-files:
  created:
    - crates/amux-daemon/src/agent/episodic/mod.rs
    - crates/amux-daemon/src/agent/episodic/schema.rs
    - crates/amux-daemon/src/agent/episodic/store.rs
    - crates/amux-daemon/src/agent/episodic/privacy.rs
    - crates/amux-daemon/src/agent/episodic/links.rs
  modified:
    - crates/amux-daemon/src/agent/mod.rs
    - crates/amux-daemon/src/agent/engine.rs
    - crates/amux-daemon/src/agent/types.rs
    - crates/amux-daemon/src/agent/heartbeat_checks.rs
    - crates/amux-daemon/src/history.rs

key-decisions:
  - "Used direct execute_batch for schema instead of rusqlite_migration crate -- simpler and consistent with existing init_schema pattern"
  - "FTS5 created with .ok() to tolerate SQLite builds without FTS5 -- same pattern as context_archive_fts"
  - "WORM episodic ledger uses separate file (episodic-ledger.jsonl) matching existing ledger naming convention"
  - "Episode type/outcome stored as TEXT in SQLite with manual enum conversion -- consistent with existing goal_runs pattern"

patterns-established:
  - "Episodic module pattern: mod.rs (types) + schema.rs (DDL) + store.rs (CRUD impl AgentEngine) + privacy.rs (scrubbing) + links.rs (relationships)"
  - "WORM append pattern for episodic data: read last entry, compute SHA-256 chain, async file append"

requirements-completed: [EPIS-01, EPIS-07, EPIS-08, EPIS-09, EPIS-10, EPIS-11]

# Metrics
duration: 17min
completed: 2026-03-27
---

# Phase 01 Plan 01: Episodic Memory Foundation Summary

**SQLite-backed episodic memory module with 12+ data types, 5-table schema with FTS5, CRUD operations, WORM ledger, PII scrubbing, and episode link management wired into AgentEngine**

## Performance

- **Duration:** 17 min
- **Started:** 2026-03-27T00:48:46Z
- **Completed:** 2026-03-27T01:06:21Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Complete episodic memory data layer: 12+ serializable types covering episodes, links, negative knowledge, counter-who state, and configuration
- SQLite schema with 5 tables (episodes, episodes_fts, episode_links, negative_knowledge, counter_who_state), 12 indexes, and FTS5 sync triggers
- Episode CRUD with suppression check, PII scrubbing, TTL computation, event emission, and WORM ledger append
- Privacy module with 4 functions (scrub_episode, is_episode_suppressed, is_episode_expired, compute_expires_at)
- Episode link management supporting 5 link types with bidirectional querying
- 16 unit tests passing, 892 existing tests still passing (zero regressions)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create episodic module types, schema, and HistoryStore integration** - `ead7031` (feat)
2. **Task 2: Implement episode CRUD, WORM append, privacy scrubbing, and link management** - `de42036` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/episodic/mod.rs` - All episodic data types with serde derives, Default impls, helper methods, and 7 unit tests
- `crates/amux-daemon/src/agent/episodic/schema.rs` - SQLite DDL for 5 tables, FTS5 virtual table, sync triggers, and init function
- `crates/amux-daemon/src/agent/episodic/store.rs` - Episode CRUD (record_episode, get_episode, list_episodes_for_goal_run) with WORM append
- `crates/amux-daemon/src/agent/episodic/privacy.rs` - PII scrubbing, suppression, expiry, TTL computation with 9 unit tests
- `crates/amux-daemon/src/agent/episodic/links.rs` - Episode link CRUD (create, get, find by type) with bidirectional queries
- `crates/amux-daemon/src/agent/mod.rs` - Added `pub mod episodic;` declaration
- `crates/amux-daemon/src/agent/engine.rs` - Added `episodic_store: RwLock<EpisodicStore>` field and initialization
- `crates/amux-daemon/src/agent/types.rs` - Added `episodic: EpisodicConfig` to AgentConfig, `EpisodeRecorded` and `CounterWhoAlert` to AgentEvent
- `crates/amux-daemon/src/agent/heartbeat_checks.rs` - Added episodic_store to test AgentEngine constructor
- `crates/amux-daemon/src/history.rs` - Added episodic schema init call and "episodic" to WORM ledger kinds

## Decisions Made
- Skipped adding `rusqlite_migration` dependency as the plan specified -- schema initialization uses `execute_batch` directly, consistent with the existing `init_schema` pattern. Adding an unused dependency would violate clean dependency principles.
- FTS5 virtual table and triggers created with `.ok()` to gracefully handle SQLite builds without FTS5 support, matching the existing `context_archive_fts` pattern.
- Episode type and outcome enums stored as TEXT strings in SQLite with manual conversion functions, consistent with how goal_runs store status enums.
- WORM hash chain uses the same SHA-256 pattern as existing ledgers but writes to a separate `episodic-ledger.jsonl` file.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed tokio_rusqlite error conversion in init_schema**
- **Found during:** Task 1
- **Issue:** `init_episodic_schema` returns `anyhow::Error` but the closure inside `conn.call()` expects `tokio_rusqlite::Error`
- **Fix:** Added `.map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))` conversion
- **Files modified:** crates/amux-daemon/src/history.rs
- **Verification:** `cargo check -p tamux-daemon` passes
- **Committed in:** ead7031 (Task 1 commit)

**2. [Rule 3 - Blocking] Added episodic_store to heartbeat_checks test constructor**
- **Found during:** Task 1
- **Issue:** Test code in heartbeat_checks.rs constructs AgentEngine struct literal but was missing the new `episodic_store` field
- **Fix:** Added `episodic_store: RwLock::new(super::episodic::EpisodicStore::default())` to the test constructor
- **Files modified:** crates/amux-daemon/src/agent/heartbeat_checks.rs
- **Verification:** `cargo test -p tamux-daemon --bin tamux-daemon` compiles in test mode
- **Committed in:** ead7031 (Task 1 commit)

**3. [Rule 3 - Blocking] Added episodic field to AgentConfig Default impl**
- **Found during:** Task 1
- **Issue:** AgentConfig has a manual `impl Default` that was missing the new `episodic` field
- **Fix:** Added `episodic: super::episodic::EpisodicConfig::default()` to the Default impl
- **Files modified:** crates/amux-daemon/src/agent/types.rs
- **Verification:** `cargo check -p tamux-daemon` passes
- **Committed in:** ead7031 (Task 1 commit)

**4. [Rule 1 - Bug] Skipped unused rusqlite_migration dependency**
- **Found during:** Task 1
- **Issue:** Plan specifies adding `rusqlite_migration = "1.3"` but the implementation doesn't use it (schema uses execute_batch directly)
- **Fix:** Did not add the dependency to avoid unused dependency warnings
- **Files modified:** None (intentional omission)
- **Verification:** All schema operations work correctly with execute_batch

---

**Total deviations:** 4 auto-fixed (1 bug, 3 blocking)
**Impact on plan:** All auto-fixes necessary for compilation and test correctness. No scope creep.

## Issues Encountered
- Pre-existing test failures in `plugin::loader::tests` (2 tests fail on version string mismatch). Verified these failures exist on main branch before our changes. Not related to episodic memory work.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None - all data types are complete, all CRUD operations are functional, all privacy controls are implemented.

## Next Phase Readiness
- Episodic data layer is complete and ready for Plans 02 and 03
- Plan 02 can build retrieval logic on top of `get_episode`, `list_episodes_for_goal_run`, and the FTS5 index
- Plan 03 can build counter-who and negative knowledge logic on top of `CounterWhoState`, `NegativeConstraint`, and the `counter_who_state` table
- `record_episode` is ready to be called from goal runner and session end paths

## Self-Check: PASSED

All 5 created files verified present. Both task commits (ead7031, de42036) verified in git log. SUMMARY.md exists.

---
*Phase: 01-memory-foundation*
*Completed: 2026-03-27*
