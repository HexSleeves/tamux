use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaneId {
    Threads,
    Chat,
    Composer,
    Mission,
    Telemetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneMeta {
    pub title: &'static str,
    pub focusable: bool,
}

#[derive(Debug, Default)]
pub struct PaneRegistry {
    panes: BTreeMap<PaneId, PaneMeta>,
}

impl PaneRegistry {
    pub fn bootstrap() -> Self {
        let mut registry = Self::default();
        registry.register(
            PaneId::Threads,
            PaneMeta {
                title: "Threads",
                focusable: true,
            },
        );
        registry.register(
            PaneId::Chat,
            PaneMeta {
                title: "Conversation",
                focusable: true,
            },
        );
        registry.register(
            PaneId::Composer,
            PaneMeta {
                title: "Composer",
                focusable: true,
            },
        );
        registry.register(
            PaneId::Mission,
            PaneMeta {
                title: "Mission",
                focusable: false,
            },
        );
        registry.register(
            PaneId::Telemetry,
            PaneMeta {
                title: "Telemetry",
                focusable: false,
            },
        );
        registry
    }

    pub fn register(&mut self, id: PaneId, meta: PaneMeta) {
        self.panes.insert(id, meta);
    }

    pub fn get(&self, id: PaneId) -> Option<PaneMeta> {
        self.panes.get(&id).copied()
    }
}
