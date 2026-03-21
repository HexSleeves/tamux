//! Engine runtime — stream cancellation, repo watchers, and memory cache.

use super::*;

impl AgentEngine {
    pub(super) async fn begin_stream_cancellation(&self, thread_id: &str) -> (u64, CancellationToken) {
        let generation = self.stream_generation.fetch_add(1, Ordering::Relaxed);
        let token = CancellationToken::new();
        let mut streams = self.stream_cancellations.lock().await;
        if let Some(previous) = streams.insert(
            thread_id.to_string(),
            StreamCancellationEntry {
                generation,
                token: token.clone(),
            },
        ) {
            previous.token.cancel();
        }
        (generation, token)
    }

    pub(super) async fn finish_stream_cancellation(&self, thread_id: &str, generation: u64) {
        let mut streams = self.stream_cancellations.lock().await;
        let should_remove = streams
            .get(thread_id)
            .map(|entry| entry.generation == generation)
            .unwrap_or(false);
        if should_remove {
            streams.remove(thread_id);
        }
    }

    pub async fn stop_stream(&self, thread_id: &str) -> bool {
        let token = {
            let streams = self.stream_cancellations.lock().await;
            streams.get(thread_id).map(|entry| entry.token.clone())
        };
        if let Some(token) = token {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub(super) async fn ensure_repo_watcher(&self, thread_id: &str, repo_root: &str) {
        let normalized_root = std::fs::canonicalize(repo_root)
            .unwrap_or_else(|_| std::path::PathBuf::from(repo_root))
            .to_string_lossy()
            .to_string();

        let mut watchers = self.repo_watchers.lock().await;
        if watchers
            .get(thread_id)
            .map(|entry| entry.repo_root == normalized_root)
            .unwrap_or(false)
        {
            return;
        }

        watchers.remove(thread_id);

        let refresh_tx = self.watcher_refresh_tx.clone();
        let callback_thread_id = thread_id.to_string();
        let callback_repo_root = normalized_root.clone();
        let mut watcher =
            match notify::recommended_watcher(move |result: notify::Result<Event>| match result {
                Ok(event) => {
                    if file_watch_event_is_relevant(&event) {
                        let _ = refresh_tx.send(callback_thread_id.clone());
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        thread_id = %callback_thread_id,
                        repo_root = %callback_repo_root,
                        "filesystem watcher error: {error}"
                    );
                }
            }) {
                Ok(watcher) => watcher,
                Err(error) => {
                    tracing::warn!(
                        thread_id = %thread_id,
                        repo_root = %normalized_root,
                        "failed to create filesystem watcher: {error}"
                    );
                    return;
                }
            };

        if let Err(error) = watcher.watch(
            std::path::Path::new(&normalized_root),
            RecursiveMode::Recursive,
        ) {
            tracing::warn!(
                thread_id = %thread_id,
                repo_root = %normalized_root,
                "failed to watch repo root: {error}"
            );
            return;
        }

        tracing::info!(
            thread_id = %thread_id,
            repo_root = %normalized_root,
            "filesystem watcher attached"
        );
        watchers.insert(
            thread_id.to_string(),
            ThreadRepoWatcher {
                repo_root: normalized_root,
                watcher,
            },
        );
    }

    pub(super) async fn remove_repo_watcher(&self, thread_id: &str) {
        let removed = self.repo_watchers.lock().await.remove(thread_id);
        if let Some(entry) = removed {
            tracing::info!(
                thread_id = %thread_id,
                repo_root = %entry.repo_root,
                "filesystem watcher removed"
            );
            drop(entry.watcher);
        }
    }

    pub(super) async fn refresh_memory_cache(&self) {
        let mut memory = AgentMemory::default();
        let memory_dirs = ordered_memory_dirs(&self.data_dir);
        for dir in &memory_dirs {
            if let Ok(soul) = tokio::fs::read_to_string(dir.join("SOUL.md")).await {
                memory.soul = soul;
                break;
            }
        }
        for dir in &memory_dirs {
            if let Ok(mem) = tokio::fs::read_to_string(dir.join("MEMORY.md")).await {
                memory.memory = mem;
                break;
            }
        }
        for dir in &memory_dirs {
            if let Ok(user) = tokio::fs::read_to_string(dir.join("USER.md")).await {
                memory.user_profile = user;
                break;
            }
        }
        *self.memory.write().await = memory;
    }

    pub(super) async fn onecontext_bootstrap_for_new_thread(&self, initial_message: &str) -> Option<String> {
        let trimmed = initial_message.trim();
        if trimmed.is_empty() {
            return None;
        }
        if !aline_available() {
            return None;
        }

        let query = trimmed
            .chars()
            .take(ONECONTEXT_BOOTSTRAP_QUERY_MAX_CHARS)
            .collect::<String>();

        let mut cmd = tokio::process::Command::new("aline");
        cmd.arg("search")
            .arg(&query)
            .arg("-t")
            .arg("session")
            .arg("--no-regex")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null());

        let output = match tokio::time::timeout(Duration::from_secs(4), cmd.output()).await {
            Ok(Ok(output)) if output.status.success() => output,
            _ => return None,
        };

        let raw = String::from_utf8_lossy(&output.stdout);
        let normalized = raw.trim();
        if normalized.is_empty() {
            return None;
        }

        Some(
            normalized
                .chars()
                .take(ONECONTEXT_BOOTSTRAP_OUTPUT_MAX_CHARS)
                .collect(),
        )
    }
}
