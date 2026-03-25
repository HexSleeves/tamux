---
phase: 18-oauth2-flow
plan: 02
subsystem: auth
tags: [oauth2, pkce, token-refresh, aes-gcm, handlebars, ipc, plugin-auth]

# Dependency graph
requires:
  - phase: 18-oauth2-flow
    plan: 01
    provides: "AES-256-GCM crypto module, credential persistence, OAuth2 IPC messages, AuthExpired error"
  - phase: 17-api-proxy
    provides: "api_call() proxy pipeline, template rendering, PluginApiError enum"
  - phase: 14-plugin-manifest
    provides: "AuthSection struct, PluginManager, PluginPersistence"
provides:
  - "OAuth2 authorization code + PKCE flow module (start, callback, exchange, refresh)"
  - "Server IPC handler for PluginOAuthStart -> PluginOAuthUrl -> PluginOAuthComplete"
  - "api_call() with automatic token lifecycle (load, check expiry, refresh, inject)"
  - "Template context extended with auth.access_token for OAuth plugins"
  - "Per-plugin Mutex for serializing concurrent token refresh attempts"
affects: [18-03-PLAN, 20-gmail-calendar-validation]

# Tech tracking
tech-stack:
  added: [oauth2 5.0.0]
  patterns: [typestate-client-builder, per-plugin-refresh-mutex, double-check-after-lock, channel-free-inline-flow]

key-files:
  created:
    - crates/amux-daemon/src/plugin/oauth2.rs
  modified:
    - crates/amux-daemon/src/plugin/mod.rs
    - crates/amux-daemon/src/plugin/template.rs
    - crates/amux-daemon/src/server.rs
    - crates/amux-daemon/Cargo.toml

key-decisions:
  - "oauth2 5.0.0 typestate: separate client builds per function to satisfy HasAuthUrl/HasTokenUrl type constraints"
  - "Inline OAuth flow in handler (not spawned) since each client runs in own task; 5-min timeout prevents blocking"
  - "60-second refresh threshold rather than 80% TTL calculation since original TTL is not stored"
  - "exchange_code takes OAuthFlowState (not separate config) to access stashed config and redirect_uri"
  - "build_context() third param is Option<Map> for backward compat; None for non-OAuth plugins"

patterns-established:
  - "oauth2 5.0.0 typestate: build client with only the endpoints needed (set_auth_uri for auth, set_token_uri for exchange)"
  - "Double-check-after-lock: re-check credential expiry after acquiring per-plugin refresh lock"
  - "Template context auth injection: auth.access_token available in Handlebars templates for OAuth plugins"

requirements-completed: [AUTH-01, AUTH-02, AUTH-04, AUTH-05]

# Metrics
duration: 13min
completed: 2026-03-25
---

# Phase 18 Plan 02: OAuth2 Flow Engine Summary

**OAuth2 authorization code + PKCE flow with ephemeral callback listener, automatic token refresh in api_call(), and auth.access_token template injection**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-25T07:38:55Z
- **Completed:** 2026-03-25T07:51:54Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Created oauth2.rs module with full OAuth2 authorization code + PKCE flow lifecycle (start, callback, exchange, refresh)
- Wired PluginOAuthStart IPC handler in server.rs: sends auth URL, awaits callback, stores tokens, sends completion
- Extended api_call() with automatic token lifecycle: load encrypted token, check expiry, refresh if needed, inject into template context
- Added per-plugin Mutex for serializing concurrent refresh attempts (Pitfall 7 prevention)
- 8 unit tests for callback parsing, URL construction, PKCE verification, and state validation

## Task Commits

Each task was committed atomically:

