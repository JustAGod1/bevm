use crate::ui::window::Tool;
use imgui::{Ui, ImString};
use crate::ui::gui::GuiState;
use imgui::StyleColor::Header;

pub struct HelpTool {
    text: &'static str
}

impl HelpTool {
    pub fn new(text: &'static str) -> HelpTool {
        HelpTool {
            text
        }
    }
}

impl Tool for HelpTool {
    fn draw(&mut self, ui: &Ui, _: &mut GuiState) {
        ui.text_wrapped(ImString::new(self.text).as_ref());
    }
}
