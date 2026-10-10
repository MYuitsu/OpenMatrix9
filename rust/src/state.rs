use std::collections::VecDeque;
#[derive(Debug, Clone, Copy)]
pub struct SectionState {
    pub visible: bool,
    pub collapsed: bool,
}
#[derive(Debug)]
pub struct UiState {
    pub selected_group: usize,
    pub sections: [SectionState; 7],
    pub history: VecDeque<usize>,
}
impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
impl UiState {
    pub fn new() -> Self {
        Self {
            selected_group: 0,
            sections: [SectionState {
                visible: true,
                collapsed: false,
            }; 7],
            history: VecDeque::new(),
        }
    }
    pub fn select_group(&mut self, index: usize, group_count: usize) -> bool {
        if index >= group_count {
            return false;
        }
        self.selected_group = index;
        true
    }
    pub fn set_section(&mut self, index: usize, visible: bool, collapsed: bool) -> bool {
        if let Some(s) = self.sections.get_mut(index) {
            *s = SectionState { visible, collapsed };
            true
        } else {
            false
        }
    }
    pub fn reset(&mut self) {
        self.selected_group = 0;
        self.sections.fill(SectionState {
            visible: true,
            collapsed: false,
        });
    }
    pub fn record_execution(&mut self, command: usize, success: bool) {
        if success {
            self.history.push_front(command);
            self.history.truncate(20);
        }
    }
}