1. **Task 1: OAuth2 flow module with PKCE, callback listener, and token exchange** - `fb44377` (feat)
2. **Task 2: Server OAuth handler, api_call() token injection + refresh, template context** - `e46c509` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/plugin/oauth2.rs` - OAuth2 flow lifecycle: start_oauth_flow, await_callback, exchange_code, refresh_access_token with 8 tests
- `crates/amux-daemon/src/plugin/mod.rs` - start_oauth_flow_for_plugin, complete_oauth_flow, get_oauth_context_with_refresh, try_refresh_token, per-plugin refresh Mutex
- `crates/amux-daemon/src/plugin/template.rs` - build_context extended with optional auth map parameter
- `crates/amux-daemon/src/server.rs` - PluginOAuthStart handler with full flow (URL -> callback -> exchange -> store -> complete)
- `crates/amux-daemon/Cargo.toml` - Added oauth2 5.0.0 dependency with reqwest feature

## Decisions Made
- Used oauth2 5.0.0 typestate builder pattern: separate client construction per function (set_auth_uri for start_oauth_flow, set_token_uri for exchange_code/refresh) to satisfy the crate's compile-time endpoint requirement checks
- OAuth flow runs inline in the per-client handler task rather than spawned, since each client connection already runs in its own tokio task and the 5-minute callback timeout prevents indefinite blocking
- Used 60-second refresh threshold instead of 80% TTL calculation because the original TTL is not stored alongside the credential; 60s is conservative enough for typical 3600s tokens
- Stashed OAuthFlowConfig inside OAuthFlowState so exchange_code doesn't need the config passed separately
- build_context() third parameter is Option<Map> for backward compatibility -- non-OAuth plugins pass None

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] oauth2 5.0.0 typestate API required separate client builds**
- **Found during:** Task 1 (OAuth2 flow module)
- **Issue:** oauth2 5.0.0 uses typestate generics where `authorize_url()` requires `EndpointSet` for auth URL and `exchange_code()` requires `EndpointSet` for token URL. A shared `build_client()` helper cannot return a single type satisfying both.
- **Fix:** Removed shared build_client helper; each function builds its own client with only the required endpoint set
- **Files modified:** `crates/amux-daemon/src/plugin/oauth2.rs`
- **Verification:** All 8 tests pass, cargo build clean
- **Committed in:** fb44377 (Task 1 commit)

**2. [Rule 1 - Bug] chrono DateTime<FixedOffset> vs DateTime<Utc> subtraction**
- **Found during:** Task 2 (token expiry check)
- **Issue:** `chrono::DateTime::parse_from_rfc3339()` returns `DateTime<FixedOffset>` which cannot be subtracted from `DateTime<Utc>` directly
- **Fix:** Added `.with_timezone(&chrono::Utc)` conversion before subtraction
- **Files modified:** `crates/amux-daemon/src/plugin/mod.rs`
- **Verification:** cargo build clean, no type errors
- **Committed in:** e46c509 (Task 2 commit)

**3. [Rule 3 - Blocking] exchange_code signature changed to take OAuthFlowState instead of separate config**
- **Found during:** Task 1/2 integration
- **Issue:** exchange_code needs the redirect_uri from the flow state and the PKCE verifier, plus the config for building the token exchange client. Passing config + state separately creates duplication.
- **Fix:** Stashed config inside OAuthFlowState; exchange_code takes only state + code
- **Files modified:** `crates/amux-daemon/src/plugin/oauth2.rs`
- **Verification:** API is cleaner, all tests pass
- **Committed in:** fb44377 (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** All auto-fixes necessary for correctness. exchange_code signature change is a minor API improvement. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations.

## User Setup Required
None - no external service configuration required. Plugin OAuth flows are triggered by users configuring client_id/client_secret in plugin settings.

## Next Phase Readiness
- OAuth2 flow fully operational end-to-end: IPC start -> callback listener -> token exchange -> encrypted storage -> automatic refresh -> template injection
- Phase 18-03 (frontend wiring) can now connect UI buttons to PluginOAuthStart IPC
- Phase 20 (Gmail/Calendar validation) can test full OAuth flow with Google credentials

## Self-Check: PASSED

All files exist, all commits verified:
- crates/amux-daemon/src/plugin/oauth2.rs: FOUND
- crates/amux-daemon/src/plugin/mod.rs: FOUND
- crates/amux-daemon/src/plugin/template.rs: FOUND
- crates/amux-daemon/src/server.rs: FOUND
- Task 1 commit fb44377: FOUND
- Task 2 commit e46c509: FOUND

---
*Phase: 18-oauth2-flow*
*Completed: 2026-03-25*
