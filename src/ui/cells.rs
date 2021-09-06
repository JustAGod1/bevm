use crate::model::{Computer, MemoryCell, Memory, Register};
use imgui::{Ui, ChildWindow, StyleColor, Id, im_str, ImString, MenuItem, FocusedWidget, StyleVar};
use crate::parse::mc::parse;
use crate::parse::{Parser, CommandInfo};
use crate::ui::window::Tool;
use crate::ui::gui::{PopupManager, Gui, GuiState};
use std::rc::Rc;
use std::cell::RefCell;
use imgui::__core::cell::RefMut;
use crate::ui::popup::{PopupParseError, PopupChoosingFile};


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
        let mut cell = cell;
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
        let mut cell = cell;
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
pub struct CellsTool<I: CommandInfo, P: Parser<I>, F>
    where F: Fn(&Computer) -> u16
{
    page: Rc<RefCell<Memory<I, P>>>,
    counter_register: F,
    representation: CellRepresentation
}

impl <I: CommandInfo, P: Parser<I>, F: Fn(&Computer) -> u16>Tool for CellsTool<I, P, F>
    where I: 'static
{
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {
        let mut idx = 0u32;

        self.draw_load_from_file(ui, state);
        self.draw_representation_selection(ui);

        let jump_needed = ui.button(im_str!("Перейти к исполняемой команде"), [0.0, 0.0]);

        let w_token = ChildWindow::new("cells_inside")
            .always_vertical_scrollbar(true)
            .border(true)
            .begin(ui).unwrap();


        let current_executed = (self.counter_register)(&mut state.computer);

        let mut next_rev_focused = false;

        let (parser, mut data) = RefMut::map_split(self.page.borrow_mut(), |r| (&mut r.parser, &mut r.data));

        let mut focused: Option<I> = None;

        for cell in data.iter_mut() {
            let token = ui.push_id(Id::Int(idx as i32));
            ui.text(format!("{:0>3X}", idx));
            ui.same_line(0.0);
            let t = if current_executed == idx as u16 {
                if jump_needed {
                    ui.set_scroll_here_y();
                }
                Some(ui.push_style_color(StyleColor::FrameBg, [1.0, 0.0, 0.0, 1.0]))
            } else {
                None
            };
            self.representation.draw(cell, ui);
            if let Some(t) = t {
                t.pop(ui);
            }
            if ui.is_item_focused() {
                focused = Some(parser.parse(cell.get()))
            }


            ui.same_line(0.0);
            let command = parser.parse(cell.get());

            if parser.supports_rev_parse() {
                let mut content = ImString::with_capacity(50);
                if next_rev_focused {
                    ui.set_keyboard_focus_here(FocusedWidget::Next);
                    next_rev_focused = false
                }
                content.push_str(command.mnemonic().as_str());
                if ui.input_text(im_str!("###mnemonic"), &mut content)
                    .callback_always(false)
                    .enter_returns_true(true)
                    .build()
                {
                    match parser.rev_parse(content.to_str()) {
                        Ok(opcode) => {
                            next_rev_focused = true;
                            cell.set(opcode);
                        }
                        Err(msg) => {
                            state.popup_manager.open(PopupParseError::new(content.to_string(), msg.to_string()))
                        }
                    }
                }

                if ui.is_item_focused() {
                    focused = Some(command)
                }
            } else {
                ui.text(command.mnemonic().as_str());
            }

            token.pop(ui);
            idx += 1;
        }


        if focused.is_some() {
            state.current_command = Some(Box::new(focused.unwrap()));
        } else {
           state.current_command = Some(Box::new(parser.parse(data.get(current_executed as usize).unwrap().get())));

        }

        w_token.end(ui);
    }
}

impl <I: CommandInfo, P: Parser<I>, F: Fn(&Computer) -> u16> CellsTool<I, P, F> {

    pub fn new(page: Rc<RefCell<Memory<I, P>>>, counter_register: F) -> CellsTool<I, P, F> {
        CellsTool {
            counter_register,
            page,
            representation: CellRepresentation::Hex
        }
    }

    fn load_from_file(&mut self, file: String) {

    }

    fn draw_load_from_file(&mut self, ui: &Ui, state: &mut GuiState) {
        let token = ui.begin_menu_bar();
        if token.is_none() { return; }
        let token = token.unwrap();

        if let Some(token) = ui.begin_menu(im_str!("Загрузить"), true) {
            if MenuItem::new(im_str!("Из файла")).build(ui) {
                state.popup_manager.open(PopupChoosingFile::new(|s| self.load_from_file(s)))
            }
            token.end(ui)
        }


        token.end(ui);

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