use anyhow::Result;

use crate::cli::SkillAction;
use crate::client;
use crate::output::truncate_for_display;

pub(crate) async fn run(action: SkillAction) -> Result<()> {
    match action {
        SkillAction::List { status, limit } => {
            let variants = client::send_skill_list(status, limit).await?;
            if variants.is_empty() {
                println!("No skills found.");
            } else {
                println!(
                    "{:<12} {:<24} {:>5}  {:>9}  {}",
                    "STATUS", "SKILL NAME", "USES", "SUCCESS", "TAGS"
                );
                for variant in &variants {
                    let success = format!("{}/{}", variant.success_count, variant.use_count);
                    let tags = variant.context_tags.join(", ");
                    println!(
                        "{:<12} {:<24} {:>5}  {:>9}  {}",
                        variant.status, variant.skill_name, variant.use_count, success, tags
                    );
                }
                println!("\n{} skill(s) shown.", variants.len());
            }
        }
        SkillAction::Discover { query, limit } => {
            let result = client::send_skill_discover(&query, limit).await?;
            println!("{}", render_skill_discovery(&result));
        }
        SkillAction::Inspect { name } => {
            let (variant, content) = client::send_skill_inspect(&name).await?;
            if let Some(variant) = variant {
                println!("Skill:       {}", variant.skill_name);
                println!(
                    "Variant:     {} ({})",
                    variant.variant_name, variant.variant_id
                );
                println!("Status:      {}", variant.status);
                println!("Path:        {}", variant.relative_path);
                println!(
                    "Usage:       {} uses ({} success, {} failure)",
                    variant.use_count, variant.success_count, variant.failure_count
                );
                if !variant.context_tags.is_empty() {
                    println!("Tags:        {}", variant.context_tags.join(", "));
                }
                if let Some(content) = content {
                    println!("\n--- SKILL.md ---\n{}", content);
                }
            } else {
                eprintln!("Skill not found: {}", name);
            }
        }
        SkillAction::Reject { name } => {
            let (success, message) = client::send_skill_reject(&name).await?;
            if success {
                println!("{}", message);
            } else {
                eprintln!("{}", message);
            }
        }
        SkillAction::Promote { name, to } => {
            let (success, message) = client::send_skill_promote(&name, &to).await?;
            if success {
                println!("{}", message);
            } else {
                eprintln!("{}", message);
            }
        }
        SkillAction::Search { query } => {
            let entries = client::send_skill_search(&query).await?;
            if entries.is_empty() {
                println!("No community skills found for '{}'.", query);
            } else {
                println!(
                    "{:<10} {:<24} {:>6} {:>8} {:<10} {}",
                    "VERIFIED", "NAME", "USES", "SUCCESS", "PUBLISHER", "DESCRIPTION"
                );
                for entry in &entries {
                    let verified = if entry.publisher_verified { "✓" } else { "-" };
                    let success = format!("{:.0}%", entry.success_rate * 100.0);
                    let publisher = truncate_for_display(&entry.publisher_id, 8);
                    let description = truncate_for_display(&entry.description, 40);
                    println!(
                        "{:<10} {:<24} {:>6} {:>8} {:<10} {}",
                        verified,
                        truncate_for_display(&entry.name, 24),
                        entry.use_count,
                        success,
                        publisher,
                        description
                    );
                }
                println!("\n{} skill(s) found.", entries.len());
            }
        }
        SkillAction::Import { source, force } => {
            let (success, message, variant_id, scan_verdict, findings_count) =
                client::send_skill_import(&source, force).await?;
            if success {
                println!(
                    "Imported skill as Draft (variant: {}).",
                    variant_id.unwrap_or_default()
                );
                if scan_verdict.as_deref() == Some("warn") {
                    println!(
                        "Note: {} security warning(s) overridden with --force.",
                        findings_count
                    );
                }
            } else {
                match scan_verdict.as_deref() {
                    Some("block") => eprintln!("Import blocked: {}", message),
                    Some("warn") => eprintln!("Import requires --force: {}", message),
                    _ => eprintln!("{}", message),
                }
                std::process::exit(1);
            }
        }
        SkillAction::Export {
            name,
            format,
            output,
        } => {
            let (success, message, output_path) =
                client::send_skill_export(&name, &format, &output).await?;
            if success {
                println!("Exported to: {}", output_path.unwrap_or_default());
            } else {
                eprintln!("Export failed: {}", message);
                std::process::exit(1);
            }
        }
        SkillAction::Publish { name } => {
            let (success, message) = client::send_skill_publish(&name).await?;
            if success {
                println!("{}", message);
            } else {
                eprintln!("Publish failed: {}", message);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn render_skill_discovery(result: &amux_protocol::SkillDiscoveryResultPublic) -> String {
    let mut lines = vec![
        format!("Confidence: {}", display_or_none(&result.confidence_tier)),
        format!(
            "Next action: {}",
            display_or_none(&result.recommended_action)
        ),
    ];

    if result.candidates.is_empty() {
        lines.push("No matching skills found.".to_string());
        return lines.join("\n");
    }

    for (index, candidate) in result.candidates.iter().enumerate() {
        lines.push(format!(
            "{}. {} [{}] score={}",
            index + 1,
            candidate.skill_name,
            candidate.status,
            (candidate.score * 100.0).round() as u32
        ));
        let reasons = if candidate.reasons.is_empty() {
            "none".to_string()
        } else {
            candidate.reasons.join(", ")
        };
        lines.push(format!("   reasons: {reasons}"));
    }

    lines.join("\n")
}

fn display_or_none(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.is_empty() { "none" } else { trimmed }
}

#[cfg(test)]
mod tests {
    use super::render_skill_discovery;

    #[test]
    fn render_skill_discovery_formats_ranked_candidates() {
        let rendered = render_skill_discovery(&amux_protocol::SkillDiscoveryResultPublic {
            query: "debug panic".to_string(),
            required: true,
            confidence_tier: "strong".to_string(),
            recommended_action: "read_skill systematic-debugging".to_string(),
            explicit_rationale_required: false,
            workspace_tags: vec!["rust".to_string()],
            candidates: vec![amux_protocol::SkillDiscoveryCandidatePublic {
                variant_id: "local:systematic-debugging:v1".to_string(),
                skill_name: "systematic-debugging".to_string(),
                variant_name: "v1".to_string(),
                relative_path: "generated/systematic-debugging/SKILL.md".to_string(),
                status: "active".to_string(),
                score: 0.93,
                confidence_tier: "strong".to_string(),
                reasons: vec![
                    "matched debug".to_string(),
                    "workspace rust".to_string(),
                    "14/16 successful uses".to_string(),
                ],
                context_tags: vec!["rust".to_string()],
                use_count: 16,
                success_count: 14,
                failure_count: 2,
            }],
        });

        assert!(rendered.contains("Confidence: strong"));
        assert!(rendered.contains("Next action: read_skill systematic-debugging"));
        assert!(rendered.contains("1. systematic-debugging [active] score=93"));
        assert!(rendered.contains(
            "reasons: matched debug, workspace rust, 14/16 successful uses"
        ));
    }
}
