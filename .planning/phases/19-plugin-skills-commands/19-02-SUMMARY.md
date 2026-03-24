---
phase: 19-plugin-skills-commands
plan: 02
subsystem: plugin
tags: [plugin, cli, skills, system-prompt, agent-awareness, slash-commands]

# Dependency graph
requires:
  - phase: 19-plugin-skills-commands
    plan: 01
    provides: "Plugin skill bundling, command registry, PluginListCommands IPC, agent_loop command interception"
  - phase: 15-plugin-cli-install
    provides: "PluginAction enum, Plugin subcommand, send_plugin_* IPC helpers"
provides:
  - "CLI `tamux plugin commands` subcommand for listing registered plugin commands"
  - "send_plugin_list_commands IPC helper for PluginListCommands message"
  - "Agent system prompt plugin skills section with plugin:<name>:<endpoint> convention"
  - "list_skills tool description updated to mention plugin-bundled skills"
affects: [20-gmail-calendar-validation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Plugin skills auto-discovery: skills in ~/.tamux/skills/plugins/{name}/ appear automatically via existing collect_skill_stems"
    - "Plugin convention documentation: plugin:<name>:<endpoint> notation in system prompt for agent awareness"

key-files:
  created: []
  modified:
    - crates/amux-cli/src/client.rs
    - crates/amux-cli/src/plugins.rs
    - crates/amux-cli/src/main.rs
    - crates/amux-daemon/src/agent/system_prompt.rs
    - crates/amux-daemon/src/agent/tool_executor.rs

key-decisions:
  - "Plugin commands function takes pre-fetched data (sync) rather than calling async IPC directly, matching existing pattern separation"
  - "Plugin skills auto-discovered by existing collect_skill_stems (no skip for plugins/ prefix), no code change needed"
  - "send_plugin_list_commands returns empty vec on error/unexpected response rather than failing, for graceful degradation"

patterns-established:
  - "Plugin commands display: fixed-width table format with COMMAND/PLUGIN/DESCRIPTION columns"
  - "Plugin skills in system prompt: conditional section only rendered when plugins/ directory has subdirectories"

requirements-completed: [PSKL-03, PSKL-04, PSKL-05, PSKL-06]

# Metrics
duration: 6min
completed: 2026-03-24
---

# Phase 19 Plan 02: Plugin CLI Commands & Agent Skills Awareness Summary

**CLI `tamux plugin commands` subcommand with formatted table output, and agent system prompt plugin skills section with plugin:<name>:<endpoint> convention for PSKL-06 auto-discovery**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-24T23:40:45Z
- **Completed:** 2026-03-24T23:47:32Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- CLI `tamux plugin commands` subcommand lists registered plugin commands in formatted table
- send_plugin_list_commands IPC helper sends PluginListCommands and handles PluginCommandsResult
- Agent system prompt conditionally includes Plugin Skills subsection when plugins/ directory has content
- list_skills tool description updated to mention plugin-bundled skills alongside built-in, generated, and community skills
- Plugin skills auto-discovered through standard skill system (collect_skill_stems does not skip plugins/ prefix)

## Task Commits

Each task was committed atomically:

1. **Task 1: CLI plugin commands subcommand and IPC helper** - `aa714c0` (feat)
2. **Task 2: Agent system prompt plugin skills awareness** - `32d2fb4` (feat)

## Files Created/Modified
- `crates/amux-cli/src/client.rs` - send_plugin_list_commands IPC helper for PluginListCommands message
- `crates/amux-cli/src/plugins.rs` - plugin_commands formatted table display function
- `crates/amux-cli/src/main.rs` - Commands variant in PluginAction enum with dispatch
- `crates/amux-daemon/src/agent/system_prompt.rs` - Plugin Skills subsection with plugin:<name>:<endpoint> convention
- `crates/amux-daemon/src/agent/tool_executor.rs` - list_skills tool description mentions plugin-bundled skills

## Decisions Made
- Plugin commands function takes pre-fetched data (sync) rather than calling async IPC directly, keeping plugins.rs free of client module dependency
- send_plugin_list_commands returns empty vec on error/unexpected response for graceful degradation (CLI won't crash if daemon is unreachable)
- Plugin skills auto-discovered by existing collect_skill_stems -- verified it does not skip plugins/ prefix, no code change needed per PSKL-06

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plugin skills and commands fully wired from CLI to agent awareness
- Ready for Phase 20 Gmail/Calendar validation plugin
- All workspace compiles clean, 11 CLI tests + 692 agent tests passing

## Self-Check: PASSED

All 5 files verified present. All 2 commit hashes verified in git log.

---
*Phase: 19-plugin-skills-commands*
*Completed: 2026-03-24*
