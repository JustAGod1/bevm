use crate::model::{Computer, Register};
use imgui::{Ui, ChildWindow, TreeNode, im_str, ImString, Io};
use crate::parse::mc::ExecutionResult;
use crate::ui::gui::{PopupManager, Gui, GuiState};
use crate::ui::window::Tool;
use crate::ui::popup::PopupMessage;

pub struct SmartControlsTool {
    auto_run: bool
}

impl Tool for SmartControlsTool {
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
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

        let w = ui.content_region_avail().get(0).unwrap() / 2.0 - 4.0;
        let h = ui.content_region_avail().get(1).unwrap() / 2.0 - 3.0;

        if ui.button(im_str!("Микро шаг"), [w, h]) {
            computer.registers.set_execute_by_tick(true);
            computer.registers.set_lever(false);
            computer.registers.set_program_mode(false);
            computer.micro_step();
        }

        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 1\nУстанавливает флаг \"Состояние тумблера\" в 0.\nУстанавливается флаг \"Программа\" в 0.\nВыполняется текущая микрокоманда и происходит переход к следующнй.")
        }

        ui.same_line(0.0);

        if ui.button(im_str!("Большой шаг"), [w,h]) {
            computer.registers.set_execute_by_tick(false);
            computer.registers.set_lever(false);
            computer.registers.set_program_mode(false);
            while !matches!(computer.micro_step(), ExecutionResult::HALTED) {}
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 0.\nУстанавливается флаг \"Программа\" в 0.\nВыполняется полный цикл микрокоманд.\nГрубо говоря выполняется одна команда.")
        }


        if ui.button(im_str!("Пуск"), [w,h]) {
            computer.registers.r_micro_command_counter = 0xA8;
            computer.registers.set_execute_by_tick(false);
            computer.registers.set_lever(true);
            computer.registers.set_program_mode(true);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 1.\nУстанавливается флаг \"Программа\" в 1.\nУстанавливает СчМК в 0A8 то есть сбрасывает состояние регистров ЭВМ\nЭВМ начинает самостоятельно выполнять команду за командой.")
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Продолжить"), [w,h]) {
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
                        state.popup_manager.open(PopupMessage::new("Остановочка","ЭВМ завершила свою работу"));
                    }
                    computer.registers.set_lever(false);
                    self.auto_run = false;
                    break;
                }
            }
        }
    }
}