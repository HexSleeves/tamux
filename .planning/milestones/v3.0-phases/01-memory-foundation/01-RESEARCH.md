# Phase 1: Memory Foundation - Research

**Researched:** 2026-03-26
**Domain:** Episodic memory, counter-who self-model, negative knowledge constraint graph in a Rust daemon with SQLite/FTS5
**Confidence:** HIGH

## Summary

Phase 1 builds three interconnected subsystems -- episodic memory store, counter-who persistent self-model, and negative knowledge constraint graph -- as in-process extension modules within the existing `AgentEngine` struct. All three follow the established pattern: new `RwLock`-guarded fields on `AgentEngine`, behavior in `impl AgentEngine` blocks across files in a new `crates/amux-daemon/src/agent/episodic/` module directory.

The existing infrastructure provides everything needed with zero new runtime dependencies for core functionality. SQLite with FTS5 (already compiled in via bundled rusqlite 0.32.1), WAL mode (already enabled in `HistoryStore`), WORM hash-chain ledger files (already implemented for four telemetry kinds), `tokio-rusqlite` async wrapper (already in use), and `scrub_sensitive()` for PII redaction (already available in `crates/amux-daemon/src/scrub.rs`) form the complete foundation. The one optional addition is `rusqlite_migration` 1.3.1 for versioned schema management, which is compatible with the project's rusqlite 0.32.1.

The integration points are well-defined: episode recording hooks into `complete_goal_run()` and `fail_goal_run()` in `goal_planner.rs`; episode retrieval hooks into `request_goal_plan()` in `goal_llm.rs`; counter-who updates hook into the agent loop; negative knowledge injection hooks into `build_system_prompt()` in `system_prompt.rs`. The existing `record_provenance_event()` pattern provides the exact template for how to instrument goal boundaries. The `consolidation.rs` idle-time pattern provides the template for TTL-based episode cleanup and FTS5 index maintenance.

**Primary recommendation:** Build all three subsystems (episodic store, counter-who, negative knowledge) as a single `episodic/` module directory with 7-8 submodule files, following the exact same patterns used by `learning/`, `liveness/`, and `metacognitive/`. Use `CREATE TABLE IF NOT EXISTS` for new tables (consistent with existing `init_schema()` pattern) plus `rusqlite_migration` 1.3.1 for forward-looking schema versioning. Use `detail=column` for the FTS5 virtual table (no phrase-proximity queries needed, ~50% smaller index). Record episodes only at goal boundaries and session-end, never mid-session.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EPIS-01 | Agent stores structured episode records automatically on goal start and completion | `complete_goal_run()` and `fail_goal_run()` in `goal_planner.rs` provide exact hook points; `GoalRun` struct has all needed fields; `record_provenance_event()` shows the instrumentation pattern |
| EPIS-02 | Agent stores causal chain data linking failures to root causes | `CausalTrace` and `CausalFactor` types already exist in `learning/traces.rs`; `persist_goal_plan_causal_trace()` already captures plan decisions; extend with failure root-cause chains |
| EPIS-03 | Agent proactively surfaces top 5 relevant past episodes before planning | `request_goal_plan()` in `goal_llm.rs` is the injection point; FTS5 BM25 retrieval against `episodes_fts` provides <100ms retrieval; inject as context into the planning prompt |
| EPIS-04 | FTS5 episodic index supports temporal retrieval | FTS5 MATCH query combined with `WHERE created_at >= ?` filter on the content table; `created_at` column with index provides temporal scoping |
| EPIS-05 | FTS5 episodic index supports entity-aware retrieval | `entities` column as JSON array in episodes table, indexed by FTS5; structured entity tags enable both FTS5 MATCH and SQL `json_each()` queries |
| EPIS-06 | FTS5 episodic index supports causal retrieval | FTS5 MATCH on `root_cause` column + `episode_links` table with `link_type='caused_by'` enables causal chain traversal |
| EPIS-07 | Episode links connect related goals | `episode_links` table with `link_type` enum (retry_of, builds_on, contradicts, supersedes) and bidirectional references |
| EPIS-08 | Session headers with auto-generated summary and tags on session end | Extend `request_goal_reflection()` pattern from `goal_llm.rs` to produce session-end episode summaries with entity tags |
| EPIS-09 | Privacy controls: opt-out, per-session suppression, TTL, PII scrubbing | `scrub_sensitive()` in `scrub.rs` handles PII; `expires_at` column handles TTL; per-session opt-out as a thread metadata flag; `EpisodicConfig` in `AgentConfig` for global controls |
| EPIS-10 | Retrieval has hard cap (max 5 episodes, max token budget) | Enforce `LIMIT 5` in FTS5 queries and a `MAX_EPISODE_INJECTION_TOKENS` constant (suggested: 1500 tokens) |
| EPIS-11 | Episodes are WORM -- append-only, never edited | Use existing WORM ledger pattern (`worm_append` in `history.rs`); add "episodic" as a new WORM chain kind; corrections are new episodes with `link_type='supersedes'` |
| CWHO-01 | Background self-model tracks what agent is doing, changed, tried | `counter_who_state` table + in-memory `CounterWhoState` struct updated on significant agent_loop state changes |
| CWHO-02 | Counter-who detects repeated approaches and suggests pivots | Track approach signatures (tool+args hash sequences) in `tried_approaches`; when 3+ variants of same approach fail, inject pivot suggestion into planning context |
| CWHO-03 | Counter-who tracks operator corrections and flags persistent patterns | Hook into operator correction detection (approval rejections, manual overrides) in `agent_loop.rs`; store correction patterns in counter-who state |
| CWHO-04 | Counter-who state persists across sessions within goal run, rehydrates from episodic store | Persist to `counter_who_state` table at session-end / goal-run boundaries; restore relevant subset at session-start based on active goal-run context |
| NKNO-01 | Constraint graph stores ruled-out approaches with reasons | `negative_knowledge` table with `constraint_type` enum (dead, dying, impossible, suspicious) + structured `subject` and `description` fields |
| NKNO-02 | Constraint entries include class of solutions eliminated | `solution_class` field on negative_knowledge entries; constraints apply to all approaches sharing the same assumption/dependency, not just the specific approach that failed |
| NKNO-03 | Agent consults constraint graph before planning | Query `negative_knowledge WHERE valid_until IS NULL OR valid_until > now()` by entity overlap with goal text; inject as "do not attempt" constraints into planning prompt via `build_system_prompt()` |
| NKNO-04 | Constraint entries have TTL-based expiry (default 30 days) | `valid_until` column with default `created_at + 30*86400*1000`; consolidation tick prunes expired entries; configurable via `EpisodicConfig.constraint_ttl_days` |
</phase_requirements>

