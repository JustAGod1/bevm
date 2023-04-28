use crate::model::{Computer, Memory, MemoryCell};
use crate::parse::{CommandInfo, Parser};
use crate::ui::gui::GuiState;
use crate::ui::popup::{PopupMessage, PopupParseError};
use crate::ui::window::Tool;
use imgui::__core::cell::RefMut;
use imgui::{InputTextFlags, Io, StyleColor, StyleVar, Ui};
use rfd::FileDialog;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::rc::Rc;

#[derive(PartialEq, Eq)]
enum CellRepresentation {
    Hex,
    Binary,
}

impl CellRepresentation {
    fn title(&self) -> &'static str {
        match self {
            CellRepresentation::Hex => "Шестнадцетеричное",
            CellRepresentation::Binary => "Бинарное",
        }
    }

    fn draw_hex(&self, cell: &mut MemoryCell, ui: &Ui) {
        let cell = cell;
        let mut data = format!("{:0>4X}", cell.get());
        let width_t = ui.push_item_width(70.0);
        if ui
            .input_text("", &mut data)
            .chars_hexadecimal(true)
            .chars_noblank(true)
            .build()
        {
            if let Ok(parsed) = u16::from_str_radix(&data, 16) {
                cell.set(parsed)
            }
        }
        width_t.end();
    }
    fn draw_binary(&self, cell: &mut MemoryCell, ui: &Ui) {
        let mut data = format!("{:0>16b}", cell.get());
        let width_t = ui.push_item_width(160.0);
        if ui
            .input_text("", &mut data)
            .chars_decimal(true)
            .chars_noblank(true)
            .build()
        {
            let data = data;
            if let Ok(parsed) = u16::from_str_radix(&data, 2) {
                cell.set(parsed)
            }
        }
        width_t.end()
    }
    fn draw(&self, cell: &mut MemoryCell, ui: &Ui) {
        match self {
            CellRepresentation::Hex => self.draw_hex(cell, ui),
            CellRepresentation::Binary => self.draw_binary(cell, ui),
        }
    }
}

pub struct CellsTool<I: CommandInfo, P: Parser<I>, F>
where
    F: Fn(&Computer) -> u16,
{
    page: Rc<RefCell<Memory<I, P>>>,
    counter_register: F,
    representation: CellRepresentation,
}

impl<I: CommandInfo, P: Parser<I>, F: Fn(&Computer) -> u16> Tool for CellsTool<I, P, F>
where
    I: 'static,
{
    fn draw(&mut self, ui: &Ui, _io: &Io, state: &mut GuiState) {
        self.draw_menu_bar(state, ui);

        let s_token = ui.push_style_var(StyleVar::ChildBorderSize(0.0));

        let current_executed = (self.counter_register)(&mut state.computer);

        let mut next_rev_focused = false;

        let (parser, mut data) =
            RefMut::map_split(self.page.borrow_mut(), |r| (&mut r.parser, &mut r.data));

        let mut focused: Option<I> = None;

        for (idx, cell) in data.iter_mut().enumerate() {
            let token = ui.push_id(idx.to_string());
            ui.text(format!("{:0>3X}", idx));
            ui.same_line();
            let t = if current_executed == idx as u16 {
                if state.jump_requested {
                    ui.set_scroll_here_y();
                    state.jump_requested = false;
                }
                Some(ui.push_style_color(StyleColor::FrameBg, [1.0, 0.0, 0.0, 1.0]))
            } else {
                None
            };
            self.representation.draw(cell, ui);
            if let Some(t) = t {
                t.pop();
            }
            if ui.is_item_focused() {
                focused = Some(parser.parse(cell.get()))
            }

            ui.same_line();
            let command = parser.parse(cell.get());

            if parser.supports_rev_parse() {
                let mut content = String::with_capacity(50);
                if next_rev_focused {
                    ui.set_keyboard_focus_here();
                    next_rev_focused = false
                }
                content.push_str(command.mnemonic().as_str());
                if ui
                    .input_text("###mnemonic", &mut content)
                    .flags(InputTextFlags::empty())
                    .enter_returns_true(true)
                    .build()
                {
                    match parser.rev_parse(&content) {
                        Ok(opcode) => {
                            next_rev_focused = true;
                            cell.set(opcode);
                        }
                        Err(msg) => state
                            .popup_manager
                            .open(PopupParseError::new(content.to_string(), msg.to_string())),
                    }
                }

                if ui.is_item_focused() {
                    focused = Some(command)
                }
            } else {
                ui.text(command.mnemonic().as_str());
            }

            token.pop();
        }

        if let Some(focused) = focused {
            state.current_command = Some(Box::new(focused));
        } else {
            state.current_command = Some(Box::new(
                parser.parse(data.get(current_executed as usize).unwrap().get()),
            ));
        }

        s_token.pop();
    }
}

