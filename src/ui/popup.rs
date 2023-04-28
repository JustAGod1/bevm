use crate::ui::gui::GuiState;
use imgui::Ui;

pub trait Popup {
    fn name(&self) -> String;

    fn draw(&mut self, ui: &Ui, state: &mut GuiState) -> bool;

    fn on_file_dropped(&mut self, _filename: &str) {}
}

pub struct PopupMessage {
    title: String,
    msg: String,
}
impl PopupMessage {
    pub fn new<S: Into<String>, T: Into<String>>(title: T, msg: S) -> PopupMessage {
        PopupMessage {
            title: title.into(),
            msg: msg.into(),
        }
    }
}

impl Popup for PopupMessage {
    fn name(&self) -> String {
        self.title.clone()
    }

    fn draw(&mut self, ui: &Ui, _state: &mut GuiState) -> bool {
        let mut open = true;
        let name = self.name();
        let popup = ui
            .modal_popup_config(name)
            .opened(&mut open)
            .always_auto_resize(true);

        popup.build(|| ui.text(self.msg.as_str()));

        open
    }
}

pub struct PopupParseError {
    src: String,
    msg: String,
}
impl PopupParseError {
    pub fn new(src: String, msg: String) -> PopupParseError {
        PopupParseError { src, msg }
    }
}
impl Popup for PopupParseError {
    fn name(&self) -> String {
        "Ошибка разбора".to_string()
    }

    fn draw(&mut self, ui: &Ui, _state: &mut GuiState) -> bool {
        let mut open = true;
        let name = self.name();
        let popup = ui
            .modal_popup_config(name)
            .opened(&mut open)
            .always_auto_resize(true);

        popup.build(|| {
            ui.text(format!(
                "Произошла ошибка во время разбора выражения {}",
                self.src
            ));
            ui.text(format!("Ошибка: {}", self.msg));
        });

        open
    }
}
