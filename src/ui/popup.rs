use crate::model::Computer;
use imgui::{Ui, ImString, im_str, ItemFlag, StyleVar};

pub trait Popup {

    fn name(&self) -> ImString;

    fn draw(&mut self, ui: &Ui, computer: &mut Computer) -> bool;

    fn on_file_dropped(&mut self, filename: &str) {}
}


pub struct PopupChoosingFile<F>
    where F: Fn(String)
{
    callback: F,
    chosen_file: Option<String>
}

impl <F>PopupChoosingFile<F>
    where F: Fn(String)
{

    pub fn new(callback: F) -> PopupChoosingFile<F>
    {
        PopupChoosingFile {
            callback,
            chosen_file: None
        }
    }
}

impl <F>Popup for PopupChoosingFile<F>
    where F: Fn(String)
{
    fn name(&self) -> ImString {
        ImString::new("Выбор файла")
    }

    fn draw(&mut self, ui: &Ui, computer: &mut Computer) -> bool {
        let mut open = true;
        let name = self.name();
        let popup = ui.popup_modal(name.as_ref())
            .opened(&mut open)
            .always_auto_resize(true);

        let mut open  = true;

        popup.build(|| {
            ui.text("Перетащите файл сюда");
            match &self.chosen_file {
                Some(f) => {
                    ui.text(format!("Файл: {}", f));

                    if ui.button(im_str!("Подтвердить"), [100.0, 0.0]) {
                        (self.callback)(f.to_string());
                        ui.close_current_popup();
                        open = false
                    }
                },
                None => {
                    ui.text(format!("Файл: Не выбран"));
                    if ui.button(im_str!("Отменить"), [100.0, 0.0]) {
                        ui.close_current_popup();
                        open = false
                    }
                },
            }

        });

        return open
    }

    fn on_file_dropped(&mut self, filename: &str) {
        self.chosen_file = Some(filename.to_string())
    }
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
