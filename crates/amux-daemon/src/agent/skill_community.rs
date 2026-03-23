//! Community skill packaging and format conversion.

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn split_frontmatter_returns_frontmatter_and_body() {
        let result = split_frontmatter("---\nname: test\n---\nbody");
        assert_eq!(result, Some(("name: test", "body")));
    }

    #[test]
    fn split_frontmatter_returns_none_without_frontmatter() {
        assert_eq!(split_frontmatter("no frontmatter"), None);
    }

    #[test]
    fn detect_skill_format_distinguishes_tamux_and_agentskills() {
        assert_eq!(
            detect_skill_format("name: demo\ntamux:\n  variant_id: abc"),
            SkillFormat::TamuxNative
        );
        assert_eq!(
            detect_skill_format("name: demo\ndescription: sample"),
            SkillFormat::AgentSkillsIo
        );
    }

    #[test]
    fn sanitize_name_for_agentskills_normalizes_variant_suffix_and_separators() {
        assert_eq!(
            sanitize_name_for_agentskills("debug_rust--async"),
            "debug-rust-async"
        );
    }

    #[test]
    fn content_hash_is_stable_sha256_hex() {
        let first = content_hash("hello world");
        let second = content_hash("hello world");

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert!(first.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn publisher_id_is_stable_truncated_sha256_hex() {
        let first = publisher_id("machine-123");
        let second = publisher_id("machine-123");

        assert_eq!(first, second);
        assert_eq!(first.len(), 16);
        assert!(first.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn to_agentskills_format_strips_tamux_extensions() {
        let skill = TamuxSkillFrontmatter {
            name: "debug_rust--async".to_string(),
            description: Some("Debug async Rust".to_string()),
            license: Some("MIT".to_string()),
            compatibility: Some(vec!["tamux>=0.1".to_string()]),
            metadata: Some(serde_yaml::from_str("category: debugging").expect("metadata yaml")),
            allowed_tools: vec!["read_file".to_string()],
            tamux: TamuxExtensions {
                maturity_status: Some("draft".to_string()),
                provenance_hash: Some("hash".to_string()),
                context_tags: vec!["rust".to_string()],
                variant_id: Some("variant-1".to_string()),
                origin_trace: Some("trace".to_string()),
                success_rate: Some(0.9),
                use_count: Some(12),
            },
        };

        let exported = to_agentskills_format(&skill, "Body");
        let (frontmatter, body) = split_frontmatter(&exported).expect("frontmatter present");

        assert_eq!(body, "Body");
        assert_eq!(detect_skill_format(frontmatter), SkillFormat::AgentSkillsIo);
        assert!(!frontmatter.contains("tamux:"));
        assert!(frontmatter.contains("allowed_tools:"));
    }

    #[test]
    fn from_agentskills_format_adds_default_tamux_extensions() {
        let imported = from_agentskills_format(
            "---\nname: debug-rust\ndescription: Debug async rust\nallowed_tools:\n  - read_file\n---\nBody",
        )
        .expect("agentskills import succeeds");

        assert_eq!(imported.name, "debug-rust");
        assert_eq!(imported.allowed_tools, vec!["read_file".to_string()]);
        assert_eq!(imported.tamux.context_tags, Vec::<String>::new());
        assert!(imported.tamux.maturity_status.is_none());
        assert!(imported.tamux.variant_id.is_none());
    }

    #[test]
    fn pack_and_unpack_skill_round_trip_tarball_contents() {
        let temp = tempfile::tempdir().expect("tempdir");
        let skill_dir = temp.path().join("skill");
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        fs::write(skill_dir.join("SKILL.md"), "---\nname: demo\n---\nBody").expect("write skill");

        let archive = temp.path().join("demo.tar.gz");
        pack_skill(&skill_dir, &archive).expect("pack succeeds");
        assert!(archive.exists());

        let unpacked = temp.path().join("unpacked");
        unpack_skill(&archive, &unpacked).expect("unpack succeeds");

        let extracted = unpacked.join("SKILL.md");
        assert_eq!(
            fs::read_to_string(extracted).expect("read extracted"),
            "---\nname: demo\n---\nBody"
        );
    }
}
