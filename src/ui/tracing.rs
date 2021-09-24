use crate::ui::window::Tool;
use imgui::{Ui, im_str, Io, ImStr, ImString};
use crate::ui::gui::GuiState;
use crate::ui::popup::PopupMessage;
use std::fs::OpenOptions;
use imgui::sys::igGetFontTexUvWhitePixel;
use sdl2::mouse::SystemCursor::No;
use std::io::Write;
use crate::parse::mc::ExecutionResult;

pub struct TraceTool {
    max_len: i32,
}

impl TraceTool {
    pub fn new() -> TraceTool {
        TraceTool {
            max_len: 200
        }
    }
}


impl Tool for TraceTool {
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {

        let text = "Инструмент для создания таблицы трассировок.\n\n\
            Для более удобного и понятного использования таблица сохраняется в формате CSV\n\n\
            Excel -> File -> Import\n\n\
            Важно сказать эксэлю, что нужно форматировать закавыченные ячейки как текст\n\n\
            Максимальная длина таблицы:"
            ;

        ui.text_wrapped(ImString::new(text).as_ref());
        let width_t = ui.push_item_width(160.0);
        ui.input_int(im_str!("###max_len"), &mut self.max_len)
            .build();
        width_t.pop(ui);
        self.max_len = self.max_len.clamp(0, 200);

        if ui.button(im_str!("Погнали!"), [160.0, 30.0]) {
            let filename = match nfd::open_pick_folder(None) {
                Ok(r) => match r {
                    nfd::Response::Okay(f) => {
                        f
                    }
                    _ => {return;}
                }
                Err(e) => {
                    state.popup_manager.open(PopupMessage::new("Ошибка выбора папки", format!("Не могу открыть окно выбора папки: {}", e.to_string())));
                    return;
                }
            };

            let filename = format!("{}/tracing.csv", filename);

            let f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(filename.as_str());

            if let Err(e) = f {
                state.popup_manager.open(PopupMessage::new("Ошибка записи", format!("Не могу открыть файл \"{}\": {}", filename, e.to_string())));
                return;
            }
            let mut f = f.unwrap();


            let mut steps_left = self.max_len;

            if let Err(e) = f.write("\"Адрес\"\t\"Код\"\t\"СК\"\t\"РА\"\t\"РК\"\t\"РД\"\t\"А\"\t\"С\"\t\"Адрес\"\t\"Новый код\"\n".as_bytes()) {
                state.popup_manager.open(PopupMessage::new("Ошибка записи", format!("Ошибка записи в файл \"{}\": {}", filename, e.to_string())));
                return
            }

            while steps_left > 0 && state.computer.registers.r_command != 0xF200 {
                let computer = &mut state.computer;
                let pos = computer.registers.r_command_counter;
                let code = computer.general_memory.borrow().data.get(pos as usize).unwrap().get();
                let mem_before = computer.general_memory.borrow().data.clone();

                computer.registers.set_execute_by_tick(false);
                computer.registers.set_lever(false);
                computer.registers.set_program_mode(false);
                while !matches!(computer.micro_step(), ExecutionResult::HALTED) {}


                let mut diff: Option<(usize, u16)> = None;
                for i in 0..mem_before.len() {
                    if computer.general_memory.borrow().data.get(i).unwrap().get() != mem_before.get(i).unwrap().get() {
                        diff = Some((i, computer.general_memory.borrow().data.get(i).unwrap().get()));
                    }
                }

                let mut line = String::new();

                // Address
                line.push_str(format!("\"{:0>3X}\"", pos).as_str()); line.push('\t');
                // Code
                line.push_str(format!("\"{:0>4X}\"", code).as_str()); line.push('\t');

                // СК
                line.push_str(format!("\"{:0>4X}\"", state.computer.registers.r_command_counter).as_str()); line.push('\t');
                // РА
                line.push_str(format!("\"{:0>4X}\"", state.computer.registers.r_address).as_str()); line.push('\t');
                // РК
                line.push_str(format!("\"{:0>4X}\"", state.computer.registers.r_command).as_str()); line.push('\t');
                // РД
                line.push_str(format!("\"{:0>4X}\"", state.computer.registers.r_data).as_str()); line.push('\t');
                // А
                line.push_str(format!("\"{:0>4X}\"", state.computer.registers.r_counter).as_str()); line.push('\t');
                // С
                line.push(if state.computer.registers.get_overflow() {'1'} else {'0'}); line.push('\t');

                if let Some((pos, nv)) = diff {
                    line.push_str(format!("\"{:0>3X}\"", pos).as_str()); line.push('\t');
                    line.push_str(format!("\"{:0>4X}\"", nv).as_str());
                }

                line.push('\n');

                if let Err(e) = f.write(line.as_bytes()) {
                    state.popup_manager.open(PopupMessage::new("Ошибка записи", format!("Ошибка записи в файл \"{}\": {}", filename, e.to_string())));
                    return
                }
                steps_left -= 1;
            }

            state.popup_manager.open(PopupMessage::new("Успех", format!("Успешно сохранил трассировку в {}", filename)));
        }


    }
}