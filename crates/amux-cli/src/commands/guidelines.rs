use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::cli::GuidelineAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GuidelineEntry {
    name: String,
    relative_path: String,
    description: Option<String>,
}

pub(crate) fn run(action: GuidelineAction) -> Result<()> {
    let root = amux_protocol::tamux_guidelines_dir();
    match action {
        GuidelineAction::Install {
            source,
            name,
            force,
        } => {
            let installed = install_guideline_command(&source, name.as_deref(), force)?;
            println!("Installed guideline: {}", installed.display());
            println!("Guidelines root: {}", root.display());
        }
        GuidelineAction::List { json } => {
            let entries = list_guideline_files(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else if entries.is_empty() {
                println!("No guidelines found under {}.", root.display());
            } else {
                println!("Guidelines under {}:", root.display());
                for entry in entries {
                    match entry.description.as_deref() {
                        Some(description) => {
                            println!(
                                "- {} ({}) - {}",
                                entry.name, entry.relative_path, description
                            )
                        }
                        None => println!("- {} ({})", entry.name, entry.relative_path),
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn install_guideline_command(
    source: &str,
    name: Option<&str>,
    force: bool,
) -> Result<PathBuf> {
    install_guideline_file(
        Path::new(source),
        &amux_protocol::tamux_guidelines_dir(),
        name,
        force,
    )
}

fn install_guideline_file(
    source: &Path,
    guidelines_root: &Path,
    name: Option<&str>,
    force: bool,
) -> Result<PathBuf> {
    let source = std::fs::canonicalize(source)
        .with_context(|| format!("guideline source was not found: {}", source.display()))?;
    if !source.is_file() {
        anyhow::bail!("guideline source must be a file: {}", source.display());
    }
    if !source
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("md"))
    {
        anyhow::bail!("guideline source must be a markdown .md file");
    }

    let filename = match name {
        Some(value) => validate_destination_name(value)?,
        None => source
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow::anyhow!("guideline source has no filename"))?,
    };
    if !filename.to_ascii_lowercase().ends_with(".md") {
        anyhow::bail!("guideline destination name must end with .md");
    }

    std::fs::create_dir_all(guidelines_root).with_context(|| {
        format!(
            "failed to create guidelines directory {}",
            guidelines_root.display()
        )
    })?;
    let destination = guidelines_root.join(filename);
    if destination.exists() && !force {
        anyhow::bail!(
            "guideline already exists: {} (use --force to overwrite)",
            destination.display()
        );
    }
    std::fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy guideline {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(destination)
}

fn validate_destination_name(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("guideline destination name must not be empty");
    }
    let path = Path::new(trimmed);
    if path.is_absolute() || path.components().count() != 1 {
        anyhow::bail!("guideline destination name must be a filename, not a path");
    }
    Ok(trimmed.to_string())
}

fn list_guideline_files(guidelines_root: &Path) -> Result<Vec<GuidelineEntry>> {
    let mut files = Vec::new();
    collect_markdown_files(guidelines_root, &mut files)?;
    files.sort();

    let mut entries = Vec::new();
    for path in files {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read guideline {}", path.display()))?;
        let relative_path = path
            .strip_prefix(guidelines_root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let name = frontmatter_value(&content, "name").unwrap_or_else(|| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("guideline")
                .to_string()
        });
        let description = frontmatter_value(&content, "description");
        entries.push(GuidelineEntry {
            name,
            relative_path,
            description,
        });
    }
    Ok(entries)
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_markdown_files(&path, out)?;
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("md"))
        {
            out.push(path);
        }
    }
    Ok(())
}

fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    let rest = content.strip_prefix("---\n")?;
    let frontmatter = rest.split_once("\n---\n")?.0;
    for line in frontmatter.lines() {
        let Some((line_key, line_value)) = line.split_once(':') else {
            continue;
        };
        if line_key.trim() == key {
            let value = line_value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_guideline_copies_markdown_without_overwriting() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("coding-task.md");
        let root = temp.path().join("guidelines");
        std::fs::write(&source, "# Coding Task\n").expect("write source");

        let installed = install_guideline_file(&source, &root, None, false).expect("install");
        assert_eq!(installed, root.join("coding-task.md"));
        assert_eq!(
            std::fs::read_to_string(&installed).expect("read installed"),
            "# Coding Task\n"
        );

        let error =
            install_guideline_file(&source, &root, None, false).expect_err("overwrite blocked");
        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn list_guidelines_reads_from_guidelines_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path().join("guidelines");
        std::fs::create_dir_all(&root).expect("create root");
        std::fs::write(
            root.join("coding-task.md"),
            "---\nname: coding-task\ndescription: Coding work\n---\n# Coding Task\n",
        )
        .expect("write guideline");

        let entries = list_guideline_files(&root).expect("list guidelines");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "coding-task");
        assert_eq!(entries[0].relative_path, "coding-task.md");
        assert_eq!(entries[0].description.as_deref(), Some("Coding work"));
    }
}
