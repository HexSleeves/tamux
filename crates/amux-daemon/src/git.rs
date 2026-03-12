use amux_protocol::GitInfo;
use std::process::Command;

/// Get git status for a working directory.
/// Uses `git` CLI to avoid a heavy libgit2 dependency.
pub fn get_git_status(path: &str) -> GitInfo {
    let default = GitInfo {
        branch: None,
        is_dirty: false,
        ahead: 0,
        behind: 0,
        untracked: 0,
        modified: 0,
        staged: 0,
    };

    // Check if it's a git repo
    let branch = match Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
    {
        Ok(output) if output.status.success() => {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if s.is_empty() {
                return default;
            }
            s
        }
        _ => return default,
    };

    // Get porcelain status
    let status_output = Command::new("git")
        .args(["status", "--porcelain=v1", "--branch"])
        .current_dir(path)
        .output();

    let (mut untracked, mut modified, mut staged, mut ahead, mut behind) =
        (0u32, 0u32, 0u32, 0u32, 0u32);

    if let Ok(output) = status_output {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("## ") {
                // Parse ahead/behind from branch line: ## main...origin/main [ahead 1, behind 2]
                if let Some(idx) = line.find("[ahead ") {
                    let rest = &line[idx + 7..];
                    if let Some(end) = rest.find(']').or_else(|| rest.find(',')) {
                        ahead = rest[..end].trim().parse().unwrap_or(0);
                    }
                }
                if let Some(idx) = line.find("behind ") {
                    let rest = &line[idx + 7..];
                    if let Some(end) = rest.find(']') {
                        behind = rest[..end].trim().parse().unwrap_or(0);
                    }
                }
            } else if line.starts_with("??") {
                untracked += 1;
            } else if line.len() >= 2 {
                let bytes = line.as_bytes();
                // First char = index status, second = work tree status
                if bytes[0] != b' ' && bytes[0] != b'?' {
                    staged += 1;
                }
                if bytes[1] != b' ' && bytes[1] != b'?' {
                    modified += 1;
                }
            }
        }
    }

    let is_dirty = untracked > 0 || modified > 0 || staged > 0;

    GitInfo {
        branch: Some(branch),
        is_dirty,
        ahead,
        behind,
        untracked,
        modified,
        staged,
    }
}
