use crate::ui::gui::GuiState;
use crate::ui::gui::Theme::{Classic, Dark, Light};
use crate::ui::open_in_app;
use crate::ui::popup::PopupMessage;
use crate::ui::window::Tool;

use imgui::{Io, Ui};

pub struct HelpTool {
    text: &'static str,
}

impl HelpTool {
    pub fn new(text: &'static str) -> HelpTool {
        HelpTool { text }
    }
}

impl Tool for HelpTool {
    fn draw(&mut self, ui: &Ui, _io: &Io, state: &mut GuiState) {
        ui.menu_bar(|| {
           ui.menu("Полезные ссылочки", || {
               if ui.menu_item("GitHub") {
                   if let Err(e) = open_in_app("https://github.com/JustAGod1/bevm") {
                       state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть ссылку: {}", e)))
                   }
               }
               if ui.menu_item("Telegram") {
                   if let Err(e) = open_in_app("https://t.me/notsofunnyhere") {
                       state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть ссылку: {}", e)))
                   }
               }
               if ui.menu_item("Методичка") {
                   if let Err(e) = open_in_app("https://yadi.sk/i/brIICpYtcb3LMg") {
                       state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть ссылку: {}", e)))
                   }
               }
               if ui.menu_item("Моя телега") {
                   if let Err(e) = open_in_app("https://t.me/JustAG0d") {
                       state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть ссылку: {}", e)))
                   }
               } else if ui.is_item_hovered() {
                   ui.tooltip_text("Мне желательно писать по поводу идей для новых фич для этой БЭВМ.\n\n\
                    Желательно придерживаться правил общения описанных на nometa.xyz.")
               }
           });
            ui.menu("Оформление", || {
                if ui.menu_item("Темное") {
                    state.theme_requested = Some(Dark)
                }
                if ui.menu_item("Светлое") {
                    state.theme_requested = Some(Light)
                }
                if ui.menu_item("Классическое") {
                    state.theme_requested = Some(Classic)
                }
                if ui.menu_item_config("Редактор").selected(state.editor_enabled).build() {
                    state.editor_enabled = !state.editor_enabled
                }
            })
        });
        ui.text_wrapped(self.text);
    }
}
