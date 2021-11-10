use crate::ui::window::Tool;
use imgui::{Ui, ImString, Io, im_str, MenuItem};
use crate::ui::gui::GuiState;
use imgui::StyleColor::Header;
use crate::ui::open_in_app;
use crate::ui::popup::PopupMessage;

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
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        ui.menu_bar(|| {
           ui.menu(im_str!("Полезные ссылочки"), true, || {
               if MenuItem::new(im_str!("GitHub")).build(ui) {
                   if let Err(e) = open_in_app("https://github.com/JustAGod1/bevm") {
                       state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть ссылку: {}", e)))
                   }
               }
               if MenuItem::new(im_str!("Telegram")).build(ui) {
                   if let Err(e) = open_in_app("https://t.me/notsofunnyhere") {
                       state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть ссылку: {}", e)))
                   }
               }
           })
        });
        ui.text_wrapped(ImString::new(self.text).as_ref());
    }
}