## Standard Stack

### Core (Already Present -- No Changes Required)

| Technology | Version | Purpose | Notes |
|------------|---------|---------|-------|
| `rusqlite` | 0.32.1 (bundled) | Episode storage, FTS5 index, negative knowledge, counter-who state | FTS5 compiled in via `-DSQLITE_ENABLE_FTS5` in bundled builds. Already used for `history_fts` and `context_archive_fts`. |
| `tokio-rusqlite` | 0.6 | Async SQLite wrapper for non-blocking episode writes | Already in daemon Cargo.toml. All episode writes go through `.call()` closures. |
| `tokio` | 1.x (full) | Async runtime | Unchanged. Episode write path spawns as background tasks. |
| `serde` + `serde_json` | 1.x | Serialization for episode types, entities JSON, counter-who state | All new types derive `Serialize`/`Deserialize`. JSON arrays stored as `TEXT` columns. |
| `sha2` | 0.10 | WORM integrity for episodic ledger | Existing `hex_hash()` and `worm_append()` pattern in `history.rs`. |
| `uuid` | 1.x (v4, serde) | Episode IDs, link IDs, constraint IDs | Existing `Uuid::new_v4()` pattern used throughout. |
| `chrono` | 0.4 | TTL calculations, temporal retrieval | Already in daemon deps. |
| `tracing` | 0.1 | Structured logging with spans for episode operations | Unchanged. New modules get span instrumentation. |
| `anyhow` + `thiserror` | 1.x / 2.x | Error handling | Unchanged. `?` propagation with `anyhow::Context`. |
| `regex` | 1 | PII scrubbing via existing `scrub_sensitive()` | Already in `scrub.rs`. |

### New Dependencies

| Technology | Version | Purpose | Why Standard |
|------------|---------|---------|--------------|
| `rusqlite_migration` | 1.3.1 | Versioned schema migrations for new episodic tables | Compatible with project's rusqlite 0.32.1 (`^0.32.1` dependency). Uses `PRAGMA user_version` -- lightweight. Supports `CREATE TABLE` + `ALTER TABLE` in ordered migration steps. Prevents schema drift across daemon versions. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `rusqlite_migration` | Continue with `CREATE TABLE IF NOT EXISTS` only | Existing pattern works for additive tables but cannot handle column renames, index changes, or table modifications in future versions. Since Phase 1 adds 4+ new tables, now is the right time to adopt structured migrations. However, if dependency minimalism is preferred, the existing pattern suffices for Phase 1's needs. |
| FTS5 `detail=column` | FTS5 `detail=full` (default) | `detail=full` supports phrase-proximity queries and positional data. Episodic retrieval does not need phrase queries. `detail=column` reduces index size by ~50% with no functional loss for keyword + temporal + entity queries. |

**Installation (if adding `rusqlite_migration`):**
```toml
# In crates/amux-daemon/Cargo.toml [dependencies]
rusqlite_migration = "1.3"
```

**Version verification:** rusqlite_migration 1.3.1 confirmed compatible with rusqlite 0.32.1 via docs.rs dependency listing (constraint: `^0.32.1`).

## Architecture Patterns

### Recommended Module Structure

