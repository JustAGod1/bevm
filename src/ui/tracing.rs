use crate::ui::window::Tool;
use imgui::{Ui, im_str, Io, ImStr, ImString, ComboBox, Selectable};
use crate::ui::gui::GuiState;
use crate::ui::popup::PopupMessage;
use std::fs::OpenOptions;
use imgui::sys::igGetFontTexUvWhitePixel;
use sdl2::mouse::SystemCursor::No;
use std::io::Write;
use crate::parse::mc::ExecutionResult;
use crate::model::{Registers, Computer};
use std::process::Command;

pub struct TraceTool {
    format: usize,
    converters: [(&'static str, fn(ui: &Ui, state: &mut GuiState, usize)); 2],
    max_len: i32,
}

impl TraceTool {
    pub fn new() -> TraceTool {
        TraceTool {
            format: 0,
            converters: [("CSV", csv_converter), ("HTML", html_converter)],
            max_len: 200
        }
    }
}

fn html_converter(ui: &Ui, state: &mut GuiState, max_len: usize) {
    let text = "Сохраняет трассировку в формате HTML\n\n\
    Это удобно когда вам нужно быстро на нее посмотреть, но неудобно когда нужно ее куда то вставить.\n\n";

    ui.text_wrapped(ImString::new(text).as_ref());

    let save = ui.button(im_str!("Сохранить"), [160.0, 30.0]);
    if ui.is_item_hovered() {
        ui.tooltip_text("Просто сохраняет трассировку")
    }
    let open = ui.button(im_str!("Открыть"), [160.0, 30.0]);
    if ui.is_item_hovered() {
        ui.tooltip_text("Сохраняет и пытается открыть")
    }

    if !open && !save {
        return;
    }


    let trace = perform_tracing(&mut state.computer, max_len);
    let mut content = String::new();

    for x in trace {
        content.push_str("\t<tr>");

        // Address
        content.push_str(format!("\t\t<td>{:0>3X}</td>", x.pos).as_str());
        // Code
        content.push_str(format!("\t\t<td>{:0>4X}</td>", x.code).as_str());

        // СК
        content.push_str(format!("\t\t<td>{:0>4X}</td>", x.registers.r_command_counter).as_str());
        // РА
        content.push_str(format!("\t\t<td>{:0>4X}</td>", x.registers.r_address).as_str());
        // РК
        content.push_str(format!("\t\t<td>{:0>4X}</td>", x.registers.r_command).as_str());
        // РД
        content.push_str(format!("\t\t<td>{:0>4X}</td>", x.registers.r_data).as_str());
        // А
        content.push_str(format!("\t\t<td>{:0>4X}</td>", x.registers.r_counter).as_str());
        // С
        content.push_str(format!("\t\t<td>{}</td>", if x.registers.get_overflow() {'1'} else {'0'}).as_str());

        if let Some((pos, nv)) = x.difference {
            content.push_str(format!("\t\t<td>{:0>3X}</td>", pos).as_str());
            content.push_str(format!("\t\t<td>{:0>4X}</td>", nv).as_str());
        } else {
            content.push_str("\t\t<td colspan=2></td>");
        }
        content.push_str("\t</tr>")
    }

    let formatted = format!("<table border=1> \n\
    <tr> \n\
        <td colspan=2>Выполняемая программа</td> \n\
        <td colspan=6>Содержимое регистров процессора после выполнения команды</td> \n\
        <td colspan=2>Ячейка, содержимое которой изменилось после выполнениия Программы</td> \n\
    </tr> \n\
    <tr> \n\
        <td>Адрес</td> \n\
        <td>Код</td> \n\
        <td>СК</td> \n\
        <td>РА</td> \n\
        <td>РК</td> \n\
        <td>РД</td> \n\
        <td>А</td> \n\
        <td>С</td> \n\
        <td>Адрес</td> \n\
        <td>Новый код</td> \n\
    </tr> \n\
    {} \n\
    </table>", content);

    let name = write_to_file(formatted.as_str(), "html", state);

    if name.is_none() || !open { return; }

    let filename = name.unwrap();

    let cmd_result = match std::env::consts::OS {
        "linux" => Command::new("sh").arg("-c").arg(format!("xdg-open {}", filename)).output(),
        "macos" => Command::new("sh").arg("-c").arg(format!("open {}", filename)).output(),
        "windows" => Command::new("cmd").arg("/c").arg(filename).output(),
        _ => {
            state.popup_manager.open(PopupMessage::new("Упс", format!("Ваша операционная система \"{}\" мне неизвестна. Не смогу тут открыть трассировку.", std::env::consts::OS)));
            return;
        }
    };

    match cmd_result {
        Ok(o) => {
            if o.status.code().unwrap_or(1) != 0 {
                String::from_utf8_lossy(o.stderr.as_slice());
                state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть файл.\n{}\n{}", String::from_utf8_lossy(o.stderr.as_slice()), String::from_utf8_lossy(o.stdout.as_slice()))))
            }
        },
        Err(e) => {
            state.popup_manager.open(PopupMessage::new("Ошибочка", format!("Не смог открыть файл: {}", e.to_string())))
        }
    }




}

fn csv_converter(ui: &Ui, state: &mut GuiState, max_len: usize) {
    let text = "Сохраняет трассировку в формате CSV\n\n\
    Разделитель: таб\n\
    Кодировка: UTF-8\n\
    Все поля закавыченны\n
    ";

    ui.text_wrapped(ImString::new(text).as_ref());
    if ui.button(im_str!("Погнали!"), [160.0, 30.0]) {
        let trace = perform_tracing(&mut state.computer, max_len);

        let mut content = String::from("\"Адрес\"\t\"Код\"\t\"СК\"\t\"РА\"\t\"РК\"\t\"РД\"\t\"А\"\t\"С\"\t\"Адрес\"\t\"Новый код\"\n");

        for x in trace {
            // Address
            content.push_str(format!("\"{:0>3X}\"", x.pos).as_str()); content.push('\t');
            // Code
            content.push_str(format!("\"{:0>4X}\"", x.code).as_str()); content.push('\t');

            // СК
            content.push_str(format!("\"{:0>4X}\"", x.registers.r_command_counter).as_str()); content.push('\t');
            // РА
            content.push_str(format!("\"{:0>4X}\"", x.registers.r_address).as_str()); content.push('\t');
            // РК
            content.push_str(format!("\"{:0>4X}\"", x.registers.r_command).as_str()); content.push('\t');
            // РД
            content.push_str(format!("\"{:0>4X}\"", x.registers.r_data).as_str()); content.push('\t');
            // А
            content.push_str(format!("\"{:0>4X}\"", x.registers.r_counter).as_str()); content.push('\t');
            // С
            content.push(if x.registers.get_overflow() {'1'} else {'0'}); content.push('\t');

            if let Some((pos, nv)) = x.difference {
                content.push_str(format!("\"{:0>3X}\"", pos).as_str()); content.push('\t');
                content.push_str(format!("\"{:0>4X}\"", nv).as_str());
            }

            content.push('\n');
        }

        write_to_file(content.as_str(), "csv", state);
    }

}


impl Tool for TraceTool {
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        let text = "Инструмент для создания таблицы трассировок.\n\n\
            Максимальная длина таблицы:"
            ;

        ui.text_wrapped(ImString::new(text).as_ref());
        let width_t = ui.push_item_width(160.0);
        ui.input_int(im_str!("###max_len"), &mut self.max_len).build();
        width_t.pop(ui);
        self.max_len = self.max_len.clamp(0, 200);
        let text = "Формат таблицы:";
        ui.text_wrapped(ImString::new(text).as_ref());

        let title = self.converters[self.format].0;
        ComboBox::new(im_str!("###selectable"))
            .preview_value(ImString::new(title).as_ref())
            .build(ui, || {
            for i in 0 .. self.converters.len() {
                if Selectable::new(ImString::new(self.converters[i].0).as_ref())
                    .selected(i == self.format)
                    .build(ui)
                {
                    self.format = i;
                }
            }
        });

        self.converters[self.format].1(ui, state, self.max_len as usize)

    }
}

fn write_to_file(s: &str, postfix: &str, state: &mut GuiState) -> Option<String> {
    let filename = match nfd::open_pick_folder(None) {
        Ok(r) => match r {
            nfd::Response::Okay(f) => {
                f
            }
            _ => { return None; }
        }
        Err(e) => {
            state.popup_manager.open(PopupMessage::new("Ошибка выбора папки", format!("Не могу открыть окно выбора папки: {}", e.to_string())));
            return None;
        }
    };

    let filename = format!("{}/tracing.{}", filename, postfix);

    let f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(filename.as_str());

    if let Err(e) = f {
        state.popup_manager.open(PopupMessage::new("Ошибка записи", format!("Не могу открыть файл \"{}\": {}", filename, e.to_string())));
        return None;
    }
    let mut f = f.unwrap();

    if let Err(e) = f.write(s.as_bytes()) {
        state.popup_manager.open(PopupMessage::new("Ошибка записи", format!("Не могу записать в файл \"{}\": {}", filename, e.to_string())));
        return None;
    }

    state.popup_manager.open(PopupMessage::new("Успех", format!("Успешно сохранил трассировку в файл \"{}\"", filename)));

    Some(filename)

}

struct TraceElement {
    pos: u16,
    code: u16,
    registers: Registers,
    difference: Option<(usize, u16)>,
}


fn perform_tracing(computer: &mut Computer, len: usize) -> Vec<TraceElement> {
    let mut steps_left = len;

    let mut result = Vec::new();

    while steps_left > 0 && computer.registers.r_command != 0xF200 {
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

        result.push(
            TraceElement {
                pos,
                code,
                registers: computer.registers.clone(),
                difference: diff
            }
        );

        steps_left -= 1;
    }

    result
}
