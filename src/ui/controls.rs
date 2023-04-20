use crate::model::Registers;

use crate::parse::mc::ExecutionResult;
use crate::ui::gui::GuiState;
use crate::ui::popup::PopupMessage;
use crate::ui::window::Tool;
use imgui::{im_str, Io, MenuItem, Ui};

pub struct SmartControlsTool {
    auto_run: bool,
    history: Vec<HistoryEntry>,
}

struct HistoryEntry {
    registers: Registers,
}

impl Tool for SmartControlsTool {
    fn draw(&mut self, ui: &Ui, _io: &Io, state: &mut GuiState) {
        self.draw_control(state, ui);
    }
}

impl SmartControlsTool {
    pub fn new() -> SmartControlsTool {
        SmartControlsTool {
            auto_run: false,
            history: vec![],
        }
    }

    fn make_history_entry(&mut self, state: &mut GuiState) {
        let entry = HistoryEntry {
            registers: state.computer.registers.clone(),
        };

        self.history.push(entry);
        if self.history.len() > 15 {
            self.history.remove(0usize);
        }
    }

    fn draw_control(&mut self, state: &mut GuiState, ui: &Ui) {
        if let Some(tok) = ui.begin_menu_bar() {
            if MenuItem::new(im_str!("Сброс ЭВМ!")).build(ui) {
                state.computer.reset_memory();
                state.computer.registers = Registers::new()
            }
            tok.end(ui);
        }

        let w = ui.content_region_avail().first().unwrap() / 3.0 - 6.0;
        let h = ui.content_region_avail().get(1).unwrap() / 2.0 - 3.0;

        if ui.button(im_str!("Микро шаг"), [w, h]) {
            self.make_history_entry(state);
            state.computer.registers.set_execute_by_tick(true);
            state.computer.registers.set_lever(false);
            state.computer.registers.set_program_mode(false);
            state.computer.micro_step();
        }

        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 1\nУстанавливает флаг \"Состояние тумблера\" в 0.\nУстанавливается флаг \"Программа\" в 0.\nВыполняется текущая микрокоманда и происходит переход к следующнй.")
        }

        ui.same_line(0.0);

        if ui.button(im_str!("Большой шаг"), [w, h]) {
            self.make_history_entry(state);
            state.computer.registers.set_execute_by_tick(false);
            state.computer.registers.set_lever(false);
            state.computer.registers.set_program_mode(false);
            while !matches!(state.computer.micro_step(), ExecutionResult::Halted) {}
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 0.\nУстанавливается флаг \"Программа\" в 0.\nВыполняется полный цикл микрокоманд.\nГрубо говоря выполняется одна команда.")
        }

        ui.same_line(0.0);

        if ui.button(im_str!("Назад"), [w, h]) && !self.history.is_empty() {
            let entry = self.history.pop().unwrap();
            state.computer.registers = entry.registers;

            self.auto_run = false
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Возвращает регистры к состоянию в котором они были до того как вы нажали последнюю кнопку.")
        }

        if ui.button(im_str!("Пуск"), [w, h]) {
            self.make_history_entry(state);
            state.computer.registers.r_micro_command_counter = 0xA8;
            state.computer.registers.set_execute_by_tick(false);
            state.computer.registers.set_lever(true);
            state.computer.registers.set_program_mode(true);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 1.\nУстанавливается флаг \"Программа\" в 1.\nУстанавливает СчМК в 0A8 то есть сбрасывает состояние регистров ЭВМ\nЭВМ начинает самостоятельно выполнять команду за командой.")
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Продолжить"), [w, h]) {
            self.make_history_entry(state);
            state.computer.registers.set_execute_by_tick(false);
            state.computer.registers.set_lever(true);
            state.computer.registers.set_program_mode(true);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Устанавливает флаг \"Исполнение\" в 0\nУстанавливает флаг \"Состояние тумблера\" в 1.\nУстанавливается флаг \"Программа\" в 1.\nНе изменяет состояние регистров ЭВМ\nЭВМ начинает самостоятельно выполнять команду за командой.")
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Прыжок"), [w, h]) {
            state.jump_requested = true
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Проскроливает к текущей исполняемой команде")
        }

        if state.computer.registers.get_lever() {
            self.auto_run = true;
        }
        if self.auto_run {
            for _ in 0..100 {
                if matches!(state.computer.micro_step(), ExecutionResult::Halted) {
                    if state.computer.registers.get_lever() {
                        state.popup_manager.open(PopupMessage::new(
                            "Остановочка",
                            "ЭВМ завершила свою работу",
                        ));
                    }
                    state.computer.registers.set_lever(false);
                    self.auto_run = false;
                    break;
                }
            }
        }
    }
}