```
crates/amux-daemon/src/agent/episodic/
    mod.rs              # Public API, re-exports, EpisodicStore struct definition
    schema.rs           # SQLite CREATE TABLE statements, migration definitions
    store.rs            # impl AgentEngine: record_episode(), retrieve_relevant_episodes()
    retrieval.rs        # FTS5 query building, BM25 ranking, temporal/entity/causal filtering
    negative_knowledge.rs  # Constraint CRUD, TTL checking, entity-scoped lookup
    counter_who.rs      # CounterWhoState struct, update/persist/restore methods
    links.rs            # Episode-to-episode link creation and traversal
    privacy.rs          # PII scrubbing wrapper, session suppression, TTL enforcement
```

### Pattern 1: Extension Module (MUST follow)

**What:** New `episodic/` directory as a public submodule of `agent/`, with `impl AgentEngine` blocks in each file and an `RwLock`-guarded field on `AgentEngine`.

**When:** Always -- this is the ONLY pattern used in the codebase for agent subsystems.

**Example (following existing patterns exactly):**
```rust
// In crates/amux-daemon/src/agent/mod.rs -- add module declaration
pub mod episodic;

// In crates/amux-daemon/src/agent/engine.rs -- add field
pub struct AgentEngine {
    // ... existing fields ...
    pub(super) episodic_store: RwLock<episodic::EpisodicStore>,
}

// In crates/amux-daemon/src/agent/episodic/store.rs
use super::*;

impl AgentEngine {
    pub(super) async fn record_episode(&self, episode: Episode) -> Result<()> {
        let scrubbed = self.scrub_episode(&episode);
        self.history.insert_episode(&scrubbed).await?;
        let mut store = self.episodic_store.write().await;
        store.update_counter_who(&scrubbed);
        // Fire-and-forget WORM append
        let _ = self.history.worm_append_telemetry(
            "episodic",
            &serde_json::to_value(&scrubbed)?,
        ).await;
        Ok(())
    }
}
```

### Pattern 2: Non-Blocking Write Path via tokio-rusqlite

**What:** All SQLite writes go through `self.conn.call(move |conn| { ... })` which executes on a background thread via mpsc channel.

**When:** Every episode insert, counter-who update, and negative knowledge write.

**Example (following existing `HistoryStore` pattern from `history.rs`):**
```rust
// In HistoryStore -- new method
pub async fn insert_episode(&self, episode: &Episode) -> Result<()> {
    let ep = episode.clone();
    self.conn.call(move |conn| {
        conn.execute(
            "INSERT INTO episodes (id, goal_run_id, thread_id, session_id, episode_type, \
             summary, outcome, root_cause, entities, causal_chain, duration_ms, tokens_used, \
             confidence, created_at, expires_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                ep.id, ep.goal_run_id, ep.thread_id, ep.session_id,
                ep.episode_type, ep.summary, ep.outcome, ep.root_cause,
                ep.entities_json, ep.causal_chain_json, ep.duration_ms,
                ep.tokens_used, ep.confidence, ep.created_at, ep.expires_at,
            ],
        ).call_err()?;
        Ok(())
    }).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
```

### Pattern 3: System Prompt Injection for LLM Context

**What:** Episodic context and negative constraints are injected as sections in the system prompt, assembled in `build_system_prompt()`.

**When:** Before goal planning (episodes + constraints), during agent loop turns (counter-who awareness).

**Injection point:** `system_prompt.rs` `build_system_prompt()` already accepts `operational_context` and `causal_guidance` as optional parameters. Add `episodic_context` and `negative_constraints` as new optional parameters.

**Example format for injected episodic context:**
```
## Past Experience (Episodic Memory)
WARNING: A similar goal "deploy auth service" failed 3 days ago.
  Root cause: SSL certificate was expired on staging server.
  Approach tried: direct deployment without cert check.
  Link: episode_abc123

REFERENCE: "set up monitoring stack" succeeded 1 week ago.
  Key insight: used docker-compose with health checks.
  Link: episode_def456

## Ruled-Out Approaches (Negative Knowledge)
DO NOT attempt: manual certificate renewal via CLI on staging
  Reason: staging server requires automated cert manager (constraint from 2026-03-20)
  Expires: 2026-04-19
```

### Pattern 4: Goal Boundary Hooks for Episode Recording

**What:** Episodes are recorded at goal completion/failure boundaries, mirroring the existing `record_provenance_event()` calls.

**When:** Inside `complete_goal_run()` and `fail_goal_run()` in `goal_planner.rs`.

**Integration (add after existing `record_provenance_event()` calls):**
```rust
// In complete_goal_run(), after self.record_provenance_event(...)
self.record_goal_episode(&updated, EpisodeOutcome::Success).await;

// In fail_goal_run(), after self.record_provenance_event(...)
self.record_goal_episode(&updated, EpisodeOutcome::Failure).await;
```

### Pattern 5: WORM Ledger for Episode Immutability

**What:** Every episode write is also appended to a `"episodic"` WORM ledger file, following the existing pattern of `"operational"`, `"cognitive"`, `"contextual"`, and `"provenance"` ledgers.

**When:** On every `record_episode()` call.

**How:** Call `self.history.worm_append_telemetry("episodic", &payload)` using the existing WORM infrastructure. The `verify_worm_integrity()` function in `history.rs` already iterates over `ledger_kinds` -- add `"episodic"` to that array.

