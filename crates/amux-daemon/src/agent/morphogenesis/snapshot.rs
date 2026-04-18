use anyhow::Result;
use rusqlite::{params, OptionalExtension};

use super::affinity_tracker::{apply_decay, apply_outcome};
use super::types::{MorphogenesisAffinity, MorphogenesisOutcome};
use crate::agent::engine::AgentEngine;

const MORPHOGENESIS_DECAY_FLOOR: f64 = 0.01;

impl AgentEngine {
    pub(crate) async fn load_morphogenesis_affinities(
        &self,
        domains: &[String],
    ) -> Result<Vec<MorphogenesisAffinity>> {
        let domains = domains.to_vec();
        let now_ms = crate::history::now_ts() * 1000;

        self.history
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT agent_id, domain, affinity_score, task_count, success_count, failure_count, last_updated_ms
                     FROM morphogenesis_affinities
                     WHERE domain = ?1",
                )?;
                let mut affinities = Vec::new();
                for domain in domains {
                    let rows = stmt.query_map(params![domain], |row| {
                        Ok(MorphogenesisAffinity {
                            agent_id: row.get(0)?,
                            domain: row.get(1)?,
                            affinity_score: row.get(2)?,
                            task_count: row.get::<_, i64>(3)? as u64,
                            success_count: row.get::<_, i64>(4)? as u64,
                            failure_count: row.get::<_, i64>(5)? as u64,
                            last_updated_ms: row.get::<_, i64>(6)? as u64,
                        })
                    })?;
                    for row in rows {
                        affinities.push(apply_decay(
                            row?,
                            now_ms,
                            MORPHOGENESIS_DECAY_FLOOR,
                        ));
                    }
                }
                Ok(affinities)
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub(crate) async fn record_morphogenesis_outcome(
        &self,
        agent_id: &str,
        domains: &[String],
        outcome: MorphogenesisOutcome,
    ) -> Result<()> {
        let agent_id = agent_id.to_string();
        let domains = domains.to_vec();
        let now_ms = (crate::history::now_ts() as i64) * 1000;

        self.history
            .conn
            .call(move |conn| {
                for domain in domains {
                    let existing = conn
                        .query_row(
                            "SELECT affinity_score, task_count, success_count, failure_count, last_updated_ms
                             FROM morphogenesis_affinities
                             WHERE agent_id = ?1 AND domain = ?2",
                            params![&agent_id, &domain],
                            |row| {
                                Ok(MorphogenesisAffinity {
                                    agent_id: agent_id.clone(),
                                    domain: domain.clone(),
                                    affinity_score: row.get(0)?,
                                    task_count: row.get::<_, i64>(1)? as u64,
                                    success_count: row.get::<_, i64>(2)? as u64,
                                    failure_count: row.get::<_, i64>(3)? as u64,
                                    last_updated_ms: row.get::<_, i64>(4)? as u64,
                                })
                            },
                        )
                        .optional()?;

                    let updated =
                        apply_outcome(existing, &agent_id, &domain, outcome, now_ms as u64);
                    conn.execute(
                        "INSERT INTO morphogenesis_affinities (
                            agent_id, domain, affinity_score, task_count, success_count, failure_count, last_updated_ms
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                         ON CONFLICT(agent_id, domain) DO UPDATE SET
                            affinity_score = excluded.affinity_score,
                            task_count = excluded.task_count,
                            success_count = excluded.success_count,
                            failure_count = excluded.failure_count,
                            last_updated_ms = excluded.last_updated_ms",
                        params![
                            updated.agent_id,
                            updated.domain,
                            updated.affinity_score,
                            updated.task_count as i64,
                            updated.success_count as i64,
                            updated.failure_count as i64,
                            updated.last_updated_ms as i64,
                        ],
                    )?;
                }
                Ok(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}
