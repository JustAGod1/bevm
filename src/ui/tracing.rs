use std::borrow::Cow::{Borrowed, Owned};
use std::cell::RefCell;
use crate::ui::window::Tool;
use imgui::{Ui, im_str, Io, ImStr, ImString, ComboBox, Selectable, TreeNode};
use crate::ui::gui::{GuiState, PopupManager};
use crate::ui::popup::PopupMessage;
use std::fs::OpenOptions;
use imgui::sys::igGetFontTexUvWhitePixel;
use sdl2::mouse::SystemCursor::No;
use std::io::{Read, Write};
use crate::parse::mc::ExecutionResult;
use crate::model::{Registers, Computer};
use imgui::TreeNodeId::Str;
use crate::ui::open_in_app;

pub struct TraceTool {
    converter: usize,
    tracer: usize,
    converters: [(&'static str, fn(ui: &Ui, state: &RefCell<&mut GuiState>, tracing: &mut dyn FnMut() -> Tracing)); 3],
    tracers: [(&'static str, fn(computer: &mut Computer, len: usize) -> Tracing); 2],
    max_len: i32,
}

impl TraceTool {
    pub fn new() -> TraceTool {
        TraceTool {
            converter: 0,
            tracer: 0,
            tracers: [("Основная память", general_tracing), ("Память МПУ", mc_tracing)],
            converters: [("CSV", csv_converter), ("HTML", html_converter), ("LaTeX", latex_converter)],
            max_len: 200
        }
    }
}

fn html_converter(ui: &Ui, state: &RefCell<&mut GuiState>, tracing: &mut dyn FnMut() -> Tracing)
{
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


    let trace = tracing();
    let mut content = String::new();

    content.push_str("\t<tr>");

    for x in trace.header {
        content.push_str("\t\t<th>");
        content.push_str(x.as_str());
        content.push_str("</th>\n");
    }

    content.push_str("\t</tr>");

    for x in trace.tracing {
        content.push_str("\t<tr>");

        for s in x {
            content.push_str(format!("\t\t<td>{}</td>", s).as_str());
        }
        content.push_str("\t</tr>")
    }

    let formatted = format!("<table border=1> \n\
    {} \n\
    </table>", content);

    let name = write_to_file(formatted.as_str(), "html", &mut state.borrow_mut().popup_manager);

    if name.is_none() || !open { return; }

    let filename = name.unwrap();

    if let Err(s) = open_in_app(filename.as_str()) {
        state.borrow_mut().popup_manager.open(PopupMessage::new("Упс", format!("Не удалось открыть файл с трассировкой: {}", s)))
    }

}

fn csv_converter(ui: &Ui, state: &RefCell<&mut GuiState>, tracing: &mut dyn FnMut() -> Tracing) {
    let text = "Сохраняет трассировку в формате CSV\n\n\
    Разделитель: таб\n\
    Кодировка: UTF-8\n\
    Все поля закавыченны\n
    ";

    ui.text_wrapped(ImString::new(text).as_ref());
    if ui.button(im_str!("Погнали!"), [160.0, 30.0]) {
        let trace = tracing();

        let mut content = String::new();
        for x in trace.header.iter().map(|a| format!("{}", a)) {
            content.push_str(x.as_str());
            content.push('\t');
        }

        for x in trace.tracing {
            content.push('\n');
            for x in x.iter().map(|a| format!("{}", a)) {
                content.push('"');
                content.push_str(x.as_str());
                content.push('"');
                content.push('\t');
            }
        }

        write_to_file(content.as_str(), "csv", &mut state.borrow_mut().popup_manager);
    }

}


fn enum_chooser<'a, T>(ui: &Ui, name: &str, num: &mut usize, variants: &'a [(&str, T)]) -> &'a T {
    let title = variants.get(*num).unwrap().0;

    ComboBox::new(ImString::new(name).as_ref())
        .preview_value(ImString::new(title).as_ref())
        .build_simple(ui, num, variants, &|a| Owned(ImString::new(a.0)));

    return &variants.get(*num).unwrap().1
}
fn latex_converter(ui: &Ui, state: &RefCell<&mut GuiState>, tracing: &mut dyn FnMut() -> Tracing) {
    let text = "Сохраняет трассировку в LaTeX\n\n\
    Используются пекеджи: multirow, babel, geometry и longtable.\n\n";
    let warning = "\nУВАГА!!! В связи с тем, что используется longtable, \
    если вы компилите прямо через pdflatex, при первой компиляции таблица может быть \
    отрендерена некорректно (это особенность longtable, никак её не убрать). \
    Просто запустите pdflatex повторно и всё отрендерится как доктор прописал.\n\
    Те, кто пользуется make'ом или latexmk могут спать спокойно. \nСпасибо за внимание!";

    ui.text_wrapped(ImString::new(text).as_ref());
    if ui.button(im_str!("Погнали!"), [160.0, 30.0]) {
        let trace: Tracing = tracing();

        let header = trace.header.join(" & ");

        let mut content = format!("\\documentclass{{article}}\n\
        \\usepackage{{multirow,longtable}}\n\
        \\usepackage[margin=1.5cm]{{geometry}}\n\
        \\usepackage[english,russian]{{babel}}\n\
        \\begin{{document}}\n\
        \\begin{{longtable}}{{|c|c|c|c|c|c|c|c|c|c|}}\n\
        \t\\caption{{Таблица трассировки}} \\\\ \n\
        \t\\hline\n\
        \t\\multicolumn{{2}}{{|c|}}{{Выполняемая команда}} & \n\
        \t\\multicolumn{{6}}{{|c|}}{{Содержимое регистров после выполнения команды}} & \n\
        \t\\multicolumn{{2}}{{|c|}}{{Изменившаяся ячейка}} \\\\\n\
        \t\\hline\n\
        \t{} \\\\\n\
        \t\\hline\n\
        \t\\endfirsthead\n\
        \t\\hline\n\
        \t{} \\\\\n\
        \t\\hline\n\
        \t\\endhead\n\
        \t\\hline\n\
        \t\\endfoot\n", header, header);

        for x in trace.tracing {

            content.push_str(x.join(" & ").as_str());
            content.push_str("\\\\\n");

            content.push_str("\t\\hline\n");
        }
        content.push_str("\\end{longtable}\n\
        \\end{document}\n");
        write_to_file(content.as_str(), "tex", &mut state.borrow_mut().popup_manager);
    }
    TreeNode::new(Str(im_str!("Предупреждение"))).build(ui, || ui.text_wrapped(ImString::new(warning).as_ref()));
}

impl Tool for TraceTool {
    fn draw(&mut self, ui: &Ui, _: &Io, state: &mut GuiState) {
        let text = "Инструмент для создания таблицы трассировок.\n\n\
            Максимальная длина таблицы:"
            ;

        ui.text_wrapped(ImString::new(text).as_ref());

        TreeNode::new(Str(im_str!("Подробности"))).build(ui, || {
            let text =
                "Выполняет программу шаг за шагом так же, как если бы вы нажимали кнопку \"Большой шаг\" и записывали бы это в табличку.\n\n\
                Прямо следует из этого факта, что трассировка будет выполняться начиная с текущего значения регистра СК\n\n\
                БЭВМ будет выполнять команду за командой до тех пор пока либо не достигнет максимальной длинны таблицы, либо в регистре РК не появится значение F000\
                , что, как правило, означает, что выполнилась команда HLT";

            ui.text_wrapped(ImString::new(text).as_ref());
            ui.separator();
            let text =
                "Таким образом, если вы хотите выполнить трассировку программы вам стоит:\n\
                1. Сбросить ЭВМ \n\
                2. Загрузить программу \n\
                3. Установить регистр СК в начало программы \n\
                4. Выполнить трассировку";

            ui.text_wrapped(ImString::new(text).as_ref());
            ui.separator();
        });

        let width_t = ui.push_item_width(160.0);
        ui.input_int(im_str!("###max_len"), &mut self.max_len).build();
        width_t.pop(ui);
        self.max_len = self.max_len.clamp(0, 200);

        let text = "Формат таблицы:";
        ui.text_wrapped(ImString::new(text).as_ref());
        let converter = enum_chooser(ui, "###converter", &mut self.converter, &self.converters);

        let text = "Вид трассировки:";
        ui.text_wrapped(ImString::new(text).as_ref());
        let tracer = enum_chooser(ui, "###tracer", &mut self.tracer, &self.tracers);



        let cell = RefCell::new(state);
        converter(ui, &cell, &mut || tracer(&mut cell.borrow_mut().computer, self.max_len as usize))

    }
}

fn write_to_file(s: &str, postfix: &str, popup_manager: &mut PopupManager) -> Option<String> {
    let filename = match nfd::open_pick_folder(None) {
        Ok(r) => match r {
            nfd::Response::Okay(f) => {
                f
            }
            _ => { return None; }
        }
        Err(e) => {
            popup_manager.open(PopupMessage::new("Ошибка выбора папки", format!("Не могу открыть окно выбора папки: {}", e.to_string())));
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
        popup_manager.open(PopupMessage::new("Ошибка записи", format!("Не могу открыть файл \"{}\": {}", filename, e.to_string())));
        return None;
    }
    let mut f = f.unwrap();

    if let Err(e) = f.write(s.as_bytes()) {
        popup_manager.open(PopupMessage::new("Ошибка записи", format!("Не могу записать в файл \"{}\": {}", filename, e.to_string())));
        return None;
    }

    popup_manager.open(PopupMessage::new("Успех", format!("Успешно сохранил трассировку в файл \"{}\"", filename)));

    Some(filename)

}

struct Tracing {
    pub header: Vec<String>,
    pub tracing: Vec<Vec<String>>
}

fn mc_tracing(computer: &mut Computer, len: usize) -> Tracing {
    let mut steps_left = len;

    let mut result = Vec::new();

    while steps_left > 0 {
        let pos = computer.registers.r_micro_command_counter;
        let code = computer.mc_memory.borrow().data.get(pos as usize).unwrap().get();

        computer.registers.set_execute_by_tick(false);
        computer.registers.set_lever(false);
        computer.registers.set_program_mode(false);

        computer.micro_step();

        result.push(
            vec![
                format!("{:0>3X}", pos),
                format!("{:0>4X}", code),
                format!("{:0>3X}", computer.registers.r_command_counter),
                format!("{:0>3X}", computer.registers.r_address),
                format!("{:0>4X}", computer.registers.r_command),
                format!("{:0>4X}", computer.registers.r_data),
                format!("{:0>4X}", computer.registers.r_counter),
                if computer.registers.get_overflow() { "1".to_owned() } else { "0".to_owned() },
                format!("{:0>4X}", computer.registers.r_buffer),
                if computer.registers.get_negative() { "1".to_owned() } else { "0".to_owned() },
                if computer.registers.get_null() { "1".to_owned() } else { "0".to_owned() },
                format!("{:0>3X}", computer.registers.r_micro_command_counter)
            ]
        );

        if computer.registers.r_command == 0xF000 { break; }

        steps_left -= 1;
    }

    Tracing {
        header: vec!["СчМК до выборки МК", "ВМК", "СК", "РА", "РК", "РД", "А", "С", "БР", "N", "Z", "СчМК"].iter().map(|a| a.to_owned().to_owned()).collect(),
        tracing: result
    }
}
fn general_tracing(computer: &mut Computer, len: usize) -> Tracing {
    let mut steps_left = len;

    let mut result = Vec::new();

    while steps_left > 0 {
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

        let mut line = vec![
            format!("{:0>3X}", pos),
            format!("{:0>4X}", code),
            format!("{:0>4X}", computer.registers.r_command_counter),
            format!("{:0>4X}", computer.registers.r_address),
            format!("{:0>4X}", computer.registers.r_command),
            format!("{:0>4X}", computer.registers.r_data),
            format!("{:0>4X}", computer.registers.r_counter),
            if computer.registers.get_overflow() { "1".to_owned() } else { "0".to_owned() },
        ];
        for i in 0..mem_before.len() {
            if computer.general_memory.borrow().data.get(i).unwrap().get() != mem_before.get(i).unwrap().get() {
                line.push(format!("{:0>3X}", i));
                line.push(format!("{:0>4X}", computer.general_memory.borrow().data.get(i).unwrap().get()));
            }
        }
        result.push(line);

        steps_left -= 1;

        if computer.registers.r_command == 0xF000 { break; }
    }

    Tracing {
        header: vec!["Адресс", "Код", "СК", "РА", "РК", "РД", "А", "С", "Адрес", "Новый код"].iter().map(|a| a.to_owned().to_owned()).collect(),
        tracing: result
    }
}