### Anti-Patterns to Avoid

- **Recording episodes mid-session or per-tool-call:** Creates episode table bloat and noisy FTS5 retrieval. Record ONLY at goal boundaries and session-end. (See Pitfall 1, 4 in PITFALLS.md)
- **Using LLM calls for negative knowledge constraint checking:** Constraint lookup must be a SQL query with entity tag matching, not an LLM relevance assessment. LLM calls add latency and cost to every planning step. (See Pitfall 5 in PITFALLS.md)
- **Injecting all matching episodes into planning context:** Hard cap at 5 episodes, enforce a token budget of 1500 tokens. More episodes = more context pollution. (See Pitfall 1 in PITFALLS.md)
- **Storing raw session transcripts in episodes:** Store structured summaries with extracted entities, error messages, and root causes. Raw transcripts are too large and too noisy. (See Pitfall 7 in PITFALLS.md)
- **Separate SQLite database for episodic data:** All tables go in the existing `~/.tamux/history.db`. The daemon is single-process; separate databases add connection management overhead with no benefit.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Full-text search | Custom string matching or regex-based search over episode summaries | SQLite FTS5 (already compiled in) | BM25 ranking, sub-millisecond query on <10K rows, automatic index updates via triggers |
| WORM append-only ledger | Custom append-only file format | Existing `worm_append_telemetry()` in `history.rs` | Hash-chain integrity already implemented, `verify_worm_integrity()` covers all ledger kinds |
| PII scrubbing | Custom regex patterns for sensitive data | Existing `scrub_sensitive()` in `scrub.rs` | Already handles AWS keys, GitHub tokens, API keys, bearer tokens, hex secrets, private keys |
| Schema versioning | Manual `CREATE TABLE IF NOT EXISTS` with version checks | `rusqlite_migration` 1.3.1 | Ordered migrations, rollback capability, `PRAGMA user_version` tracking, handles future schema changes |
| Async SQLite writes | Custom async wrapper or thread pool | Existing `tokio-rusqlite` 0.6 via `self.conn.call()` | Already used throughout `HistoryStore`, background thread with mpsc channel |
| Episode ID generation | Custom ID scheme | `uuid::Uuid::new_v4()` | Already used for all IDs in the codebase (threads, messages, tasks, goal runs) |
| Confidence band mapping | Custom threshold logic | Existing `confidence_band()` in `explanation.rs` | Already maps 0.0-1.0 to Confident/Likely/Uncertain/Guessing with tested thresholds |

**Key insight:** This phase requires almost no new library code. The existing infrastructure (FTS5, WORM, tokio-rusqlite, scrub_sensitive, confidence bands, provenance events) provides 80% of the plumbing. The work is writing ~800-1000 lines of new Rust module code that wires these existing primitives together in the episodic memory pattern.

## Common Pitfalls

### Pitfall 1: Context Pollution from Over-Eager Episode Retrieval

**What goes wrong:** Episodic memory retrieval injects too many "somewhat relevant" memories into the planning context, diluting the LLM's attention on the actual task.
**Why it happens:** FTS5 keyword matching without a relevance floor returns loosely related episodes. Everything looks "somewhat similar."
**How to avoid:**
- Hard cap: max 5 episodes, max 1500 injection tokens
- Multi-signal ranking: FTS5 BM25 score + temporal recency weight + entity overlap score
- Relevance floor: discard episodes with BM25 rank below a threshold (tune empirically)
- Track `retrieval_used` per injected episode in WORM audit; if >60% go unused, tighten retrieval
**Warning signs:** Token usage spikes after enabling episodic retrieval; agent responses reference irrelevant past sessions.

### Pitfall 2: SQLite Write Contention from Episode Writes

**What goes wrong:** Episode writes compete with existing WORM telemetry, command log, and thread history writes, all funneling through the same `tokio-rusqlite` connection.
**Why it happens:** SQLite is single-writer. `tokio-rusqlite` serializes all `.call()` invocations behind one background thread.
**How to avoid:**
- WAL mode already enabled (confirmed in `HistoryStore` initialization: `conn.pragma_update(None, "journal_mode", "WAL")?;`)
- Batch episode writes at goal boundaries (not per-tool-call) -- reduces write frequency to 1-2 per goal run
- Make episode WORM append fire-and-forget from the goal completion hot path
- If contention is observed under load, consider a second `tokio-rusqlite::Connection` for episodic writes (both connections write to same file; WAL allows concurrent reads)
**Warning signs:** `tokio-rusqlite` `.call()` P95 exceeds 50ms; WORM timestamp gaps correlate with episode writes.

### Pitfall 3: Negative Knowledge Constraint Graph Becomes a Tar Pit

