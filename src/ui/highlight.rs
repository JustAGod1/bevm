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
    fn draw(&mut self, ui: &Ui, _io: &Io, state: &mut GuiState) {
        let Some(command) = state.current_command.as_ref() else {
            return;
        };

        command.draw_highlight(ui)
    }
}
