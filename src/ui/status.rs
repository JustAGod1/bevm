use crate::ui::window::Tool;
use imgui::{Ui, im_str, ImString, Io};
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
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {

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
                        ui.tooltip_text(format!("(бит: {})\n{}", $location, $tooltip))
                    }
                    cnt += 1
                }
            };
        }

        status_flag!("Перенос (C)", "Сообщает о переполнении аккумулятора", 0);
        status_flag!("Прерывание", "Флаг, который говорит о том что было запрошено прерывание\nВ скором времени МПУ должен это обработать\nЭтот флаг невозможно установить, если флаг \"Разрешение прерывания\" не активен.", 5);
        status_flag!("Выборка команды", "Не работает!\n\nФактически этот флажок означает, что МПУ сейчас находится на этапе выбора команды.\nФактически это не нужная фича кмк", 9);

        status_flag!("Нуль (Z)", "Сообщает, что в регистре БР хранится 0", 1);
        status_flag!("Состояние ВУ (Ф)", "Команда TSF устанавливает этот флаг если опрашиваемое ВУ \"готово\".\nЗатем увеличивает СК на один если этот флаг равен 1.\nБольше никак не используется.", 6);
        status_flag!("Выборка адреса", "Не работает!\n\nФактически этот флажок означает, что МПУ сейчас находится на этапе выбора адреса.\nФактически это не нужная фича кмк", 10);

        status_flag!("Знак (N)", "Сообщает, что в регистре БР отрицательное число", 2);
        status_flag!("Состояние тумблера", "Работа - 1\nОстанов - 0\nЕсли \"Работа\", ЭВМ продолжает выполнение как не в себя.\nЕсли \"Останов\", ЭВМ остановится когда закончит выполнять команду.", 7);
        status_flag!("Исполнение", "0 - По циклам\n1 - По тактам\nТактом здесь считается выполнение одной команды МПУ\nЦиклом здесь считается выполнение одной команды не из МКУ", 11);

        status_flag!("Постоянный 0", "Для безусловного перехода МПУ использует сравнение этого бита с 0.\nУстановите здесь единицу чтобы все сломать!", 3);
        status_flag!("Программа", "Не работает!\n\nВ методичке написали что-то типа:\nУстанавливается когда ЭВМ выполняет команды не по тактам, а самостоятельно. В этом режиме работают прерывания.\n\nТак как при таком раскладе становится сложно отлаживать прерывания, в этом эмуляторе ЭВМ этот флаг не учитывается.\nЕще в листинге команд МПУ в методичке нет ни одной команды, которая бы обращалась к этому флагу. Потому смысл совсем теряется.", 8);
        status_flag!("Ввод-вывод", "Этот бит устанавливается в 1, когда выполняется микрокоманда \"Организация связей с ВУ\"(4100)\nПри этом РД должен быть равен РА\nКогда этот бит установлен начинает работать модуль связи с ВУ.\nВ этот момент считается, что регистр РД хранит в себе команду ВУ\nТип команды будет вычислен исходя из содержимого битов 8-11 регистра РД\nСразу после этого флаг будет сброшен.", 12);

        status_flag!("Разрешение прерывания", "Когда этот бит установлен, ВУ могут вызвать прерывание ЭВМ", 4);


        if !state.computer.registers.get_allow_interupt() {
            state.computer.registers.set_interrupt(false);
        }

        if state.computer.registers.get_io() {
            state.computer.process_io_command();
        }

        unsafe {
            igEndTable();
        }



    }
}