**What goes wrong:** Constraint count grows unbounded; agent spends more time checking constraints than doing work; over-generalized constraints block valid approaches.
**Why it happens:** Every failure creates a constraint. Without TTL and scoping, constraints accumulate indefinitely.
**How to avoid:**
- TTL-based expiry: default 30 days (`valid_until = created_at + 30*86400*1000`)
- Entity-scoped lookup: index `negative_knowledge(subject)` and match by entity tags, not FTS5 similarity
- Cap active constraints at ~20 per problem domain; consolidation tick merges overlapping constraints
- Constraint checking is a SQL query with structured matching, never an LLM call
**Warning signs:** Planning latency grows with constraint count; agent rejects approaches operators know are valid.

### Pitfall 4: Counter-Who State Grows Stale or Unbounded

**What goes wrong:** Counter-who accumulates irrelevant state from old sessions; or is lost entirely on restart.
**Why it happens:** No decay function; no session-scoped restoration.
**How to avoid:**
- Persist counter-who to `counter_who_state` table at session-end and goal-run boundaries
- Restore the *relevant subset* at session-start based on current goal-run context, not the full historical state
- Decay: entries older than 7 days compress to one-line summaries; entries older than 30 days archived to episodic store
- Cap `tried_approaches` at 20 entries per goal-run context
**Warning signs:** Counter-who injection into prompts grows beyond 500 tokens; stale approaches listed for unrelated goals.

### Pitfall 5: FTS5 Index Bloat from Append-Only Records

**What goes wrong:** Episodes are WORM (never deleted), so the FTS5 index grows monotonically. Over months, query latency increases.
**Why it happens:** FTS5 `detail=full` indexes produce ~45% overhead of original data; append-only means no natural cleanup.
**How to avoid:**
- Use `detail=column` for the FTS5 virtual table (no phrase queries needed; ~50% smaller index than `detail=full`)
- Consolidation tick runs FTS5 `rebuild` during idle time (following existing `maybe_run_consolidation_if_idle()` pattern)
- TTL-based episode expiry (EPIS-09) naturally limits index size; expired episodes are deleted from the content table, and FTS5 `rebuild` removes stale index entries
- At <10K episodes, FTS5 queries remain <10ms even without optimization
**Warning signs:** FTS5 query latency exceeds 50ms; database file size growth rate exceeds data insertion rate.

### Pitfall 6: Episode Extraction Loses Critical Details

**What goes wrong:** Session-end episode summaries are too compressed, losing specific error messages, version numbers, and file paths that would be most valuable for future retrieval.
**Why it happens:** LLM summarization optimizes for brevity, not retrieval utility.
**How to avoid:**
- Structured extraction alongside narrative summaries: every episode includes (1) narrative summary, (2) structured entity list (files, tools, libraries, versions, error codes), (3) outcome, (4) verbatim error messages in `root_cause`
- Use the existing `GoalReflectionResponse` pattern from `goal_llm.rs` as the template -- it already extracts structured data from goal runs
- Never summarize error messages or stack traces -- extract them verbatim into the `root_cause` field
- Counter-who state at session-end signals what mattered (use it to prioritize what to extract)
**Warning signs:** FTS5 retrieval misses episodes that should be relevant; episode summaries are generic and lack technical specificity.

## Code Examples

### Episode SQLite Schema (new tables in `episodic/schema.rs`)

