use crate::agent::context::structural_memory::{MemoryGraphEdgeUpsert, MemoryGraphUpdateBatch};

use super::types::MemoryPalaceEdge;

pub(crate) fn prune_memory_update_batch(
    batch: MemoryGraphUpdateBatch,
    min_weight: f64,
) -> (MemoryGraphUpdateBatch, Vec<MemoryPalaceEdge>) {
    let mut pruned_edges = Vec::new();
    let kept_edges = batch
        .edges
        .into_iter()
        .filter(|edge| {
            let keep = edge.weight >= min_weight;
            if !keep {
                pruned_edges.push(MemoryPalaceEdge {
                    source_node_id: edge.source_node_id.clone(),
                    target_node_id: edge.target_node_id.clone(),
                    relation: edge.relation_type.clone(),
                    weight: edge.weight,
                });
            }
            keep
        })
        .collect::<Vec<MemoryGraphEdgeUpsert>>();

    (
        MemoryGraphUpdateBatch {
            nodes: batch.nodes,
            edges: kept_edges,
        },
        pruned_edges,
    )
}
