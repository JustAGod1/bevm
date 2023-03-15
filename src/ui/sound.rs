use imgui::{im_str, ImString};

use crate::ui::window::Tool;

pub struct SoundPanel {
    button_size: f32,
}
// Play for 2 seconds
impl SoundPanel {
    pub fn new() -> Self {
        SoundPanel { button_size: 150. }
    }
}


impl Tool for SoundPanel {
    fn draw(&mut self, ui: &imgui::Ui, io: &imgui::Io, state: &mut super::gui::GuiState) {
        
    }
}
