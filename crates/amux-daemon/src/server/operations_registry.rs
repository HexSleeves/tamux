use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct OperationRegistry {
    records: Mutex<HashMap<String, OperationRecord>>,
    dedup_index: Mutex<HashMap<String, String>>,
}

impl OperationRegistry {
    pub(super) fn accept_operation(
        &self,
        kind: &str,
        dedup: Option<String>,
    ) -> OperationRecord {
        if let Some(existing) = dedup.as_ref().and_then(|dedup_key| {
            let dedup_index = self.dedup_index.lock().expect("operation dedup mutex poisoned");
            let operation_id = dedup_index.get(dedup_key)?.clone();
            drop(dedup_index);
            self.snapshot(&operation_id).and_then(|snapshot| {
                if matches!(
                    snapshot.state,
                    amux_protocol::OperationLifecycleState::Accepted
                        | amux_protocol::OperationLifecycleState::Started
                ) {
                    self.record(&operation_id)
                } else {
                    None
                }
            })
        }) {
            return existing;
        }

        let record = OperationRecord {
            operation_id: uuid::Uuid::new_v4().to_string(),
            kind: kind.to_string(),
            dedup: dedup.clone(),
            state: amux_protocol::OperationLifecycleState::Accepted,
            revision: 0,
        };

        {
            let mut records = self.records.lock().expect("operation records mutex poisoned");
            records.insert(record.operation_id.clone(), record.clone());
        }

        if let Some(dedup_key) = dedup {
            let mut dedup_index = self
                .dedup_index
                .lock()
                .expect("operation dedup mutex poisoned");
            dedup_index.insert(dedup_key, record.operation_id.clone());
        }

        record
    }

    pub(super) fn mark_started(&self, operation_id: &str) {
        self.update_state(operation_id, amux_protocol::OperationLifecycleState::Started);
    }

    pub(super) fn mark_completed(&self, operation_id: &str) {
        self.update_state(operation_id, amux_protocol::OperationLifecycleState::Completed);
    }

    pub(super) fn mark_failed(&self, operation_id: &str) {
        self.update_state(operation_id, amux_protocol::OperationLifecycleState::Failed);
    }

    pub(super) fn snapshot(&self, operation_id: &str) -> Option<amux_protocol::OperationStatusSnapshot> {
        self.record(operation_id).map(|record| record.snapshot())
    }

    fn record(&self, operation_id: &str) -> Option<OperationRecord> {
        let records = self.records.lock().expect("operation records mutex poisoned");
        records.get(operation_id).cloned()
    }

    fn update_state(&self, operation_id: &str, state: amux_protocol::OperationLifecycleState) {
        let mut records = self.records.lock().expect("operation records mutex poisoned");
        if let Some(record) = records.get_mut(operation_id) {
            if record.state != state {
                record.state = state;
                record.revision = record.revision.saturating_add(1);
            }
        }
    }
}