```sql
-- Core episode table
CREATE TABLE IF NOT EXISTS episodes (
    id TEXT PRIMARY KEY,
    goal_run_id TEXT,
    thread_id TEXT,
    session_id TEXT,
    episode_type TEXT NOT NULL,        -- 'goal_completion', 'goal_failure', 'session_end', 'discovery'
    summary TEXT NOT NULL,
    outcome TEXT NOT NULL,             -- 'success', 'failure', 'partial', 'abandoned'
    root_cause TEXT,                   -- for failures: verbatim error or root cause description
    entities TEXT,                     -- JSON array: ["file:src/main.rs", "tool:execute_command", "lib:reqwest"]
    causal_chain TEXT,                 -- JSON array of step->cause->effect chains
    solution_class TEXT,               -- for negative knowledge: class of approaches this applies to
    duration_ms INTEGER,
    tokens_used INTEGER,
    confidence REAL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER                 -- TTL-based expiry, NULL = never expires
);
CREATE INDEX IF NOT EXISTS idx_episodes_goal ON episodes(goal_run_id);
CREATE INDEX IF NOT EXISTS idx_episodes_thread ON episodes(thread_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_type_ts ON episodes(episode_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_outcome ON episodes(outcome, created_at DESC);

-- FTS5 full-text index over episode text fields
-- detail=column: no phrase-proximity queries, ~50% smaller index
CREATE VIRTUAL TABLE IF NOT EXISTS episodes_fts USING fts5(
    summary, entities, root_cause,
    content=episodes, content_rowid=rowid,
    detail=column
);

-- Triggers to keep FTS5 in sync with content table
CREATE TRIGGER IF NOT EXISTS episodes_ai AFTER INSERT ON episodes BEGIN
    INSERT INTO episodes_fts(rowid, summary, entities, root_cause)
    VALUES (new.rowid, new.summary, new.entities, new.root_cause);
END;
CREATE TRIGGER IF NOT EXISTS episodes_ad AFTER DELETE ON episodes BEGIN
    INSERT INTO episodes_fts(episodes_fts, rowid, summary, entities, root_cause)
    VALUES ('delete', old.rowid, old.summary, old.entities, old.root_cause);
END;

-- Episode-to-episode links
CREATE TABLE IF NOT EXISTS episode_links (
    id TEXT PRIMARY KEY,
    source_episode_id TEXT NOT NULL REFERENCES episodes(id),
    target_episode_id TEXT NOT NULL REFERENCES episodes(id),
    link_type TEXT NOT NULL,           -- 'retry_of', 'builds_on', 'contradicts', 'supersedes', 'caused_by'
    evidence TEXT,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_episode_links_source ON episode_links(source_episode_id);
CREATE INDEX IF NOT EXISTS idx_episode_links_target ON episode_links(target_episode_id);

-- Negative knowledge constraints
CREATE TABLE IF NOT EXISTS negative_knowledge (
    id TEXT PRIMARY KEY,
    episode_id TEXT REFERENCES episodes(id),
    constraint_type TEXT NOT NULL,     -- 'ruled_out', 'impossible_combination', 'known_limitation'
    subject TEXT NOT NULL,             -- what was tried (specific enough to match)
    solution_class TEXT,               -- broader class of approaches eliminated
    description TEXT NOT NULL,
    confidence REAL NOT NULL,
    valid_until INTEGER,               -- TTL expiry timestamp, NULL = no expiry
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_neg_knowledge_subject ON negative_knowledge(subject, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_neg_knowledge_valid ON negative_knowledge(valid_until);

-- Counter-who persistent self-model state
CREATE TABLE IF NOT EXISTS counter_who_state (
    id TEXT PRIMARY KEY,
    goal_run_id TEXT,
    thread_id TEXT,
    current_focus TEXT,                -- what the agent is currently working on
    recent_changes TEXT,               -- JSON array of recent state changes
    tried_approaches TEXT,             -- JSON array of {approach_hash, description, outcome, timestamp}
    correction_patterns TEXT,          -- JSON array of operator corrections
    active_constraints TEXT,           -- JSON array of active negative knowledge IDs
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_counter_who_goal ON counter_who_state(goal_run_id);
```

