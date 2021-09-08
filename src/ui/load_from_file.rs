use std::rc::Rc;
use std::cell::RefCell;
use crate::model::{Memory, Computer, MemoryCell};
use crate::ui::window::Tool;
use imgui::{Ui, im_str, ImString};
use crate::ui::gui::{GuiState, PopupManager};
use crate::ui::popup::{Popup, PopupMessage};
use std::fs::{File, canonicalize, OpenOptions};
use crate::parse::file::parse_file;
use std::io::{BufReader, Write};
use crate::parse::{CommandInfo, Parser};
use std::path::PathBuf;

pub struct LoadFromFileTool {}

impl LoadFromFileTool {
    pub fn new() -> LoadFromFileTool {
        LoadFromFileTool {}
    }
}

impl Tool for LoadFromFileTool {
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {
        if ui.button(im_str!("Загрузить в основную память"), [300.0, 45.0]) {
            state.popup_manager.open(PopupChoosingFile::new(true, state.last_file_general.clone()));
        }
        if ui.button(im_str!("Загрузить в память МПУ"), [300.0, 45.0]) {
            state.popup_manager.open(PopupChoosingFile::new(false, state.last_file_mc.clone()));
        }

        fn report(state: &mut GuiState, file: &str, r: Result<(), String>) {
            let path = match std::env::current_dir() {
                Err(e) => e.to_string(),
                Ok(p) => p.join(file).to_str().unwrap_or("Не могу прочитать").to_string()
            };

            match r {
                Ok(_) => state.popup_manager.open(PopupMessage::new("Успех", format!("Успешно сохранил в файл {}", path))),
                Err(e) => state.popup_manager.open(PopupMessage::new("Провал", format!("Не могу сохранить в файл \"{}\": {}", path, e)))
            }
        }

        fn save<I: CommandInfo, P: Parser<I>>(file: &str, data: &Memory<I, P>) -> Result<(), String> {
            let mut f = OpenOptions::new()
                .create(true)
                .append(false)
                .write(true)
                .truncate(true)
                .open(file)
                .map_err(|e| e.to_string())?;


            let mut s = String::new();
            let mut prev_zero = true;
            let mut prev_prev_zero = true;

            let mut pos = 0usize;
            for cell in &data.data {
                prev_prev_zero = prev_zero;

                let v = cell.get();
                if v == 0 {
                    prev_zero = true
                } else {
                    if prev_prev_zero && prev_zero {
                        s.push_str(format!("$pos {:X}\n", pos).as_str())
                    }
                    if data.parser.supports_rev_parse() {
                        let mnemonic = data.parser.parse(v).mnemonic();
                        s.push_str(mnemonic.as_str())
                    } else {
                        s.push_str(format!("{:0>4X}", v).as_str())
                    }
                    s.push('\n');
                    prev_zero = false;
                }

                pos += 1;
            }

            f.write(s.as_bytes());
            f.flush();

            Ok(())
        }

        ui.separator();

        if ui.button(im_str!("Сохранить основную память"), [300.0, 45.0]) {
            let r = save("general.mm", &state.computer.general_memory.borrow());

            report(state, "general.mm", r);
        }
        if ui.button(im_str!("Сохранить память МПУ"), [300.0, 45.0]) {
            let r = save("mc.mm", &state.computer.mc_memory.borrow());
            report(state, "mc.mm", r);
        }
    }
}


pub struct PopupChoosingFile
{
    general_memory: bool,
    chosen_file: Option<String>,
}

impl PopupChoosingFile
{
    pub fn new(general_memory: bool, chosen_file: Option<String>) -> PopupChoosingFile
    {
        PopupChoosingFile {
            general_memory,
            chosen_file,
        }
    }
}

impl Popup for PopupChoosingFile
{
    fn name(&self) -> ImString {
        ImString::new("Выбор файла")
    }

    fn draw(&mut self, ui: &Ui, state: &mut GuiState) -> bool {
        let mut open = true;
        let name = self.name();
        let popup = ui.popup_modal(name.as_ref())
            .opened(&mut open)
            .always_auto_resize(true);


        let mut open = false;

        popup.build(|| {
            open = true;
            ui.text("Перетащите файл сюда");
            match &self.chosen_file {
                Some(f) => {
                    ui.text(format!("Файл: {}", f));

                    if ui.button(im_str!("Подтвердить"), [100.0, 0.0]) {
                        if self.general_memory {
                            state.last_file_general = Some(f.clone())
                        } else {
                            state.last_file_mc = Some(f.clone())
                        };
                        match File::open(f) {
                            Ok(mut f) => self.parse_file(&mut f, state),
                            Err(e) => state.popup_manager.open(PopupMessage::new("Ошибка открытия файла", e.to_string()))
                        }

                        ui.close_current_popup();
                        open = false
                    }
                }
                None => {
                    ui.text(format!("Файл: Не выбран"));
                    if ui.button(im_str!("Отменить"), [100.0, 0.0]) {
                        ui.close_current_popup();
                        open = false
                    }
                }
            }
        });

        return open;
    }

    fn on_file_dropped(&mut self, filename: &str) {
        self.chosen_file = Some(filename.to_string())
    }
}


impl PopupChoosingFile {
    fn parse_file(&self, f: &mut File, state: &mut GuiState) {
        let parse_result = if self.general_memory {
            parse_file(f, &state.computer.general_memory.borrow().parser, 0x7FF)
        } else {
            parse_file(f, &state.computer.mc_memory.borrow().parser, 0x7FF)
        };


        if parse_result.is_err() {
            let msg = parse_result.unwrap_err();
            state.popup_manager.open(PopupMessage::new("Ошибка во время парсинга", msg));
            return;
        }

        let parse_result = parse_result.unwrap();

        if self.general_memory {
            Self::apply(&mut state.computer.general_memory.borrow_mut(), parse_result)
        } else {
            Self::apply(&mut state.computer.mc_memory.borrow_mut(), parse_result)
        }
    }

    fn apply<I: CommandInfo, P: Parser<I>>(mem: &mut Memory<I, P>, data: Vec<(u16, u16)>) {
        for x in &mut mem.data {
            x.set(0)
        }

        for (pos, v) in data {
            mem.data.get_mut(pos as usize).unwrap().set(v);
        }
    }
}