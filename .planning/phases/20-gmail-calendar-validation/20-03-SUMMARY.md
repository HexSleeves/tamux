---
plan: "20-03"
phase: "20-gmail-calendar-validation"
status: complete
started: "2026-03-25T10:00:00Z"
completed: "2026-03-25T12:30:00Z"
---

# Plan 20-03: End-to-End Validation — Summary

## What Was Built
End-to-end validation of the Gmail/Calendar plugin package through human testing, with numerous bug fixes discovered and applied during validation.

## Outcome
Phase 20 validated successfully with fixes. The full plugin loop works: install → configure → OAuth → API call → real Google data returned. Calendar events retrieved, email inbox listed, events created.

## Validation Results

| Step | Status | Notes |
|------|--------|-------|
| Plugin install (nested detect) | PASS | Both gmail and calendar registered from single package |
| Plugin list | PASS | Both appear in CLI and TUI |
| Plugin commands | PASS | All commands visible in TUI palette and sendable |
| Settings UI | PASS | Client ID, Client Secret, Calendar ID fields render |
| Settings persistence | PASS (after fix) | INSERT OR REPLACE was cascading deletes — fixed to ON CONFLICT |
| OAuth flow | PASS | Google OAuth2 completes, tokens stored |
| Calendar read | PASS | Agent answers "what's on my calendar?" with real events |
| Gmail read | PASS | Agent lists inbox with real email data |
| Calendar create | PASS (after fix) | Body template escaping fixed |
| Slash commands | PASS (after fix) | Routed to agent as chat messages |

## Bugs Found and Fixed During Validation

| Bug | Root Cause | Fix Commit |
|-----|-----------|------------|
| Missing client_id/client_secret fields | Manifests didn't declare OAuth credential settings | 498a458 |
| TUI settings don't save | Base modal handler ate Enter for plugin fields | d8baec9 |
| TUI corrupted by OAuth browser | xdg-open stderr bleeds into alternate screen | 9d1f4f8 |
| TUI blocked by concierge LLM call | Single-connection handler blocked by RequestConciergeWelcome | 3f7db69 |
| Skills invisible to agent | Skills were .yaml not .md (agentskills.io format) | d50d582 |
| No Authorization header on API calls | Manifests missing Bearer token header | a2bfc11 |
| Keys erased on daemon restart | INSERT OR REPLACE triggered ON DELETE CASCADE | 93557fe |
| Secret fields save masked value | Edit buffer showed ******** for secrets | 93557fe |
| Commands invisible in TUI palette | Palette was hardcoded, no plugin commands | 92b42af |
| Slash commands do nothing | execute_command catch-all showed "Unknown command" | b80c871 |
| 411 on POST without body | Google requires Content-Length for bodyless POST | 2a42737 |
| Body template parsing errors | JSON escaping breaks Handlebars expressions with quotes | 95d67d4 |
| Palette commands send immediately | Should insert into input for user to add context | 02f103f |

## Key Files
- `plugins/tamux-plugin-gmail-calendar/` — Full plugin package (2 manifests, 2 skills, README, package.json)
- `crates/amux-daemon/src/plugin/persistence.rs` — Critical CASCADE fix
- `crates/amux-tui/src/app/modal_handlers.rs` — Plugin settings routing fix
- `crates/amux-tui/src/app/events.rs` — Concierge blocking fix, plugin commands fetch
- `crates/amux-tui/src/app/commands.rs` — Command palette dispatch

## Self-Check: PASSED
