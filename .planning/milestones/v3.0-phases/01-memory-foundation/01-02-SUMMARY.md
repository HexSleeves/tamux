---
phase: 01-memory-foundation
plan: 02
subsystem: agent-intelligence
tags: [fts5, bm25, episodic-retrieval, goal-planning, recency-weighting, system-prompt]

# Dependency graph
requires:
  - "01-01: Episodic memory data types, SQLite schema with FTS5 index, episode CRUD, WORM ledger"
provides:
  - "FTS5 retrieval engine with BM25 + recency re-ranking (retrieve_relevant_episodes)"
  - "Temporal and entity-scoped episode queries (search_episodes_temporal, search_episodes_by_entity)"
  - "Episodic context formatting with severity labels and token budget (format_episodic_context)"
  - "Goal boundary episode recording (record_goal_episode on complete/fail)"
  - "Session end episode recording (record_session_end_episode)"
  - "Episode surfacing in goal planning context (request_goal_plan injects past episodes)"
  - "System prompt episodic context injection parameter"
affects: [01-03-PLAN, counter-who, negative-knowledge, goal-planning-quality, system-prompt-context]

# Tech tracking
tech-stack:
  added: [regex]
  patterns: [fts5-bm25-retrieval, recency-exponential-decay, over-fetch-rerank, episodic-context-injection]

key-files:
  created:
    - crates/amux-daemon/src/agent/episodic/retrieval.rs
  modified:
    - crates/amux-daemon/src/agent/episodic/mod.rs
    - crates/amux-daemon/src/agent/episodic/store.rs
    - crates/amux-daemon/src/agent/goal_planner.rs
    - crates/amux-daemon/src/agent/goal_llm.rs
    - crates/amux-daemon/src/agent/system_prompt.rs
    - crates/amux-daemon/src/agent/agent_loop.rs

key-decisions:
  - "FTS5 over-fetch 3x then re-rank with recency weighting -- ensures BM25 relevance combined with temporal freshness"
  - "Exponential decay with 14-day half-life for recency weighting -- balances recent vs historically important episodes"
  - "Severity labels (WARNING/CAUTION/REFERENCE/NOTE) in episodic context -- failure episodes get prominent visual treatment"
  - "Episode surfacing injected into goal planning prompt rather than system prompt -- keeps system prompt clean, goal-specific context in goal path"

patterns-established:
  - "FTS5 retrieval pattern: over-fetch from SQLite FTS5, re-rank with domain signal, truncate to limit"
  - "Episodic context formatting pattern: header + labeled entries with token budget enforcement"
  - "Goal boundary recording pattern: extract entities/causal chains from GoalRun, delegate to record_episode"

requirements-completed: [EPIS-02, EPIS-03, EPIS-04, EPIS-05, EPIS-06]

# Metrics
duration: 7min
completed: 2026-03-27
---

# Phase 01 Plan 02: Episodic Retrieval and Goal Integration Summary

**FTS5 retrieval engine with BM25 + recency re-ranking, goal boundary episode recording, and episodic context injection into goal planning prompts**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-27T01:12:15Z
- **Completed:** 2026-03-27T01:19:44Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Complete FTS5 retrieval engine: format_fts5_query, retrieve_relevant_episodes with BM25 + exponential-decay recency re-ranking, temporal and entity-scoped query variants
- Goal boundary episode recording: record_goal_episode extracts entities (file paths, step titles) and causal chains from GoalRun, wired into both complete_goal_run and fail_goal_run
- Episode surfacing in goal planning: request_goal_plan retrieves top 5 relevant past episodes via FTS5 and injects formatted context with severity labels before the LLM call
- System prompt episodic context support: build_system_prompt accepts optional episodic_context parameter for injection alongside learned patterns
- 9 new unit tests covering query formatting, recency weighting, and context rendering (25 total episodic tests passing)

## Task Commits

Each task was committed atomically:

1. **Task 1: Build FTS5 retrieval engine with multi-signal ranking** - `3f23ca2` (feat)
2. **Task 2: Wire episode recording into goal boundaries and surfacing into planning** - `4a007e4` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/episodic/retrieval.rs` - FTS5 retrieval engine: query formatting, recency weighting, BM25 retrieval, temporal/entity queries, episodic context formatting with 9 tests
- `crates/amux-daemon/src/agent/episodic/mod.rs` - Added `pub mod retrieval;` declaration
- `crates/amux-daemon/src/agent/episodic/store.rs` - Added record_goal_episode and record_session_end_episode helper methods
- `crates/amux-daemon/src/agent/goal_planner.rs` - Wired episodic recording into complete_goal_run (Success) and fail_goal_run (Failure)
- `crates/amux-daemon/src/agent/goal_llm.rs` - Added episode retrieval and context injection before goal plan LLM call
- `crates/amux-daemon/src/agent/system_prompt.rs` - Added episodic_context parameter to build_system_prompt
- `crates/amux-daemon/src/agent/agent_loop.rs` - Updated both build_system_prompt call sites with None episodic_context

## Decisions Made
- FTS5 over-fetch 3x limit then re-rank with recency weighting -- pure BM25 doesn't account for temporal relevance, over-fetching ensures the re-ranking pool has enough candidates
- Exponential decay with half-life ~14 days (decay constant -0.05) -- provides smooth transition from "very relevant" to "somewhat relevant" without cliff effects
- Severity labels (WARNING for failures, REFERENCE for successes) in formatted context -- makes failure episodes visually prominent so the LLM prioritizes avoiding past mistakes
- Episodic context injected into goal planning prompt (goal_llm.rs) rather than always in system prompt -- keeps the system prompt lean for normal chat, only goal planning gets episodic context
- regex crate used for entity extraction from GoalRunStep instructions -- regex already in workspace deps, simple pattern for file path detection

## Deviations from Plan

None -- plan executed exactly as written.

## Issues Encountered
- Pre-existing test failures in `plugin::loader::tests` (2 tests) remain unchanged from main branch. Not related to episodic memory work.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None - all retrieval functions are fully implemented, all wiring is complete, all tests pass.

## Next Phase Readiness
- Retrieval engine is complete and ready for Plan 03 (counter-who and negative knowledge)
- record_session_end_episode is implemented and available for wiring into session lifecycle (not yet called -- Plan 03 or future work)
- build_system_prompt episodic_context parameter is ready for direct injection when thread-level episodic context is desired (currently None in agent_loop, active in goal planning path)

## Self-Check: PASSED

All 7 files verified present. Both task commits (3f23ca2, 4a007e4) verified in git log. SUMMARY.md exists.

---
*Phase: 01-memory-foundation*
*Completed: 2026-03-27*
