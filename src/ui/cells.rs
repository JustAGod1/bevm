use crate::model::{Computer, MemoryCell, Memory, Register};
use imgui::{Ui, ChildWindow, StyleColor, Id, im_str, ImString, MenuItem};
use crate::parse::mc::parse;
use crate::parse::Parser;
use crate::ui::window::Tool;
use crate::ui::gui::PopupManager;
use std::rc::Rc;
use std::cell::RefCell;
use imgui::__core::cell::RefMut;
use crate::ui::popup::PopupParseError;


#[derive(PartialEq, Eq)]
enum CellRepresentation {
    Hex,
    Binary,
}

impl CellRepresentation {
    fn title(&self) -> String {
        return match self {
            CellRepresentation::Hex => "Шестнадцетеричное".to_string(),
            CellRepresentation::Binary => "Бинарное".to_string(),
        };
    }

    fn draw_hex(&self, cell: &mut MemoryCell, ui: &Ui) {
        let mut data = ImString::from(format!("{:0>4X}", cell.get()));
        let width_t = ui.push_item_width(70.0);
        if ui.input_text(im_str!(""), &mut data)
            .chars_hexadecimal(true)
            .chars_noblank(true)
            .build() {
            let data = data.to_str();
            if let Ok(parsed) = u16::from_str_radix(data, 16) {
                cell.set(parsed)
            }
        }
        width_t.pop(ui);

    }
    fn draw_binary(&self, cell: &mut MemoryCell, ui: &Ui) {
        let mut data = ImString::from(format!("{:0>16b}", cell.get()));
        let width_t = ui.push_item_width(160.0);
        if ui.input_text(im_str!(""), &mut data)
            .chars_decimal(true)
            .chars_noblank(true)
            .build() {
            let data = data.to_str();
            if let Ok(parsed) = u16::from_str_radix(data, 2) {
                cell.set(parsed)
            }
        }
        width_t.pop(ui);

    }
    fn draw(&self, cell: &mut MemoryCell, ui: &Ui) {
        match self {
            CellRepresentation::Hex => self.draw_hex(cell, ui),
            CellRepresentation::Binary => self.draw_binary(cell, ui),
        }

    }

}
pub struct CellsTool<P: Parser, F>
    where F: Fn(&Computer) -> u16
{
    title: String,
    page: Rc<RefCell<Memory<P>>>,
    counter_register: F,
    representation: CellRepresentation
}

impl <P: Parser, F: Fn(&Computer) -> u16>Tool for CellsTool<P, F> {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn draw(&mut self, computer: &mut Computer, ui: &Ui, manager: &mut PopupManager) {
        let mut idx = 0u32;

        self.draw_representation_selection(ui);



        let current_executed = (self.counter_register)(computer);

        let (parser, mut data) = RefMut::map_split(self.page.borrow_mut(), |r| (&mut r.parser, &mut r.data));

        for cell in data.iter_mut() {
            let token = ui.push_id(Id::Int(idx as i32));
            ui.text(format!("{:0>3X}", idx));
            ui.same_line(0.0);
            let t = if current_executed == idx as u16 {
                Some(ui.push_style_color(StyleColor::FrameBg, [1.0, 0.0, 0.0, 1.0]))
            } else {
                None
            };
            self.representation.draw(cell, ui);
            if let Some(t) = t {
                t.pop(ui);
            }


            let command = parser.parse(cell.get());
            ui.same_line(0.0);

            if parser.supports_rev_parse() {
                let mut content = ImString::with_capacity(50);
                content.push_str(command.as_str());
                if ui.input_text(im_str!("###mnemonic"), &mut content)
                    .callback_always(false)
                    .enter_returns_true(true)
                    .build()
                {
                    match parser.rev_parse(content.to_string()) {
                        Ok(opcode) => {
                            cell.set(opcode)
                        }
                        Err(msg) => {
                            manager.open(PopupParseError::new(content.to_string(), msg.to_string()))
                        }
                    }
                }
            } else {
                ui.text(command);
            }

            token.pop(ui);
            idx += 1;
        }
    }
}

impl <P: Parser, F: Fn(&Computer) -> u16> CellsTool<P, F> {

    pub fn new<S: Into<String>>(title: S, page: Rc<RefCell<Memory<P>>>, counter_register: F) -> CellsTool<P, F> {
        CellsTool {
            title: title.into(),
            counter_register,
            page,
            representation: CellRepresentation::Hex
        }
    }

    fn draw_representation_selection(&mut self, ui: &Ui) {
        let token = ui.begin_menu_bar();
        if token.is_none() { return; }
        let token = token.unwrap();

        if let Some(token) = ui.begin_menu(im_str!("Представление ячеек"), true) {
            if MenuItem::new(ImString::from(CellRepresentation::Hex.title()).as_ref())
                .selected(self.representation == CellRepresentation::Hex)
                .build(ui)
            {
                self.representation = CellRepresentation::Hex
            }
            if MenuItem::new(ImString::from(CellRepresentation::Binary.title()).as_ref())
                .selected(self.representation == CellRepresentation::Binary)
                .build(ui)
            {
                self.representation = CellRepresentation::Binary
            }
            token.end(ui)
        }


        token.end(ui);
    }

    pub fn draw(&mut self, computer: &mut Computer, ui: &Ui) {

    }

}