use crate::ui::window::Tool;
use imgui::{Ui, im_str, ImString};
use crate::ui::gui::GuiState;
use crate::bit_at;
use imgui::sys::{igBeginTable, ImGuiTableFlags_None, ImVec2, igNextColumn, igTableNextRow, ImGuiTableRowFlags_None, igTableNextColumn, igEndTable};
use std::os::raw::c_int;

macro_rules! set_bit_at {
    ($opcode:expr, $pos:expr, $v:expr) => {
        {
            use core::ops::*;
            if $v {
                $opcode.bitor(1u16.shl($pos as u16) as u16)
            } else {
                $opcode.bitand(1u16.shl($pos as u16).bitxor(0xFFFF) as u16)
            }
        }
    };
}
pub struct StatusTool;
impl StatusTool {

    pub fn new() -> StatusTool {
        StatusTool {}
    }

}

impl Tool for StatusTool {
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {

        unsafe {
            igBeginTable(im_str!("Status").as_ptr(), 3, ImGuiTableFlags_None as c_int, ImVec2::new(0.0, 0.0), 0.0);
        }
        let mut cnt = 0;

        macro_rules! status_flag {
            ($name:expr, $tooltip:expr, $location:expr) => {
                {
                    unsafe {
                        if cnt % 3 == 0 {
                            igTableNextRow(ImGuiTableRowFlags_None as c_int, 0.0);
                        }
                        igTableNextColumn();

                    }
                    let mut updated = bit_at!(state.computer.registers.r_status, $location);
                    ui.checkbox(ImString::new($name).as_ref(), &mut updated);
                    state.computer.registers.r_status = set_bit_at!(state.computer.registers.r_status, $location, updated);

                    if ui.is_item_hovered() {
                        ui.tooltip_text(format!("{} (бит: {})", $tooltip, $location))
                    }
                    cnt += 1
                }
            };
        }

        status_flag!("Перенос (C)", "Сообщает о переполнении аккамулятора", 0);
        status_flag!("Нуль (Z)", "Сообщает, что в регистре БР хранится 0", 1);
        status_flag!("Знак (N)", "Сообщает, что в регистре БР отрицательное число", 2);
        status_flag!("Постоянный 0", "Для безусловного перехода МПУ использует сравнение этого бита с 0.\nУстановите здесь единицу чтобы все сломать!", 3);
        status_flag!("Разрешение прерывания", "Когда этот бит установлен ВУ могут вызвать прерывание ЭВМ", 4);
        status_flag!("Прерывание", "TODO", 5);
        status_flag!("Состояние ВУ", "TODO", 6);
        status_flag!("Состояние тумблера", "TODO", 7);
        status_flag!("Программа", "TODO", 8);
        status_flag!("Выборка команды", "TODO", 9);
        status_flag!("Выборка адреса", "TODO", 10);
        status_flag!("Исполнение", "TODO", 11);
        status_flag!("Ввод-вывод", "TODO", 12);

        unsafe {
            igEndTable();
        }



    }
}