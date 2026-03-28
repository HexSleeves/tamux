use std::path::Path;
use std::sync::{Mutex, OnceLock};

use anyhow::Result;

use super::super::memory::MemoryTarget;
use crate::history::HistoryStore;

const USER_SYNC_STATE_CLEAN: &str = "clean";
const USER_SYNC_STATE_DIRTY: &str = "dirty";
const USER_SYNC_STATE_RECONCILING: &str = "reconciling";
const USER_PROFILE_IMPORT_SENTINEL: &str = "__legacy_user_import_done";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::agent) enum UserProfileSyncState {
    Clean,
    Dirty,
    Reconciling,
}

impl UserProfileSyncState {
    pub(in crate::agent) fn as_str(self) -> &'static str {
        match self {
            Self::Clean => USER_SYNC_STATE_CLEAN,
            Self::Dirty => USER_SYNC_STATE_DIRTY,
            Self::Reconciling => USER_SYNC_STATE_RECONCILING,
        }
    }

    pub(in crate::agent) fn from_str(value: &str) -> Self {
        match value {
            USER_SYNC_STATE_DIRTY => Self::Dirty,
            USER_SYNC_STATE_RECONCILING => Self::Reconciling,
            _ => Self::Clean,
        }
    }
}

fn sync_state_guard() -> &'static Mutex<UserProfileSyncState> {
    static STATE: OnceLock<Mutex<UserProfileSyncState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(UserProfileSyncState::Clean))
}

