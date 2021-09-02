use crate::ui::window::Tool;
use imgui::{Ui, TreeNode, im_str, ImString};
use crate::ui::gui::GuiState;
use crate::model::{Computer, Register};

pub struct RegistersTool;

impl RegistersTool {

    pub fn new() -> RegistersTool {
        RegistersTool {}
    }

}

impl Tool for RegistersTool {
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {
        fn reg_field(ui: &Ui, computer: &mut Computer, register: Register, name: &str, tooltip: &str) {
            let mut content = ImString::from(format!("{:0>4X}", register.get(computer)));
            let t = ui.push_item_width(80.0);
            if ui.input_text(ImString::from(name.to_string()).as_ref(), &mut content)
                .chars_hexadecimal(true)
                .allow_tab_input(false)
                .build()
            {
                if let Ok(parsed) = u16::from_str_radix(content.to_str(), 16) {
                    register.set(computer, parsed);
                }
            }
            t.pop(ui);
            if ui.is_item_hovered() {
                ui.tooltip_text(tooltip)
            }
        }

        let computer = &mut state.computer;

        reg_field(ui, computer, Register::Counter, "А", "Аккамулятор. Основной регистр с данными.");
        ui.same_line(0.0);
        reg_field(ui, computer, Register::CommandCounter, "СК", "Счетчик команд. Указывает на текущую выполняюмую команду.");


        if let Some(token) = TreeNode::new(im_str!("Регистры микрокоманд")).push(ui) {
            let mut content = ImString::from(format!("{:0>2X}", computer.registers.r_micro_command_counter));
            let t = ui.push_item_width(80.0);
            if ui.input_text(im_str!("СчМК"), &mut content)
                .chars_hexadecimal(true)
                .allow_tab_input(false)
                .build()
            {
                if let Ok(parsed) = u8::from_str_radix(content.to_str(), 16) {
                    computer.registers.r_micro_command_counter = parsed
                }
            }
            t.pop(ui);
            if ui.is_item_hovered() {
                ui.tooltip_text("Счетчик микрокоманд. Текущая микрокоманда.")
            }
            ui.same_line(0.0);
            reg_field(ui, computer, Register::Status, "РС", "Регистр состояния. В битах этого регистра хранится информация о состоянии ЭВМ.");

            reg_field(ui, computer, Register::MicroCommand, "РМК", "Регистр микрокоманды. Сюда цпу помещает микрокоманду во время ее выполнения.");
            ui.same_line(0.0);
            let mut content = ImString::from(format!("{:0>2X}", computer.registers.r_buffer));
            let t = ui.push_item_width(80.0);
            if ui.input_text(im_str!("БР"), &mut content)
                .chars_hexadecimal(true)
                .allow_tab_input(false)
                .build()
            {
                if let Ok(parsed) = u32::from_str_radix(content.to_str(), 16) {
                    let parsed = if parsed > 0x1FFFF { 0xFFFF } else { parsed };
                    computer.registers.r_buffer = parsed
                }
            }
            t.pop(ui);
            if ui.is_item_hovered() {
                ui.tooltip_text("Буфферный регистр. Через него проходят данные в микрокомандах.")
            }


            reg_field(ui, computer, Register::Address, "РА", "Регистр адреса. Микрокоманда должна поместить адрес сюда, чтобы положить данные в БР");
            ui.same_line(0.0);
            reg_field(ui, computer, Register::Command, "РК", "Регистр команды. Микрокоманда помещает команду из БР сюда.");

            reg_field(ui, computer, Register::Data, "РД", "Регистр данных. Сюда микрокоманды помещают данные.");

            token.pop(ui);
        } else {
            if ui.is_item_hovered() {
                ui.tooltip_text("... или же \"Не смотри сюда до последней лабы\"")
            }
        }
    }
}