### Episode Rust Data Types (in `episodic/mod.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub goal_run_id: Option<String>,
    pub thread_id: Option<String>,
    pub session_id: Option<String>,
    pub episode_type: EpisodeType,
    pub summary: String,
    pub outcome: EpisodeOutcome,
    pub root_cause: Option<String>,
    pub entities: Vec<String>,
    pub causal_chain: Vec<CausalStep>,
    pub solution_class: Option<String>,
    pub duration_ms: Option<u64>,
    pub tokens_used: Option<u32>,
    pub confidence: Option<f64>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeType {
    GoalCompletion,
    GoalFailure,
    SessionEnd,
    Discovery,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeOutcome {
    Success,
    Failure,
    Partial,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalStep {
    pub step: String,      // what was done
    pub cause: String,     // why it happened
    pub effect: String,    // what resulted
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeLink {
    pub id: String,
    pub source_episode_id: String,
    pub target_episode_id: String,
    pub link_type: LinkType,
    pub evidence: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    RetryOf,
    BuildsOn,
    Contradicts,
    Supersedes,
    CausedBy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativeConstraint {
    pub id: String,
    pub episode_id: Option<String>,
    pub constraint_type: ConstraintType,
    pub subject: String,
    pub solution_class: Option<String>,
    pub description: String,
    pub confidence: f64,
    pub valid_until: Option<u64>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    RuledOut,
    ImpossibleCombination,
    KnownLimitation,
}

/// In-memory counter-who state for the current session/goal-run context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CounterWhoState {
    pub goal_run_id: Option<String>,
    pub thread_id: Option<String>,
    pub current_focus: Option<String>,
    pub recent_changes: Vec<String>,
    pub tried_approaches: Vec<TriedApproach>,
    pub correction_patterns: Vec<CorrectionPattern>,
    pub active_constraint_ids: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriedApproach {
    pub approach_hash: String,
    pub description: String,
    pub outcome: EpisodeOutcome,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionPattern {
    pub pattern: String,
    pub correction_count: u32,
    pub last_correction_at: u64,
}

/// Top-level store struct held as RwLock field on AgentEngine.
#[derive(Debug, Default)]
pub struct EpisodicStore {
    pub counter_who: CounterWhoState,
    /// Cached active negative knowledge constraints (refreshed periodically).
    pub cached_constraints: Vec<NegativeConstraint>,
    /// Configuration
    pub config: EpisodicConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicConfig {
    pub enabled: bool,
    pub episode_ttl_days: u64,        // default 90
    pub constraint_ttl_days: u64,     // default 30
    pub max_retrieval_episodes: usize, // default 5
    pub max_injection_tokens: usize,   // default 1500
    pub per_session_suppression: bool, // default false (operator can override per-session)
}

impl Default for EpisodicConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            episode_ttl_days: 90,
            constraint_ttl_days: 30,
            max_retrieval_episodes: 5,
            max_injection_tokens: 1500,
            per_session_suppression: false,
        }
    }
}
```

### FTS5 Retrieval Query Pattern (in `episodic/retrieval.rs`)

```rust
/// Retrieve relevant past episodes for a goal query.
/// Returns at most `limit` episodes, ranked by BM25 + recency.
pub async fn search_episodes_fts(
    conn: &tokio_rusqlite::Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<Episode>> {
    let query = query.to_string();
    conn.call(move |conn| {
        // FTS5 MATCH with BM25 ranking, filtered by non-expired
        let mut stmt = conn.prepare(
            "SELECT e.id, e.goal_run_id, e.thread_id, e.session_id, e.episode_type,
                    e.summary, e.outcome, e.root_cause, e.entities, e.causal_chain,
                    e.solution_class, e.duration_ms, e.tokens_used, e.confidence,
                    e.created_at, e.expires_at,
                    bm25(episodes_fts) as rank
             FROM episodes e
             JOIN episodes_fts ON e.rowid = episodes_fts.rowid
             WHERE episodes_fts MATCH ?1
               AND (e.expires_at IS NULL OR e.expires_at > ?2)
             ORDER BY rank
             LIMIT ?3"
        )?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let rows = stmt.query_map(
            rusqlite::params![query, now, limit as i64],
            |row| { /* map to Episode */ Ok(()) }
        )?;
        // ... collect and return
        Ok(Vec::new())
    }).await.map_err(|e| anyhow::anyhow!("{e}"))
}
```

### AgentConfig Extension (in `types.rs`)

```rust
// Add to AgentConfig struct
pub struct AgentConfig {
    // ... existing fields ...

    /// Episodic memory configuration (Phase 1: Memory Foundation).
    #[serde(default)]
    pub episodic: EpisodicConfig,
}
```

### New AgentEvent Variants (in `types.rs`)

```rust
pub enum AgentEvent {
    // ... existing variants ...

    /// Emitted when a new episode is recorded.
    EpisodeRecorded {
        episode_id: String,
        episode_type: String,
        outcome: String,
        summary: String,
    },
    /// Emitted when counter-who detects a repeated approach pattern.
    CounterWhoAlert {
        thread_id: String,
        pattern: String,      // description of the repeated pattern
        attempt_count: u32,
        suggestion: String,   // pivot suggestion
    },
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| FTS5 `detail=full` default | `detail=column` for non-phrase workloads | Always available, often overlooked | ~50% index size reduction with no functional loss for keyword+entity queries |
| Raw session transcript storage | Structured extraction + narrative summary | 2025 research (Hindsight 20/20 paper) | Narratives outperform fragments for downstream retrieval quality |
| LLM self-reported confidence | Hybrid structural + verbal + historical calibration | 2025 (KDD survey, ICLR 2025) | LLMs are systematically overconfident; structural signals required |
| Global negative knowledge graph | TTL-scoped, entity-tagged constraints with cap | 2025 (constraint satisfaction research) | Unbounded constraint graphs paralyze agents; TTL + caps prevent tar pit |

**Deprecated/outdated:**
- Vector embeddings for episodic retrieval: Explicitly out of scope per project decisions. FTS5 meets the <100ms constraint without embedding model dependency. Deferred to v3.1+ (AMEM-01).
- External agent memory frameworks (Mem0, Honcho as primary): tamux already has Honcho integration as optional cross-session memory. Episodic memory is a different layer -- structured goal-level records, not conversation memory.

## Open Questions

1. **FTS5 prefix index configuration**
   - What we know: FTS5 supports prefix indexes (`prefix='2 3'`) for faster prefix queries. The existing `history_fts` and `context_archive_fts` do not use prefix indexes.
   - What's unclear: Whether episodic queries will commonly use prefix matching (e.g., "depl*" matching "deploy", "deployment"). Entity-type-prefixed entities like "file:..." suggest prefix queries could be useful.
   - Recommendation: Start without prefix indexes. If query patterns show prefix usage, add them via migration. The cost of unused prefix indexes is wasted storage, not correctness.

2. **Episode extraction quality -- template vs LLM**
   - What we know: `request_goal_reflection()` already uses LLM structured output for goal-run summaries. Template-based extraction is cheaper but less nuanced.
   - What's unclear: Whether template-based extraction (pulling fields directly from `GoalRun` struct) produces sufficiently rich summaries, or whether an LLM call is needed for episode extraction.
   - Recommendation: Start with template-based extraction from `GoalRun` fields (title, goal, outcome, failure_cause, reflection_summary, step summaries). The reflection LLM call already runs at goal completion -- piggyback episode extraction onto its output rather than making a separate LLM call. Add structured fields (entities, root_cause) to the `GoalReflectionResponse` schema.

3. **Counter-who update frequency**
   - What we know: Agent loop ticks every ~50ms in TUI. Updating counter-who on every tick is too frequent.
   - What's unclear: The right granularity -- per-tool-call, per-message, or per-step.
   - Recommendation: Update counter-who on (1) tool call completion (success or failure), (2) goal step transitions, (3) operator messages. This provides enough signal for repeat detection without overwhelming the state.

4. **Write contention benchmark needed**
   - What we know: WAL mode is enabled. Episode writes happen at goal boundaries (low frequency). WORM appends are synchronous file I/O.
   - What's unclear: Whether the single `tokio-rusqlite` connection handles episodic writes without observable contention under realistic goal-run workloads.
   - Recommendation: Instrument with `tracing` spans. If P95 write latency exceeds 50ms during testing, split to a second connection. Defer this optimization until contention is measured.

## Project Constraints (from CLAUDE.md)

The following directives from CLAUDE.md constrain implementation:

- **Tech stack:** Rust daemon -- all episodic memory code is Rust, in `crates/amux-daemon/`
- **Local-first:** All episode data stored in local SQLite (`~/.tamux/history.db`). No cloud dependency, no phone-home, no account required.
- **Provider-agnostic:** Episode extraction must not depend on a specific LLM provider. Template-based extraction preferred; if LLM is used, it goes through the existing provider-agnostic `run_goal_structured()` pipeline.
- **Backward compatibility:** New tables use `CREATE TABLE IF NOT EXISTS`. Existing `~/.tamux/` data directory structure unchanged. No breaking changes to IPC protocol (new message variants are additive).
- **Single binary:** No new external processes. Episodic store is in-process within the daemon.
- **Platform parity:** SQLite is cross-platform. No OS-specific code needed for episodic memory.
- **GSD Workflow:** Implementation follows GSD phases -- plan before edit.
- **Daemon reads config from DB, NOT config.json:** New `EpisodicConfig` fields must be read/written through the existing config IPC mechanism (confirmed in MEMORY.md: "daemon reads config from DB, NOT config.json").

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `crates/amux-daemon/src/history.rs` (6554 lines) -- HistoryStore, init_schema, FTS5 usage, WORM implementation, WAL mode confirmation
- Codebase analysis: `crates/amux-daemon/src/agent/engine.rs` -- AgentEngine struct with all RwLock/Mutex fields, constructor pattern
- Codebase analysis: `crates/amux-daemon/src/agent/goal_planner.rs` -- complete_goal_run(), fail_goal_run(), record_provenance_event() hook points
- Codebase analysis: `crates/amux-daemon/src/agent/goal_llm.rs` -- request_goal_plan(), request_goal_reflection(), run_goal_structured() pipeline
- Codebase analysis: `crates/amux-daemon/src/agent/system_prompt.rs` -- build_system_prompt() injection points for operational_context, causal_guidance
- Codebase analysis: `crates/amux-daemon/src/agent/consolidation.rs` -- idle-time consolidation pattern, decay confidence computation
- Codebase analysis: `crates/amux-daemon/src/agent/learning/traces.rs` -- CausalTrace, CausalFactor, DecisionType, existing causal chain infrastructure
- Codebase analysis: `crates/amux-daemon/src/agent/explanation.rs` -- ConfidenceBand, confidence_band() function
- Codebase analysis: `crates/amux-daemon/src/scrub.rs` -- scrub_sensitive() PII scrubbing implementation
- Codebase analysis: `crates/amux-daemon/Cargo.toml` -- rusqlite 0.32 bundled, tokio-rusqlite 0.6 confirmed
- Codebase analysis: `Cargo.lock` -- rusqlite 0.32.1 resolved version confirmed
- [SQLite FTS5 Extension](https://www.sqlite.org/fts5.html) -- detail levels, BM25 ranking, external content tables
- [rusqlite_migration 1.3.1 on docs.rs](https://docs.rs/rusqlite_migration/1.3.1/rusqlite_migration/) -- rusqlite ^0.32.1 dependency confirmed

### Secondary (MEDIUM confidence)
- Project-level research: `.planning/research/ARCHITECTURE.md` -- episodic store design, component boundaries, data flow
- Project-level research: `.planning/research/PITFALLS.md` -- 14 pitfalls with prevention strategies
- Project-level research: `.planning/research/STACK.md` -- zero-new-deps principle, technology decisions
- Project-level research: `.planning/research/SUMMARY.md` -- phase ordering rationale, dependency chain

### Tertiary (LOW confidence)
- [Hindsight is 20/20: Building Agent Memory](https://arxiv.org/html/2512.12818v1) -- narrative vs fragmented extraction (arxiv preprint, not peer-reviewed)
- [MAST Multi-Agent Failure Taxonomy](https://arxiv.org/pdf/2503.13657) -- coordination failure patterns (preprint)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all core dependencies already in codebase; `rusqlite_migration` 1.3.1 compatibility confirmed against Cargo.lock
- Architecture: HIGH -- all patterns copy existing codebase conventions exactly; integration points identified with line-number precision
- Pitfalls: HIGH -- backed by project-level research + codebase-specific analysis of write contention, FTS5 index behavior, and constraint graph scaling
- Schema design: HIGH -- follows existing `init_schema()` patterns exactly; FTS5 external content table pattern matches `context_archive_fts`

**Research date:** 2026-03-26
**Valid until:** 2026-04-25 (stable domain -- SQLite, Rust patterns don't change quickly)