impl<I: CommandInfo, P: Parser<I>, F: Fn(&Computer) -> u16> CellsTool<I, P, F> {
    pub fn new(page: Rc<RefCell<Memory<I, P>>>, counter_register: F) -> CellsTool<I, P, F> {
        CellsTool {
            counter_register,
            page,
            representation: CellRepresentation::Hex,
        }
    }

    fn draw_menu_bar(&mut self, state: &mut GuiState, ui: &Ui) {
        ui.menu_bar(|| {
            ui.menu("Опции", || {
                self.draw_file_actions(state, ui);
                self.draw_representation_selection(ui);
            });
        })
    }

    fn on_save_to_file(&mut self, state: &mut GuiState) {
        let Some(filename) = FileDialog::new()
            .add_filter("", &["mm"])
            .save_file() else {
                state.popup_manager.open(PopupMessage::new(
                    "Ошибка выбора файла",
                    format!("Не удалось выбрать файл"),
                ));
                return;
        };
        let filename = filename
            .into_os_string()
            .into_string()
            .unwrap_or("".to_owned());

        match self.save_to_file(&filename) {
            Ok(_) => state.popup_manager.open(PopupMessage::new(
                "Успех",
                format!("Успешно сохранил в файл {}", filename),
            )),
            Err(e) => state.popup_manager.open(PopupMessage::new(
                "Провал",
                format!("Не могу сохранить в файл \"{}\": {}", filename, e),
            )),
        }
    }

    fn save_to_file(&mut self, file: &str) -> Result<(), String> {
        let mut f = OpenOptions::new()
            .create(true)
            .append(false)
            .write(true)
            .truncate(true)
            .open(file)
            .map_err(|e| e.to_string())?;

        let mut s = String::new();
        let mut prev_zero = true;

        for (pos, cell) in self.page.borrow().data.iter().enumerate() {
            let v = cell.get();
            if v == 0 {
                prev_zero = true
            } else {
                if prev_zero {
                    s.push_str(format!("$pos {:X}\n", pos).as_str())
                }
                let str = self.page.borrow().parser.parse(v).file_string();
                s.push_str(str.as_str());
                s.push('\n');
                prev_zero = false;
            }
        }

        f.write(s.as_bytes()).map_err(|_| "Can't write file")?;
        f.flush().map_err(|_| "Can't write file")?;

        Ok(())
    }

    fn choose_file(state: &mut GuiState, filter: Option<&str>) -> Option<File> {
        let filter = filter.map(|f| [f]);
        let dialog = if filter.is_none() {
            FileDialog::new()
        } else {
            FileDialog::new().add_filter("", filter.as_ref().unwrap())
        };

        let Some(file_name) = dialog.pick_file() else {
            state.popup_manager.open(PopupMessage::new(
                "Ошибка выбора файла",
                format!("Не удалось выбрать файл"),
            ));
            return None;
        };
        let file_name = file_name
            .into_os_string()
            .into_string()
            .unwrap_or("".to_owned());

        File::open(file_name).map(Some).unwrap_or_else(|e| {
            state
                .popup_manager
                .open(PopupMessage::new("Ошибка открытия файла", e.to_string()));
            None
        })
    }

