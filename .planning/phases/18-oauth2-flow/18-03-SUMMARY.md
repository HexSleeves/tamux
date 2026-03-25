---
phase: 18-oauth2-flow
plan: 03
subsystem: ui
tags: [oauth2, electron, tui, cli, ipc, plugin-auth]

# Dependency graph
requires:
  - phase: 18-02
    provides: daemon OAuth2 flow (start, exchange, refresh) and protocol messages
provides:
  - Electron PluginsTab Connect/Reconnect wired to daemon PluginOAuthStart IPC
  - Real auth_status display in Electron (not_configured/connected/expired)
  - PluginOAuthComplete event listener with automatic plugin list refresh
  - TUI auth_status column with color-coded real status from daemon
  - TUI Connect action triggering OAuth flow and opening browser
  - TUI OAuth completion handling with plugin list refresh
  - CLI bridge PluginOAuthUrl and PluginOAuthComplete message handling
  - auth_status field in CLI bridge JSON plugin output
affects: [20-gmail-calendar-validation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Electron plugin-oauth-complete dedicated IPC event channel"
    - "Bridge event.data fallback for plugin responses without data wrapper"

key-files:
  created: []
  modified:
    - frontend/src/types/amux-bridge.d.ts
    - frontend/src/lib/pluginStore.ts
    - frontend/src/components/settings-panel/PluginsTab.tsx
    - frontend/electron/main.cjs
    - frontend/electron/preload.cjs
    - crates/amux-tui/src/widgets/settings.rs
    - crates/amux-tui/src/app/settings_handlers.rs
    - crates/amux-tui/src/app/events.rs
    - crates/amux-tui/src/state/settings.rs
    - crates/amux-tui/src/state/mod.rs
    - crates/amux-tui/src/client.rs
    - crates/amux-tui/src/main.rs
    - crates/amux-cli/src/client.rs

key-decisions:
  - "plugin-oauth-complete sent as dedicated Electron IPC event (not agent-event) for clean separation"
  - "Electron main process opens browser via shell.openExternal on PluginOAuthUrl receipt"
  - "Bridge event resolution uses event.data ?? event fallback for backward compat with plugin responses"
  - "Plugin response types added to bridge pending resolution list (Rule 1 fix for pre-existing bug)"

patterns-established:
  - "Plugin OAuth IPC: renderer calls pluginOAuthStart, main process opens browser, renderer listens for plugin-oauth-complete"

requirements-completed: [AUTH-07]

# Metrics
duration: 13min
completed: 2026-03-25
---

# Phase 18 Plan 03: UI OAuth Wiring Summary

**Electron and TUI OAuth Connect buttons wired to daemon IPC with real auth_status display replacing Phase 16 hardcoded placeholders**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-25T07:55:56Z
- **Completed:** 2026-03-25T08:09:00Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments
- Electron Connect/Reconnect button triggers real OAuth flow via daemon IPC instead of hardcoded Google URL
- Auth badge shows real status from daemon (not_configured/connected/expired) in both Electron and TUI
- TUI Connect action opens browser and handles OAuth completion with status line feedback
- CLI bridge handles all new OAuth protocol messages for exhaustive match coverage

## Task Commits

Each task was committed atomically:

1. **Task 1: Electron PluginsTab OAuth wiring with real auth_status** - `b023ded` (feat)
2. **Task 2: TUI OAuth Connect action and CLI protocol handling** - `8b4b13e` (feat)

## Files Created/Modified
- `frontend/src/types/amux-bridge.d.ts` - Added pluginOAuthStart, onPluginOAuthComplete, auth_status to plugin types
- `frontend/src/lib/pluginStore.ts` - Added auth_status field, startOAuth action, OAuth listener initialization
- `frontend/src/components/settings-panel/PluginsTab.tsx` - Replaced hardcoded auth_status with real values, wired Connect button to startOAuth
- `frontend/electron/main.cjs` - Added plugin-oauth-start IPC handler, plugin-oauth-complete event forwarding, plugin response types in bridge resolution
- `frontend/electron/preload.cjs` - Added pluginOAuthStart and onPluginOAuthComplete bridge methods
- `crates/amux-tui/src/widgets/settings.rs` - Real auth_status display with color coding (OK=green, Expired=yellow, Setup=dim)
- `crates/amux-tui/src/app/settings_handlers.rs` - Connect button triggers PluginOAuthStart via DaemonCommand
- `crates/amux-tui/src/app/events.rs` - PluginOAuthUrl opens browser, PluginOAuthComplete refreshes list
- `crates/amux-tui/src/state/settings.rs` - Added auth_status field to PluginListItem
- `crates/amux-tui/src/state/mod.rs` - Added PluginOAuthStart to DaemonCommand enum
- `crates/amux-tui/src/client.rs` - Added plugin_oauth_start method, PluginOAuthUrl/Complete event types and handlers
- `crates/amux-tui/src/main.rs` - Added PluginOAuthStart dispatch to client
- `crates/amux-cli/src/client.rs` - Added auth_status to bridge JSON output, PluginOAuthUrl/Complete handlers, PluginOAuthStart bridge command

## Decisions Made
- Plugin OAuth complete sent as dedicated Electron IPC channel (`plugin-oauth-complete`) for clean separation from agent-event stream
- Electron main process opens browser via `shell.openExternal` rather than renderer-side `window.open` for proper system browser integration
- Bridge event resolution uses `event.data ?? event` fallback to handle both wrapped (status-response) and unwrapped (plugin) response formats
- 30-second timeout for plugin-oauth-start query (longer than default 5s due to OAuth flow setup time)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing plugin response types in bridge event routing list**
- **Found during:** Task 1 (Electron IPC wiring)
- **Issue:** Plugin response types (plugin-list-result, plugin-get-result, plugin-settings, plugin-action-result, plugin-test-connection-result) were missing from the bridge response type list in main.cjs, causing sendAgentQuery to always timeout for plugin operations
- **Fix:** Added all plugin response types to the pending resolution list
- **Files modified:** frontend/electron/main.cjs
- **Verification:** Build passes, IPC pattern now consistent
- **Committed in:** b023ded (Task 1 commit)

**2. [Rule 1 - Bug] Bridge event.data undefined for plugin responses**
- **Found during:** Task 1 (Electron IPC wiring)
- **Issue:** Plugin bridge JSON output does not wrap data in a `data` field (unlike status-response), so `event.data` returns undefined and resolve passes undefined to callers
- **Fix:** Changed resolve call to `event.data ?? event` for backward-compatible fallback
- **Files modified:** frontend/electron/main.cjs
- **Verification:** Both wrapped and unwrapped response formats resolve correctly
- **Committed in:** b023ded (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs, Rule 1)
**Impact on plan:** Both fixes were necessary for plugin IPC to function. Pre-existing issues from Phase 16 that blocked OAuth wiring. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full OAuth2 UI pipeline complete: Electron + TUI + CLI all wired
- Ready for Phase 20 (Gmail/Calendar validation) end-to-end testing
- Plugin auth status visible across all client surfaces

## Self-Check: PASSED

All 13 modified files confirmed present. Both task commits (b023ded, 8b4b13e) verified in git log.

---
*Phase: 18-oauth2-flow*
*Completed: 2026-03-25*
