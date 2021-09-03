use crate::model::{Computer, Register};
use imgui::{Ui, ChildWindow, TreeNode, im_str, ImString};
use image::flat::Error::TooLarge;
use crate::parse::mc::ExecutionResult;
use crate::ui::gui::{PopupManager, Gui, GuiState};
use crate::ui::popup::PopupHalted;
use crate::ui::window::Tool;

pub struct SmartControlsTool {
    auto_run: bool
}

impl Tool for SmartControlsTool {
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {
        self.draw_control(state, ui);
    }
}

impl SmartControlsTool {
    pub fn new() -> SmartControlsTool {
        SmartControlsTool {
            auto_run: false
        }
    }

    fn draw_control(&mut self, state: &mut GuiState, ui: &Ui) {
        let computer = &mut state.computer;

        if ui.button(im_str!("Микро шаг"), [138.0, 45.0]) {
            computer.registers.set_execute_by_tick(true);
            computer.registers.set_lever(false);
            computer.registers.set_program_mode(false);
            computer.micro_step();
        }

        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 1\nУстанавливает флаг \"Состояние тумблера\" в 0.\nУстанавливается флаг \"Программа\" в 0.\nВыполняется текущая микрокоманда и происходит переход к следующнй.")
        }

        ui.same_line(0.0);

        if ui.button(im_str!("Большой шаг"), [138.0, 45.0]) {
            computer.registers.set_execute_by_tick(false);
            computer.registers.set_lever(false);
            computer.registers.set_program_mode(false);
            while !matches!(computer.micro_step(), ExecutionResult::HALTED) {}
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 0.\nУстанавливается флаг \"Программа\" в 0.\nВыполняется полный цикл микрокоманд.\nГрубо говоря выполняется одна команда.")
        }


        if ui.button(im_str!("Пуск"), [138.0, 45.0]) {
            computer.registers.r_micro_command_counter = 0xA8;
            computer.registers.set_execute_by_tick(false);
            computer.registers.set_lever(true);
            computer.registers.set_program_mode(true);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 1.\nУстанавливается флаг \"Программа\" в 1.\nУстанавливает СчМК в 0A8 то есть сбрасывает состояние регистров ЭВМ\nЭВМ начинает самостоятельно выполнять команду за командой.")
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Продолжить"), [138.0, 45.0]) {
            computer.registers.set_execute_by_tick(false);
            computer.registers.set_lever(true);
            computer.registers.set_program_mode(true);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 1.\nУстанавливается флаг \"Программа\" в 1.\nНе изменяет состояние регистров ЭВМ\nЭВМ начинает самостоятельно выполнять команду за командой.")
        }

        if computer.registers.get_lever() {
            self.auto_run = true;
        }
        if self.auto_run {
            for _ in 0..100 {
                if matches!(computer.micro_step(), ExecutionResult::HALTED) {
                    if computer.registers.get_lever() {
                        state.popup_manager.open(PopupHalted::new());
                    }
                    computer.registers.set_lever(false);
                    self.auto_run = false;
                    break;
                }
            }
        }
    }
}