#[cfg(test)]
fn test_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
pub(in crate::agent) fn acquire_user_sync_test_guard() -> std::sync::MutexGuard<'static, ()> {
    match test_guard().lock() {
        Ok(guard) => guard,
        // Recover from a poisoned guard so a panicking test does not cascade-fail
        // subsequent tests that otherwise would pass.
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub(in crate::agent) fn current_user_sync_state() -> UserProfileSyncState {
    *sync_state_guard()
        .lock()
        .expect("user profile sync state mutex poisoned")
}

#[cfg(test)]
pub(in crate::agent) fn set_user_sync_state_for_test(state: UserProfileSyncState) {
    *sync_state_guard()
        .lock()
        .expect("user profile sync state mutex poisoned") = state;
}

fn set_user_sync_state(state: UserProfileSyncState) {
    *sync_state_guard()
        .lock()
        .expect("user profile sync state mutex poisoned") = state;
}

/// Atomically transition from any non-`Reconciling` state to `Reconciling`.
///
/// Returns `true` if the caller acquired the reconcile slot (and is now responsible
/// for driving the reconcile to completion), or `false` if a reconcile was already
/// in progress.  Using this instead of a separate check + set eliminates the TOCTOU
/// window where two concurrent callers could both believe they own the reconcile.
fn try_acquire_reconcile() -> bool {
    let mut guard = sync_state_guard()
        .lock()
        .expect("user profile sync state mutex poisoned");
    if *guard == UserProfileSyncState::Reconciling {
        return false;
    }
    *guard = UserProfileSyncState::Reconciling;
    true
}

fn active_memory_dir(agent_data_dir: &Path) -> std::path::PathBuf {
    super::super::active_memory_dir(agent_data_dir)
}

fn user_memory_path(agent_data_dir: &Path) -> std::path::PathBuf {
    active_memory_dir(agent_data_dir).join(MemoryTarget::User.file_name())
}

async fn bootstrap_legacy_user_import(agent_data_dir: &Path, history: &HistoryStore) -> Result<()> {
    if history
        .get_profile_field(USER_PROFILE_IMPORT_SENTINEL)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let path = user_memory_path(agent_data_dir);
    let existing = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let trimmed = existing.trim();
    if !trimmed.is_empty() {
        history
            .upsert_profile_field(
                "legacy_user_md",
                &serde_json::to_string(trimmed)?,
                0.30,
                "legacy_import",
            )
            .await?;
        history
            .append_profile_event(
                &format!("op_evt_{}", uuid::Uuid::new_v4()),
                "legacy_user_import",
                Some("legacy_user_md"),
                Some(&serde_json::to_string(trimmed)?),
                "legacy_import",
                None,
            )
            .await?;
    }

    history
        .upsert_profile_field(
            USER_PROFILE_IMPORT_SENTINEL,
            "\"true\"",
            1.0,
            "legacy_import",
        )
        .await?;
    Ok(())
}

fn render_user_profile_markdown(fields: &[crate::history::OperatorProfileFieldRow]) -> String {
    let mut lines = vec![
        "# User".to_string(),
        "Profile summary is generated from SQLite-backed operator profile.".to_string(),
        "".to_string(),
    ];

    let mut ordered = fields
        .iter()
        .filter(|row| row.field_key != USER_PROFILE_IMPORT_SENTINEL)
        .cloned()
        .collect::<Vec<_>>();
    ordered.sort_by(|a, b| a.field_key.cmp(&b.field_key));
    for row in ordered {
        lines.push(format!("- {}: {}", row.field_key, row.field_value_json));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub(in crate::agent) async fn reconcile_user_profile_from_db(
    agent_data_dir: &Path,
    history: &HistoryStore,
) -> Result<()> {
    if !try_acquire_reconcile() {
        // A reconcile is already in progress; return no-op success so the caller
        // does not need to distinguish "skipped" from "done".  The in-flight
        // reconcile will complete and leave state as Clean (or Dirty on error).
        return Ok(());
    }
    reconcile_inner(agent_data_dir, history).await
}

/// Drive the reconcile body.  **Caller must have already set state to `Reconciling`**
/// (either via `set_user_sync_state` or `try_acquire_reconcile`).
/// All error paths reset state to `Dirty` so the slot is never left stuck.
async fn reconcile_inner(agent_data_dir: &Path, history: &HistoryStore) -> Result<()> {
    if let Err(error) = bootstrap_legacy_user_import(agent_data_dir, history).await {
        set_user_sync_state(UserProfileSyncState::Dirty);
        return Err(error);
    }

    let rows = match history.list_profile_fields().await {
        Ok(r) => r,
        Err(error) => {
            set_user_sync_state(UserProfileSyncState::Dirty);
            return Err(error);
        }
    };

    let content = render_user_profile_markdown(&rows);
    let path = user_memory_path(agent_data_dir);
    match tokio::fs::write(&path, content).await {
        Ok(()) => {
            set_user_sync_state(UserProfileSyncState::Clean);
            Ok(())
        }
        Err(error) => {
            set_user_sync_state(UserProfileSyncState::Dirty);
            Err(error.into())
        }
    }
}

pub(in crate::agent) async fn stage_legacy_user_memory_write(
    history: &HistoryStore,
    content: &str,
) -> Result<()> {
    stage_legacy_user_memory_write_events(history, content).await?;
    set_user_sync_state(UserProfileSyncState::Dirty);
    Ok(())
}

/// Write the legacy-append events to the DB **without** touching sync state.
/// Used by `handle_user_memory_append_with_reconcile` so that the caller can
/// manage the state transition atomically around the reconcile slot acquisition.
async fn stage_legacy_user_memory_write_events(
    history: &HistoryStore,
    content: &str,
) -> Result<()> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let value_json = serde_json::to_string(trimmed)?;
    history
        .upsert_profile_field("legacy_user_signal", &value_json, 0.55, "legacy_append")
        .await?;
    history
        .append_profile_event(
            &format!("op_evt_{}", uuid::Uuid::new_v4()),
            "legacy_user_memory_append",
            Some("legacy_user_signal"),
            Some(&value_json),
            "legacy_append",
            None,
        )
        .await?;
    Ok(())
}

pub(in crate::agent) async fn handle_user_memory_append_with_reconcile(
    agent_data_dir: &Path,
    history: &HistoryStore,
    content: &str,
) -> Result<()> {
    // Atomically claim the reconcile slot BEFORE any staging work so the state
    // never escapes the Reconciling→Dirty→re-acquire TOCTOU window.
    let acquired = try_acquire_reconcile();

    // Stage the event to DB only (no state change here); the state transition is
    // owned by the reconcile-slot logic above and by reconcile_inner below.
    // If staging fails and we own the reconcile slot, release it to Dirty so the
    // state machine is never left stuck in Reconciling.
    if let Err(error) = stage_legacy_user_memory_write_events(history, content).await {
        if acquired {
            set_user_sync_state(UserProfileSyncState::Dirty);
        }
        return Err(error);
    }

    if !acquired {
        // A reconcile is already in progress.  The staged event will be picked up
        // on the next reconcile cycle.  Leave state as Reconciling (owned by the
        // in-flight reconcile).
        return Ok(());
    }

    // We own the Reconciling state; drive the reconcile directly.
    reconcile_inner(agent_data_dir, history).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stage_legacy_append_marks_dirty() -> Result<()> {
        let _guard = acquire_user_sync_test_guard();
        let root =
            std::env::temp_dir().join(format!("tamux-user-sync-test-{}", uuid::Uuid::new_v4()));
        let history = crate::history::HistoryStore::new_test_store(&root).await?;
        set_user_sync_state_for_test(UserProfileSyncState::Clean);

        stage_legacy_user_memory_write(&history, "- prefers concise output").await?;

        assert_eq!(current_user_sync_state(), UserProfileSyncState::Dirty);
        let field = history
            .get_profile_field("legacy_user_signal")
            .await?
            .expect("legacy signal field should exist");
        assert_eq!(field.source, "legacy_append");
        Ok(())
    }

    #[tokio::test]
    async fn reconcile_renders_deterministic_user_md() -> Result<()> {
        let _guard = acquire_user_sync_test_guard();
        let root =
            std::env::temp_dir().join(format!("tamux-user-sync-test-{}", uuid::Uuid::new_v4()));
        let history = crate::history::HistoryStore::new_test_store(&root).await?;
        let memory_dir = super::active_memory_dir(&root);
        tokio::fs::create_dir_all(&memory_dir).await?;
        tokio::fs::write(memory_dir.join("USER.md"), "# User\nlegacy").await?;
        history
            .upsert_profile_field("preferred_name", "\"Milan\"", 1.0, "onboarding")
            .await?;
        set_user_sync_state_for_test(UserProfileSyncState::Dirty);

        reconcile_user_profile_from_db(&root, &history).await?;
        let rendered = tokio::fs::read_to_string(memory_dir.join("USER.md")).await?;
        assert!(rendered.contains("- preferred_name: \"Milan\""));
        assert_eq!(current_user_sync_state(), UserProfileSyncState::Clean);
        Ok(())
    }

    #[tokio::test]
    async fn reconcile_sets_dirty_on_write_error() -> Result<()> {
        let _guard = acquire_user_sync_test_guard();
        let root =
            std::env::temp_dir().join(format!("tamux-user-sync-test-{}", uuid::Uuid::new_v4()));
        let history = crate::history::HistoryStore::new_test_store(&root).await?;
        let memory_dir = super::active_memory_dir(&root);
        tokio::fs::create_dir_all(&memory_dir).await?;
        // Make USER.md a directory so tokio::fs::write to it fails with EISDIR,
        // exercising the Dirty-on-error reset in reconcile_inner.
        tokio::fs::create_dir_all(memory_dir.join("USER.md")).await?;
        set_user_sync_state_for_test(UserProfileSyncState::Clean);

        let result = reconcile_user_profile_from_db(&root, &history).await;
        assert!(
            result.is_err(),
            "expected error because USER.md is a directory"
        );
        assert_eq!(
            current_user_sync_state(),
            UserProfileSyncState::Dirty,
            "state must be Dirty after reconcile error, not stuck in Reconciling"
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_user_appends_both_stage_and_no_stuck_reconciling() -> Result<()> {
        let _guard = acquire_user_sync_test_guard();
        let root =
            std::env::temp_dir().join(format!("tamux-user-sync-test-{}", uuid::Uuid::new_v4()));
        let history = crate::history::HistoryStore::new_test_store(&root).await?;
        let memory_dir = super::active_memory_dir(&root);
        tokio::fs::create_dir_all(&memory_dir).await?;
        tokio::fs::write(memory_dir.join("USER.md"), "").await?;
        set_user_sync_state_for_test(UserProfileSyncState::Clean);

        let h1 = history.clone();
        let h2 = history.clone();
        let r1 = root.clone();
        let r2 = root.clone();

        let (res1, res2) = tokio::join!(
            handle_user_memory_append_with_reconcile(&r1, &h1, "prefers dark mode"),
            handle_user_memory_append_with_reconcile(&r2, &h2, "uses vim keybindings"),
        );
        res1?;
        res2?;

        // Both events must be durably staged in the DB.
        let events = history.list_profile_events(20).await?;
        let append_count = events
            .iter()
            .filter(|e| e.event_type == "legacy_user_memory_append")
            .count();
        assert!(
            append_count >= 2,
            "both concurrent appends should be staged; got {append_count}"
        );

        // State must not be stuck in Reconciling after both operations complete.
        assert_ne!(
            current_user_sync_state(),
            UserProfileSyncState::Reconciling,
            "state must not be stuck in Reconciling after concurrent appends"
        );
        Ok(())
    }

    /// Fix 1: Staging failure after acquiring the reconcile slot must transition
    /// state to Dirty, not leave it stuck in Reconciling.
    #[tokio::test]
    async fn staging_failure_after_acquire_resets_state_to_dirty() -> Result<()> {
        let _guard = acquire_user_sync_test_guard();
        let root =
            std::env::temp_dir().join(format!("tamux-user-sync-test-{}", uuid::Uuid::new_v4()));
        let history = crate::history::HistoryStore::new_test_store(&root).await?;
        let memory_dir = super::active_memory_dir(&root);
        tokio::fs::create_dir_all(&memory_dir).await?;
        tokio::fs::write(memory_dir.join("USER.md"), "").await?;
        set_user_sync_state_for_test(UserProfileSyncState::Clean);

        // Drop the operator_profile_fields table so that stage_legacy_user_memory_write_events
        // fails with "no such table", simulating a mid-flight DB error after the reconcile
        // slot has been acquired.
        history
            .conn
            .call(|conn| {
                conn.execute_batch("DROP TABLE IF EXISTS operator_profile_fields")?;
                Ok(())
            })
            .await?;

        let result = handle_user_memory_append_with_reconcile(&root, &history, "should fail").await;
        assert!(
            result.is_err(),
            "expected staging to fail with no such table"
        );
        assert_eq!(
            current_user_sync_state(),
            UserProfileSyncState::Dirty,
            "state must be Dirty after staging failure, not stuck in Reconciling"
        );
        Ok(())
    }

    /// Fix 2: `reconcile_user_profile_from_db` must be a no-op when a reconcile is
    /// already in progress (atomic acquisition semantics prevent duplicate runs).
    #[tokio::test]
    async fn direct_reconcile_is_noop_when_already_reconciling() -> Result<()> {
        let _guard = acquire_user_sync_test_guard();
        let root =
            std::env::temp_dir().join(format!("tamux-user-sync-test-{}", uuid::Uuid::new_v4()));
        let history = crate::history::HistoryStore::new_test_store(&root).await?;
        let memory_dir = super::active_memory_dir(&root);
        tokio::fs::create_dir_all(&memory_dir).await?;
        // Put a sentinel in USER.md; a real reconcile would overwrite it.
        tokio::fs::write(memory_dir.join("USER.md"), "SENTINEL").await?;

        // Simulate an already-in-flight reconcile by setting state to Reconciling.
        set_user_sync_state_for_test(UserProfileSyncState::Reconciling);

        // reconcile_user_profile_from_db must return Ok(()) without running and
        // without touching the file.
        reconcile_user_profile_from_db(&root, &history).await?;

        let contents = tokio::fs::read_to_string(memory_dir.join("USER.md")).await?;
        assert_eq!(
            contents, "SENTINEL",
            "USER.md must be untouched when reconcile was skipped"
        );
        // State must remain Reconciling (owned by the simulated in-flight reconcile).
        assert_eq!(
            current_user_sync_state(),
            UserProfileSyncState::Reconciling,
            "state must remain Reconciling when direct reconcile was a no-op"
        );
        Ok(())
    }

    #[tokio::test]
    async fn append_reconcile_write_error_keeps_db_updates_and_marks_dirty() -> Result<()> {
        let _guard = acquire_user_sync_test_guard();
        let root =
            std::env::temp_dir().join(format!("tamux-user-sync-test-{}", uuid::Uuid::new_v4()));
        let history = crate::history::HistoryStore::new_test_store(&root).await?;
        let memory_dir = super::active_memory_dir(&root);
        tokio::fs::create_dir_all(&memory_dir).await?;
        // Make USER.md a directory so reconcile write fails after DB staging succeeds.
        tokio::fs::create_dir_all(memory_dir.join("USER.md")).await?;

        set_user_sync_state_for_test(UserProfileSyncState::Clean);
        let result = handle_user_memory_append_with_reconcile(&root, &history, "uses neovim").await;
        assert!(result.is_err(), "expected reconcile to fail when USER.md is a directory");
        assert_eq!(
            current_user_sync_state(),
            UserProfileSyncState::Dirty,
            "sync state must be Dirty when USER.md sync fails"
        );

        let staged_field = history
            .get_profile_field("legacy_user_signal")
            .await?
            .expect("legacy_user_signal should be persisted even when USER.md write fails");
        assert_eq!(staged_field.source, "legacy_append");

        let events = history.list_profile_events(20).await?;
        assert!(
            events
                .iter()
                .any(|event| event.event_type == "legacy_user_memory_append"),
            "legacy append event should be present even when USER.md write fails"
        );
        Ok(())
    }
}
