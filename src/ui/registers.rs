use crate::ui::window::Tool;
use imgui::{Ui, TreeNode, im_str, ImString};
use crate::ui::gui::GuiState;
use crate::model::{Computer, Register};
use imgui::sys::{igBeginTable, ImGuiTableFlags_BordersH, ImGuiTableFlags_None, ImVec2, igEndTable, igTableNextColumn, igTableNextRow, ImGuiTableRowFlags_None};
use std::os::raw::c_int;

pub struct RegistersTool;

impl RegistersTool {
    pub fn new() -> RegistersTool {
        RegistersTool {}
    }
}

impl Tool for RegistersTool {
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {
        fn reg_field(ui: &Ui, computer: &mut Computer, register: Register, tooltip: &str) {
            let mut content = ImString::from(register.format(computer));
            let t = ui.push_item_width(80.0);
            if ui.input_text(ImString::from(register.mnemonic().to_string()).as_ref(), &mut content)
                .chars_hexadecimal(true)
                .allow_tab_input(false)
                .build()
            {
                if let Ok(parsed) = u32::from_str_radix(content.to_str(), 16) {
                    register.assign_wide(computer, parsed);
                }
            }
            t.pop(ui);
            if ui.is_item_hovered() {
                ui.tooltip_text(tooltip)
            }
        }

        let computer = &mut state.computer;

        unsafe {
            igBeginTable(im_str!("general_reg").as_ptr(), 2, ImGuiTableFlags_None as c_int, ImVec2::new(250.0, 0.0), 0.0);
            igTableNextRow(ImGuiTableRowFlags_None as c_int, 0.0);
            igTableNextColumn();
            reg_field(ui, computer, Register::Counter, "Аккамулятор. Основной регистр с данными.");
            igTableNextColumn();
            reg_field(ui, computer, Register::CommandCounter, "Счетчик команд. Указывает на текущую выполняюмую команду.");
            igEndTable()
        }


        if let Some(token) = TreeNode::new(im_str!("Регистры микрокоманд")).push(ui) {
            unsafe {
                igBeginTable(im_str!("mc_reg").as_ptr(), 2, ImGuiTableFlags_None as c_int, ImVec2::new(250.0, 0.0), 0.0);
                igTableNextRow(ImGuiTableRowFlags_None as c_int, 0.0);
                igTableNextColumn();
                reg_field(ui, computer, Register::McCounter, "Счетчик микрокоманд. Текущая микрокоманда.");
                igTableNextColumn();
                reg_field(ui, computer, Register::Status, "Регистр состояния. В битах этого регистра хранится информация о состоянии ЭВМ.");
                igTableNextRow(ImGuiTableRowFlags_None as c_int, 0.0);
                igTableNextColumn();

                reg_field(ui, computer, Register::MicroCommand, "Регистр микрокоманды. Сюда цпу помещает микрокоманду во время ее выполнения.");
                igTableNextColumn();
                reg_field(ui, computer, Register::Buffer, "Буфферный регистр. Через него проходят данные в микрокомандах.");
                igTableNextRow(ImGuiTableRowFlags_None as c_int, 0.0);
                igTableNextColumn();


                reg_field(ui, computer, Register::Address, "Регистр адреса. Микрокоманда должна поместить адрес сюда, чтобы положить данные в БР");
                igTableNextColumn();
                reg_field(ui, computer, Register::Command, "Регистр команды. Микрокоманда помещает команду из БР сюда.");
                igTableNextRow(ImGuiTableRowFlags_None as c_int, 0.0);
                igTableNextColumn();

                reg_field(ui, computer, Register::Data, "Регистр данных. Сюда микрокоманды помещают данные.");

                igEndTable();
                token.pop(ui);
            }
        } else {
            if ui.is_item_hovered() {
                ui.tooltip_text("... или же \"Не смотри сюда до последней лабы\"")
            }
        }
    }
}