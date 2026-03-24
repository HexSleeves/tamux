//! Skill bundling for plugins: copies YAML skill files on install, removes on uninstall.

use anyhow::Result;
use std::path::Path;

use super::manifest::PluginManifest;

/// Copy bundled skill files from plugin dir to the standard skills directory.
/// Target: `skills_root/plugins/{plugin_name}/`
/// Returns count of successfully copied files.
pub(crate) fn install_bundled_skills(
    _plugins_dir: &Path,
    _plugin_name: &str,
    _manifest: &PluginManifest,
    _skills_root: &Path,
) -> Result<usize> {
    todo!()
}

/// Remove skill directory for a plugin.
/// Target: `skills_root/plugins/{plugin_name}/`
pub(crate) fn remove_bundled_skills(_plugin_name: &str, _skills_root: &Path) -> Result<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_manifest(name: &str, skills: Option<Vec<String>>) -> PluginManifest {
        PluginManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            schema_version: 1,
            description: None,
            author: None,
            license: None,
            tamux_version: None,
            settings: None,
            api: None,
            commands: None,
            skills,
            auth: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn install_bundled_skills_copies_yaml_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        let skills_root = tmp.path().join("skills");
        std::fs::create_dir_all(&skills_root).unwrap();

        // Create plugin dir with skill files
        let plugin_dir = plugins_dir.join("my-plugin");
        std::fs::create_dir_all(plugin_dir.join("skills")).unwrap();
        std::fs::write(plugin_dir.join("skills/search.yaml"), "name: search").unwrap();
        std::fs::write(plugin_dir.join("skills/send.yaml"), "name: send").unwrap();

        let manifest = make_manifest(
            "my-plugin",
            Some(vec![
                "skills/search.yaml".to_string(),
                "skills/send.yaml".to_string(),
            ]),
        );

        let count = install_bundled_skills(&plugins_dir, "my-plugin", &manifest, &skills_root).unwrap();
        assert_eq!(count, 2);

        let target_dir = skills_root.join("plugins").join("my-plugin");
        assert!(target_dir.join("search.yaml").exists());
        assert!(target_dir.join("send.yaml").exists());
    }

    #[test]
    fn install_bundled_skills_noop_when_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        let skills_root = tmp.path().join("skills");

        let manifest = make_manifest("my-plugin", None);
        let count = install_bundled_skills(&plugins_dir, "my-plugin", &manifest, &skills_root).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn install_bundled_skills_noop_when_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        let skills_root = tmp.path().join("skills");

        let manifest = make_manifest("my-plugin", Some(vec![]));
        let count = install_bundled_skills(&plugins_dir, "my-plugin", &manifest, &skills_root).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn remove_bundled_skills_removes_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let skills_root = tmp.path().join("skills");
        let target_dir = skills_root.join("plugins").join("my-plugin");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("search.yaml"), "name: search").unwrap();

        remove_bundled_skills("my-plugin", &skills_root).unwrap();
        assert!(!target_dir.exists());
    }

    #[test]
    fn remove_bundled_skills_noop_when_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let skills_root = tmp.path().join("skills");
        // Dir does not exist -- should not error
        remove_bundled_skills("nonexistent-plugin", &skills_root).unwrap();
    }
}
