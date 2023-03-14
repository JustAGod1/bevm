use crate::ui::gui::GuiState;
use crate::ui::gui::Theme::{Classic, Dark, Light};
use crate::ui::open_in_app;
use crate::ui::popup::PopupMessage;
use crate::ui::window::Tool;
use imgui::StyleColor::Header;
use imgui::{im_str, Context, ImString, Io, MenuItem, Ui};

pub struct HelpTool {
    text: &'static str,
}

impl HelpTool {
    pub fn new(text: &'static str) -> HelpTool {
        HelpTool { text }
    }
}

impl Tool for HelpTool {
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        ui.menu_bar(|| {
            ui.menu(im_str!("Полезные ссылочки"), true, || {
                if MenuItem::new(im_str!("GitHub")).build(ui) {
                    if let Err(e) = open_in_app("https://github.com/JustAGod1/bevm") {
                        state.popup_manager.open(PopupMessage::new(
                            "Ошибочка",
                            format!("Не смог открыть ссылку: {}", e),
                        ))
                    }
                }
                if MenuItem::new(im_str!("Telegram")).build(ui) {
                    if let Err(e) = open_in_app("https://t.me/notsofunnyhere") {
                        state.popup_manager.open(PopupMessage::new(
                            "Ошибочка",
                            format!("Не смог открыть ссылку: {}", e),
                        ))
                    }
                }
                if MenuItem::new(im_str!("Методичка")).build(ui) {
                    if let Err(e) = open_in_app("https://yadi.sk/i/brIICpYtcb3LMg") {
                        state.popup_manager.open(PopupMessage::new(
                            "Ошибочка",
                            format!("Не смог открыть ссылку: {}", e),
                        ))
                    }
                }
                if MenuItem::new(im_str!("Моя телега")).build(ui) {
                    if let Err(e) = open_in_app("https://t.me/JustAG0d") {
                        state.popup_manager.open(PopupMessage::new(
                            "Ошибочка",
                            format!("Не смог открыть ссылку: {}", e),
                        ))
                    }
                } else if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "Мне желательно писать по поводу идей для новых фич для этой БЭВМ.\n\n\
                    Желательно придерживаться правил общения описанных на nometa.xyz.",
                    )
                }
            });
            ui.menu(im_str!("Оформление"), true, || {
                if MenuItem::new(im_str!("Темное")).build(ui) {
                    state.theme_requested = Some(Dark)
                }
                if MenuItem::new(im_str!("Светлое")).build(ui) {
                    state.theme_requested = Some(Light)
                }
                if MenuItem::new(im_str!("Классическое")).build(ui) {
                    state.theme_requested = Some(Classic)
                }
                if MenuItem::new(im_str!("Редактор"))
                    .selected(state.editor_enabled)
                    .build(ui)
                {
                    state.editor_enabled = !state.editor_enabled
                }
            })
        });
        ui.text_wrapped(ImString::new(self.text).as_ref());
    }
}
