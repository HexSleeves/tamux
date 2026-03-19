#[derive(Debug, Clone)]
pub enum AppAction {
    Tick,
    Resize { width: u16, height: u16 },
    Focus(FocusTarget),
    OpenModal(ModalKind),
    CloseTopModal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Threads,
    Chat,
    Mission,
    Composer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind {
    CommandPalette,
    ThreadPicker,
    SessionPicker,
    ProviderPicker,
    ModelPicker,
    PluginPicker,
    McpPicker,
    SkillsPicker,
    ApprovalOverlay,
}
