use crate::model::Computer;
use imgui::{Ui, ImString, im_str};

pub trait Popup {

    fn name(&self) -> ImString;

    fn draw(&mut self, ui: &Ui, computer: &mut Computer) -> bool;

}


pub struct PopupHalted;
impl PopupHalted {
    pub fn new() -> PopupHalted {
        PopupHalted{}
    }

}
impl Popup for PopupHalted {


    fn name(&self) -> ImString { ImString::from("Остановочка".to_string()) }

    fn draw(&mut self, ui: &Ui, computer: &mut Computer) -> bool{
        let mut open = true;
        let popup = ui.popup_modal(im_str!("Остановочка"))
            .opened(&mut open)
            .always_auto_resize(true);


        popup.build(|| {
                ui.text("ЭВМ завершила свою работу")
            });

        return open
    }
}

pub struct PopupParseError {
    src: String,
    msg: String
}
impl PopupParseError {
    pub fn new(src: String, msg: String) -> PopupParseError {
        PopupParseError {
            src,
            msg
        }
    }

}
impl Popup for PopupParseError {


    fn name(&self) -> ImString { ImString::from("Ошибка разбора".to_string()) }

    fn draw(&mut self, ui: &Ui, computer: &mut Computer) -> bool{
        let mut open = true;
        let name = self.name();
        let popup = ui.popup_modal(name.as_ref())
            .opened(&mut open)
            .always_auto_resize(true);


        popup.build(|| {
            ui.text(format!("Произошла ошибка во время разбора выражения {}", self.src));
            ui.text(format!("Ошибка: {}", self.msg));
        });

        return open
    }
}