    fn on_load_from_file(&mut self, state: &mut GuiState) {
        let Some(mut f) = Self::choose_file(state, Some("mm")) else { return };

        let parse_result =
            match crate::parse::file::parse_file(&mut f, &self.page.borrow().parser, 0xFF) {
                Ok(result) => result,
                Err(msg) => {
                    state
                        .popup_manager
                        .open(PopupMessage::new("Ошибка во время парсинга", msg));
                    return;
                }
            };

        let mem = &mut self.page.borrow_mut().data;
        for x in mem.iter_mut() {
            x.set(0)
        }

        for (pos, v) in parse_result {
            mem.get_mut(pos as usize).unwrap().set(v);
        }
    }

    fn load_bpc(&mut self, state: &mut GuiState) {
        let Some(f) = Self::choose_file(state, Some("bpc")) else {
            return;
        };

        let mut start_pos: Option<u16> = None;

        let mut parse_result = Vec::<(u16, u16)>::new();

        for (line, line_num) in BufReader::new(f).lines().zip(1..) {
            match line {
                Ok(line) => {
                    let split: Vec<&str> = line.split(' ').collect();

                    if split.len() < 2 {
                        state.popup_manager.open(PopupMessage::new(
                            "Ошибочка",
                            format!("Неверный формат({}) на строчке {}", line, line_num),
                        ));
                        return;
                    }

                    let pos = u16::from_str_radix(split[0], 16);
                    if let Err(_e) = pos {
                        state.popup_manager.open(PopupMessage::new(
                            "Ошибочка",
                            format!(
                                "Не могу распарсить позицию {} на строчке {}",
                                split[0], line_num
                            ),
                        ));
                        return;
                    }
                    let pos = pos.unwrap();

                    let cmd_str = if split[1].starts_with('+') {
                        start_pos = Some(pos);
                        split[1]
                    } else {
                        &split[1][1..]
                    };

                    let Ok(cmd) = u16::from_str_radix(cmd_str, 16) else {
                        state.popup_manager.open(PopupMessage::new(
                            "Ошибочка",
                            format!(
                                "Не могу распарсить команду {} на строчке {}",
                                cmd_str, line_num
                            ),
                        ));
                        return;
                    };

                    parse_result.push((pos, cmd))
                }
                Err(e) => state
                    .popup_manager
                    .open(PopupMessage::new("Ошибочка", e.to_string())),
            }
        }

        self.page
            .borrow_mut()
            .data
            .iter_mut()
            .for_each(|x| x.set(0));

        for (pos, v) in parse_result {
            self.page
                .borrow_mut()
                .data
                .get_mut(pos as usize)
                .unwrap()
                .set(v);
        }

        if let Some(pos) = start_pos {
            state.computer.registers.r_command_counter = pos;
        }
    }

    fn draw_file_actions(&mut self, state: &mut GuiState, ui: &Ui) {
        if let Some(token) = ui.begin_menu("Файл") {
            if ui.menu_item("Сохранить") {
                self.on_save_to_file(state);
            }
            if ui.menu_item("Загрузить") {
                self.on_load_from_file(state);
            }
            if ui.menu_item("Загрузить .bpc") {
                self.load_bpc(state);
            }

            token.end()
        }
    }

    fn draw_representation_selection(&mut self, ui: &Ui) {
        if let Some(token) = ui.begin_menu("Представление ячеек") {
            if ui
                .menu_item_config(CellRepresentation::Hex.title())
                .selected(self.representation == CellRepresentation::Hex)
                .build()
            {
                self.representation = CellRepresentation::Hex
            }
            if ui
                .menu_item_config(CellRepresentation::Binary.title())
                .selected(self.representation == CellRepresentation::Binary)
                .build()
            {
                self.representation = CellRepresentation::Binary
            }
            token.end()
        }
    }
}
