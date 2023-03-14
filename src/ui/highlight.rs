use crate::ui::gui::GuiState;
use crate::ui::window::Tool;
use imgui::{Io, Ui};

pub struct CommandHighlightTool;

impl CommandHighlightTool {
    pub fn new() -> CommandHighlightTool {
        CommandHighlightTool {}
    }
}

impl Tool for CommandHighlightTool {
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        if state.current_command.is_none() {
            return;
        }

        let command = state.current_command.as_ref().unwrap();

        command.draw_highlight(ui)
    }
}
