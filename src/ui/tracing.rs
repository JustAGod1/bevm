use crate::ui::window::Tool;
use imgui::{Ui, im_str, Io};
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
        ui.text("Инструмент для создания таблицы трассировок.");
        ui.text("Для более удобного и понятного использования таблица сохраняется в формате CSV");
        let width_t = ui.push_item_width(160.0);
        ui.input_int(im_str!("Максимальная длина таблицы"), &mut self.max_len)
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

            if let Err(e) = f.write("\"Адрес\",\"Код\",\"СК\",\"РА\",\"РК\",\"РД\",\"А\",\"С\",\"Адрес\",\"Новый код\"\n".as_bytes()) {
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
                line.push_str(pos.to_string().as_str()); line.push(',');
                // Code
                line.push_str(code.to_string().as_str()); line.push(',');

                // СК
                line.push_str(state.computer.registers.r_command_counter.to_string().as_str()); line.push(',');
                // РА
                line.push_str(state.computer.registers.r_address.to_string().as_str()); line.push(',');
                // РК
                line.push_str(state.computer.registers.r_command.to_string().as_str()); line.push(',');
                // РД
                line.push_str(state.computer.registers.r_data.to_string().as_str()); line.push(',');
                // А
                line.push_str(state.computer.registers.r_counter.to_string().as_str()); line.push(',');
                // С
                line.push(if state.computer.registers.get_overflow() {'1'} else {'0'}); line.push(',');

                if let Some((pos, nv)) = diff {
                    line.push_str(pos.to_string().as_str()); line.push(',');
                    line.push_str(nv.to_string().as_str());
                }

                line.push('\n');

                if let Err(e) = f.write(line.as_bytes()) {
                    state.popup_manager.open(PopupMessage::new("Ошибка записи", format!("Ошибка записи в файл \"{}\": {}", filename, e.to_string())));
                    return
                }
                steps_left -= 1;
            }

        }


    }
}