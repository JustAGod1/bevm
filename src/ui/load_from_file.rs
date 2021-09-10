use std::rc::Rc;
use std::cell::RefCell;
use crate::model::{Memory, Computer, MemoryCell};
use crate::ui::window::Tool;
use imgui::{Ui, im_str, ImString};
use crate::ui::gui::{GuiState, PopupManager};
use crate::ui::popup::{Popup, PopupMessage};
use std::fs::{File, canonicalize, OpenOptions, read_to_string};
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
        fn load_file(state: &mut GuiState, general_memory: bool) {



            let file_name= match nfd::open_file_dialog(Some("mm"), None) {
                Ok(r) => match r {
                    nfd::Response::Okay(f) => {
                        f
                    }
                    _ => {return; }
                }
                Err(e) => {
                    state.popup_manager.open(PopupMessage::new("Ошибка выбора файла", format!("Не могу открыть окно выбора файла: {}", e.to_string())));
                    return;
                }
            };


            let mut f = match File::open(file_name) {
                Ok(mut f) => f,
                Err(e) => {
                    state.popup_manager.open(PopupMessage::new("Ошибка открытия файла", e.to_string()));
                    return;
                }
            };

            let parse_result = if general_memory {
                crate::parse::file::parse_file(&mut f, &state.computer.general_memory.borrow().parser, 0x7FF)
            } else {
                crate::parse::file::parse_file(&mut f, &state.computer.mc_memory.borrow().parser, 0xFF)
            };


            if parse_result.is_err() {
                let msg = parse_result.unwrap_err();
                state.popup_manager.open(PopupMessage::new("Ошибка во время парсинга", msg));
                return;
            }

            let parse_result = parse_result.unwrap();

            fn apply<I: CommandInfo, P: Parser<I>>(mem: &mut Memory<I, P>, data: Vec<(u16, u16)>) {
                for x in &mut mem.data {
                    x.set(0)
                }

                for (pos, v) in data {
                    mem.data.get_mut(pos as usize).unwrap().set(v);
                }
            }
            if general_memory {
                apply(&mut state.computer.general_memory.borrow_mut(), parse_result)
            } else {
                apply(&mut state.computer.mc_memory.borrow_mut(), parse_result)
            }
        }


        if ui.button(im_str!("Загрузить в основную память"), [300.0, 45.0]) {
            load_file(state, true);
        }
        if ui.button(im_str!("Загрузить в память МПУ"), [300.0, 45.0]) {
            load_file(state, false);
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

        fn choose_folder(state: &mut GuiState) -> Option<String> {
            match nfd::open_pick_folder(None) {
                Ok(r) => match r {
                    nfd::Response::Okay(f) => {
                        return Some(f)
                    }
                    _ => {}
                }
                Err(e) => state.popup_manager.open(PopupMessage::new("Ошибка выбора папки", format!("Не могу открыть окно выбора папки: {}", e.to_string())))
            }
            None
        }

        if ui.button(im_str!("Сохранить основную память"), [300.0, 45.0]) {
            if let Some(name) = choose_folder(state) {
                let path = format!("{}/general.mm", name);
                let r = save(path.as_str(), &state.computer.general_memory.borrow());
                report(state, path.as_str(), r);

            }
        }
        if ui.button(im_str!("Сохранить память МПУ"), [300.0, 45.0]) {
            if let Some(name) = choose_folder(state) {
                let path = format!("{}/mc.mm", name);
                let r = save(path.as_str(), &state.computer.mc_memory.borrow());
                report(state, path.as_str(), r);
            }
        }
    }
}


pub struct PopupChoosingFile
{
    general_memory: bool,
    chosen_file: Option<String>,
}