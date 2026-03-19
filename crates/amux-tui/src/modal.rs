use crate::actions::ModalKind;

#[derive(Debug, Default, Clone)]
pub struct ModalStack {
    stack: Vec<ModalKind>,
}

impl ModalStack {
    pub fn push(&mut self, modal: ModalKind) {
        self.stack.push(modal);
    }

    pub fn pop(&mut self) -> Option<ModalKind> {
        self.stack.pop()
    }

    pub fn top(&self) -> Option<ModalKind> {
        self.stack.last().copied()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}
