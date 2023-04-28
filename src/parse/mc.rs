use crate::model::{Computer, Register};
use crate::parse::{CommandInfo, Parser};
use crate::utils::bit_registers::*;
use core::ops::*;

use imgui::Ui;

struct RangeDescriptor {
    range: Range<u16>,
    short_description: &'static str,
    explained: &'static str,
}

struct MicroCommandDescriptor {
    global_descriptions: &'static str,
    descriptors: Vec<RangeDescriptor>,
}

impl RangeDescriptor {
    fn value(&self, opcode: u16, into: &mut String) {
        for pos in self.range.clone().rev() {
            if opcode.bitand(1.shl(pos) as u16) != 0 {
                into.push('1')
            } else {
                into.push('0')
            }
        }
    }

    fn new(
        range: Range<u16>,
        short_description: &'static str,
        explained: &'static str,
    ) -> RangeDescriptor {
        RangeDescriptor {
            range,
            short_description,
            explained,
        }
    }
}

impl MicroCommandDescriptor {
    fn new(global_description: &'static str) -> MicroCommandDescriptor {
        MicroCommandDescriptor {
            global_descriptions: global_description,
            descriptors: vec![],
        }
    }

    fn bit(&mut self, bit: u16, short_description: &'static str, explained: &'static str) {
        self.range(bit, bit, short_description, explained)
    }
    fn range(
        &mut self,
        from: u16,
        to: u16,
        short_description: &'static str,
        explained: &'static str,
    ) {
        self.descriptors.push(RangeDescriptor::new(
            to..from + 1,
            short_description,
            explained,
        ))
    }

    fn make_description(&self, ui: &Ui, cmd: &dyn MicroCommand) {
        let opcode = cmd.opcode();

        ui.text_wrapped(self.global_descriptions);

        ui.separator();
        ui.text("Вертикальное представление:");

        let mut vertical = String::new();
        for descriptor in &self.descriptors {
            descriptor.value(opcode, &mut vertical);
            vertical.push(' ')
        }
        ui.text(vertical);
        ui.text("Поля (есть подсказки при наведении):");
        for descriptor in &self.descriptors {
            let mut description_line = String::new();

            descriptor.value(opcode, &mut description_line);
            description_line.push_str(" - ");
            description_line.push_str(descriptor.short_description);

            ui.text(description_line);

            if ui.is_item_hovered() {
                ui.tooltip_text(descriptor.explained)
            }
        }
        ui.separator();
        ui.text("Горизонтальное представление:");

        let horizontal = cmd.horizontal();
        ui.text(format!(
            "Hex: {:0>4X} {:0>4X}",
            horizontal.shr(16),
            horizontal.bitand(0xFFFF)
        ));
        ui.text(format!(
            "Bin: {:0>8b} {:0>8b} {:0>8b} {:0>8b}",
            horizontal.shr(24),
            horizontal.shr(16u32).bitand(0xFF),
            horizontal.shr(8u32).bitand(0xFF),
            horizontal.bitand(0xFF)
        ));
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ExecutionResult {
    Success,
    Jumped,
    Halted,
}

pub trait MicroCommand {
    fn run(&self, computer: &mut Computer) -> ExecutionResult;
    fn mnemonic(&self) -> String;
    fn draw_highlight(&self, ui: &Ui);
    fn opcode(&self) -> u16;
    fn horizontal(&self) -> u32;
}

pub struct OperationalCommand0(u16);

pub struct OperationalCommand1(u16);

pub struct ControlCommand(u16);

pub struct MicroCommandInfo {
    command: Box<dyn MicroCommand>,
}

impl MicroCommandInfo {
    fn new(command: Box<dyn MicroCommand>) -> MicroCommandInfo {
        MicroCommandInfo { command }
    }
}

impl CommandInfo for MicroCommandInfo {
    fn file_string(&self) -> String {
        format!("{:0>4X}", self.command.opcode())
    }

    fn mnemonic(&self) -> String {
        self.command.mnemonic()
    }

    fn draw_highlight(&self, ui: &Ui) {
        self.command.draw_highlight(ui)
    }
}

pub struct McParser;

impl McParser {
    pub fn new() -> McParser {
        McParser {}
    }
}

impl Parser<MicroCommandInfo> for McParser {
    fn parse(&self, opcode: u16) -> MicroCommandInfo {
        MicroCommandInfo::new(parse(opcode))
    }

    fn supports_rev_parse(&self) -> bool {
        false
    }

    fn rev_parse(&self, _: &str) -> Result<u16, String> {
        unimplemented!()
    }
}

pub fn parse(opcode: u16) -> Box<dyn MicroCommand> {
    let sum = sub_sum(opcode, 15, 14);
    match sum {
        0 => Box::new(OperationalCommand0(opcode)),
        1 => Box::new(OperationalCommand1(opcode)),
        _ => Box::new(ControlCommand(opcode)),
    }
}

use std::convert::TryInto;

impl MicroCommand for ControlCommand {
    fn run(&self, computer: &mut Computer) -> ExecutionResult {
        computer.log(
            true,
            format!(
                "Сравнил {} бит из регистра {} с {}",
                self.bit_location(),
                self.register().mnemonic(),
                if self.needed_bit() { 1 } else { 0 }
            ),
        );
        if bit_at(
            self.register().get(computer),
            self.bit_location().try_into().unwrap(),
        ) == self.needed_bit()
        {
            computer.log(
                true,
                format!(
                    "Присвоил значение {:0>4X} регистру СчМК",
                    self.jump_address()
                ),
            );
            computer.registers.r_micro_command_counter = self.jump_address();
            return ExecutionResult::Jumped;
        }

        ExecutionResult::Success
    }
    fn mnemonic(&self) -> String {
        format!(
            "if {}[{}] == {} GOTO {:0>4X}",
            self.register().mnemonic(),
            self.bit_location(),
            if self.needed_bit() { 1 } else { 0 },
            self.jump_address()
        )
    }

    fn draw_highlight(&self, ui: &Ui) {
        let description = "Эта микрокоманда нужна для организации условных переходов в мпу.\n\n\
        Работает все довольно просто:\n\
        1. Берем регистр который указан в поле \"Проверяемый регистр\"\n\
        2. Сравниваем его бит, номер которого записан в поле \"Проверяемый бит\" с битом сравнения.\n\
        3. Если они совпадают, присваиваем значение поля \"Адрес перехода\" регистру СчМК. Иначе делаем \
        ничего
        ";

        let mut descriptor = MicroCommandDescriptor::new(description);

        descriptor.bit(
            15,
            "Код операции",
            "Означает, что эта команда является операционной командой",
        );
        descriptor.bit(
            14,
            "Бит сравнения",
            "Прыжок будет совершен если сравниваемый бит совпадет с этим",
        );

        descriptor.range(
            13,
            12,
            "Проверяемый регистр",
            "Из этого регистра мы возьмем проверяемый бит.\n\
        00 - РС\n\
        01 - РД\n\
        10 - РК\n\
        11 - А
        ",
        );

        descriptor.range(
            11,
            8,
            "Проверяемый бит",
            "Номер бита, который нам нужно сравнить.",
        );

        descriptor.range(7, 0, "Адрес перехода", "В случае когда проверяемый бит совпадет с битом сравнения в СчМК будет присвоено это значение");

        descriptor.make_description(ui, self)
    }

    fn opcode(&self) -> u16 {
        self.0
    }

    fn horizontal(&self) -> u32 {
        // type
        let mut result = 1u32;
        //empty
        result = result.shl(2);

        result = result.shl(1);
        if self.register() == Register::Counter {
            result += 1
        }

        result = result.shl(1);
        if self.register() == Register::Command {
            result += 1
        }

        result = result.shl(1);
        if self.register() == Register::Data {
            result += 1
        }

        result = result.shl(1);
        if self.register() == Register::Status {
            result += 1
        }

        result = result.shl(1);
        result += if self.needed_bit() { 1 } else { 0 };

        result = result.shl(8);
        result += self.jump_address() as u32;

        result = result.shl(16 - self.bit_location());
        result += 1;
        result = result.shl(self.bit_location());

        result
    }
}

fn set_bit(num: &mut u32, pos: u8, value: bool) {
    let value = if value { 1 } else { 0 };
    let bit = value.shl(pos) as u32;
    *num = num.bitand(bit.bitxor(0xFFFF_FFFF));
    *num = num.bitor(bit);
}

impl MicroCommand for OperationalCommand0 {
    fn run(&self, computer: &mut Computer) -> ExecutionResult {
        match self.shift() {
            Shift::Right => {
                let c = computer.registers.get_overflow();

                let overflow = bit_at(computer.registers.r_counter, 0);
                computer.registers.r_buffer = (computer.registers.r_counter as u32).shr(1u32);
                computer.log(true, format!("Присвоил регистру БР значение {:0>4X} из сдвинутого вправо регистра А({:0>4X})", computer.registers.r_buffer, computer.registers.r_counter));
                if c {
                    computer.registers.r_buffer = computer.registers.r_buffer.bitor(0x8000);
                    computer.log(true, "Установил 15 бит регистра БР в 1 так как до начала сдвига был установлен флаг C".to_string());
                }
                if overflow {
                    computer.log(
                        true,
                        "Установил 16 бит регистра БР в 1 т.к. произошло переполнение".to_string(),
                    );
                    computer.registers.r_buffer = computer.registers.r_buffer.bitor(0x10000);
                }
                return ExecutionResult::Success;
            }
            Shift::Left => {
                let c = computer.registers.get_overflow();
                computer.registers.r_buffer = (computer.registers.r_counter as u32)
                    .shl(1u32)
                    .bitand(0x1FFFF);
                computer.log(true, format!("Присвоил регистру БР значение {:0>4X} из сдвинутого влево регистра А({:0>4X})", computer.registers.r_buffer, computer.registers.r_counter));
                if c {
                    computer.registers.r_buffer = computer.registers.r_buffer.bitor(0x1);
                    computer.log(true, "Установил 0 бит регистра БР в 1 так как до начала сдвига был установлен флаг C".to_string());
                }
                return ExecutionResult::Success;
            }
            _ => {}
        }

        match self.memory() {
            Memory::Write => {
                computer
                    .general_memory
                    .borrow_mut()
                    .data
                    .get_mut(computer.registers.r_address.bitand(0x7FF) as usize)
                    .unwrap()
                    .set(computer.registers.r_data);
                computer.log(
                    false,
                    format!(
                        "Присвоил значение {:0>4X} в ячейку {:0>4X}",
                        computer.registers.r_data, computer.registers.r_address
                    ),
                );
            }
            Memory::Read => {
                computer.registers.r_data = computer
                    .general_memory
                    .borrow_mut()
                    .data
                    .get_mut(computer.registers.r_address.bitand(0x7FF) as usize)
                    .unwrap()
                    .get();
                computer.log(
                    false,
                    format!(
                        "Прочитал значение {:0>4X} из ячейки {:0>4X}",
                        computer.registers.r_data, computer.registers.r_address
                    ),
                );
            }
            Memory::None => {}
        };

        let complement = self.complement();
        let left = self
            .left_input()
            .map(|r| {
                if complement == Complement::Left {
                    computer.log(
                        true,
                        format!(
                            "Инвертировал регистр {}({:0>4X})",
                            r.mnemonic(),
                            r.get(computer)
                        ),
                    );
                    r.get(computer).bitxor(0xFFFF)
                } else {
                    r.get(computer)
                }
            })
            .unwrap_or(if complement == Complement::Left {
                0xFFFF
            } else {
                0
            });

        let right = self
            .right_input()
            .map(|r| {
                if complement == Complement::Right {
                    computer.log(
                        true,
                        format!(
                            "Инвертировал регистр {}({:0>4X})",
                            r.mnemonic(),
                            r.get(computer)
                        ),
                    );
                    r.get(computer).bitxor(0xFFFF)
                } else {
                    r.get(computer)
                }
            })
            .unwrap_or(if complement == Complement::Right {
                0xFFFF
            } else {
                0
            });

        match self.operation() {
            Operation::LeftPlusRight => {
                computer.log(
                    true,
                    format!(
                        "Произвел операцию {}({:0>4X} + {:0>4X}) и положил в БР",
                        self.expression_mnemonic(),
                        left,
                        right
                    ),
                );
                computer.registers.r_buffer = (right as u32).wrapping_add(left as u32);
            }
            Operation::LeftPlusRightPlusOne => {
                computer.log(
                    true,
                    format!(
                        "Произвел операцию {}({:0>4X} + {:0>4X} + 1) и положил в БР",
                        self.expression_mnemonic(),
                        left,
                        right
                    ),
                );
                computer.registers.r_buffer =
                    (right as u32).wrapping_add(left.wrapping_add(1) as u32);
            }
            Operation::LeftAndRight => {
                computer.log(
                    true,
                    format!(
                        "Произвел операцию {}({:0>4X} & {:0>4X}) и положил в БР",
                        self.expression_mnemonic(),
                        left,
                        right
                    ),
                );
                computer.registers.r_buffer = (right as u32) & (left as u32);
            }
        };

        ExecutionResult::Success
    }

    fn mnemonic(&self) -> String {
        match self.shift() {
            Shift::Right => return "БР=A >> 1".to_string(),
            Shift::Left => return "БР=A << 1".to_string(),
            _ => {}
        }

        let expression = self.expression_mnemonic();

        let memory = match self.memory() {
            Memory::Write => "*РА = РД",
            Memory::Read => "РД = *РА",
            Memory::None => "",
        };

        format!("{}{}", expression, memory)
    }

    fn draw_highlight(&self, ui: &Ui) {
        let desc = "Операционная команда 0\n\n\
            Ее предназначение - работа с основной памятью, побитовые сдвиги и арифметические действия";

        let mut descriptor = MicroCommandDescriptor::new(desc);

        descriptor.range(
            15,
            14,
            "Код операции",
            "Означает, что эта команда является операционной командой 0",
        );
        descriptor.range(
            13,
            12,
            "Левый вход",
            "Регистр который будет выполнять роль левого операнда.\n\
        00 - 0 - это не регистр. Это просто ноль.\n\
        01 - А\n\
        10 - РС\n\
        11 - КР",
        );
        descriptor.range(
            11,
            10,
            "Пустое место",
            "Это просто бесполезные биты. Не важны что тут будет. Они бесполезны.",
        );
        descriptor.range(
            9,
            8,
            "Правый вход",
            "Регистр, который будет выполнять роль правого операнда.\n\
        00 - 0 - это не регистр. Это просто ноль.\n\
        01 - РД\n\
        10 - РК\n\
        11 - СК",
        );
        descriptor.range(
            7,
            6,
            "Обратный код",
            "От какого операнда мы будем искать обратный код.\n\
        Oбратный код это когда единицы на нули и нули на единицы\n\
        00 - ни от какого\n\
        01 - от левого\n\
        10 - от правого\n\
        11 - ни от какого",
        );
        descriptor.range(
            5,
            4,
            "Операция",
            "Вид операции которую мы применим к операндам:\n\
        00 - Левый + Правый\n\
        01 - Левый + Правый + 1\n\
        10 - Левый & Правый(& - побитовое И)\n\
        11 - Левый + Правый",
        );
        descriptor.range(
            3,
            2,
            "Сдвиг",
            "Это поле - чад. Если мы что-то сдвигаем, то больше ничего не делаем.\n\
        Результат бинарного сдвига попадает в БР\n\
        Сдвигаем мы регистр A\n\
        00 - нет сдвига\n\
        01 - сдвиг вправо\n\
        10 - сдвиг влево\n\
        11 - нет сдвига\n",
        );
        descriptor.range(
            1,
            0,
            "Память",
            "\
        00 - нет обмена\n\
        01 - чтение: возьми из ячейки, адрес которой лежит в РА, основной памяти и положи в РД\n\
        10 - запись: наоборот\n\
        11 - нет обмена",
        );

        descriptor.make_description(ui, self);
    }

    fn opcode(&self) -> u16 {
        self.0
    }

    fn horizontal(&self) -> u32 {
        let mut result = 0;

        for i in 28..=31 {
            set_bit(&mut result, i, false);
        }

        match self.left_input() {
            None => {}
            Some(register) => {
                if register == Register::Counter {
                    set_bit(&mut result, 4, true);
                }
                if register == Register::Status {
                    set_bit(&mut result, 5, true);
                }
                if register == Register::Command {
                    set_bit(&mut result, 6, true);
                }
            }
        }

        match self.right_input() {
            None => {}
            Some(register) => {
                if register == Register::Data {
                    set_bit(&mut result, 1, true);
                }
                if register == Register::CommandCounter {
                    set_bit(&mut result, 3, true);
                }
                if register == Register::Command {
                    set_bit(&mut result, 6, true);
                }
            }
        }

        match self.complement() {
            Complement::Left => {
                set_bit(&mut result, 7, true);
            }
            Complement::Right => {
                set_bit(&mut result, 8, true);
            }
            Complement::None => {}
        }

        match self.operation() {
            Operation::LeftPlusRight => {}
            Operation::LeftPlusRightPlusOne => {
                set_bit(&mut result, 10, true);
            }
            Operation::LeftAndRight => {
                set_bit(&mut result, 9, true);
            }
        }

        match self.shift() {
            Shift::Left => {
                set_bit(&mut result, 12, true);
            }
            Shift::Right => {
                set_bit(&mut result, 11, true);
            }
            Shift::None => {}
        }

        match self.memory() {
            Memory::Read => {
                set_bit(&mut result, 23, true);
            }
            Memory::Write => {
                set_bit(&mut result, 24, true);
            }
            Memory::None => {}
        }

        result
    }
}

impl MicroCommand for OperationalCommand1 {
    fn run(&self, computer: &mut Computer) -> ExecutionResult {
        if self.hlt() {
            computer.log(false, "Оппа, моя остановочка.".to_string());
            return ExecutionResult::Halted;
        }

        let io = self.io();
        if !io.is_empty() {
            for cmd in io {
                match cmd {
                    IOControl::Connect => {
                        if computer.registers.r_data == computer.registers.r_command {
                            computer.log(true, "Установил флаг ВВОД-ВВЫОД и передал управление модулю взаимодействия с ВУ".to_string());
                            computer.registers.set_io(true);
                            computer.process_io_command();
                        } else {
                            computer.log(true, "Было запрошено взаимодействие с ВУ но РД не равен РК. Запрос проигнорирован.".to_string());
                        }
                    }
                    IOControl::DisableInterruption => {
                        computer.log(false, "Запретил прерывания".to_string());
                        computer.registers.set_allow_interrupt(false);
                        computer.registers.set_interrupt(false);
                    }
                    IOControl::EnableInterruption => {
                        computer.log(false, "Разрешил прерывания".to_string());
                        computer.registers.set_allow_interrupt(true);
                    }
                    IOControl::Reset => {
                        computer.log(false, "Сбросил флаги готовности ВУ".to_string());
                        for device in &mut computer.io_devices {
                            device.ready = false;
                        }
                    }
                }
            }

            return ExecutionResult::Success;
        }

        match self.c() {
            CUpdate::Reset => {
                computer.log(false, "Сбросил флаг переноса".to_string());
                computer.registers.set_overflow(false);
            }
            CUpdate::Assign => {
                if computer.registers.r_buffer > 0xFFFF {
                    computer.registers.r_buffer = computer.registers.r_buffer.bitand(0xFFFF);
                    computer.registers.set_overflow(true);
                    computer.log(
                        false,
                        "Установил флаг переноса и убрал лишнюю единицу у БР".to_string(),
                    );
                }
            }
            CUpdate::SetOne => {
                computer.log(false, "Установил флаг переноса".to_string());
                computer.registers.set_overflow(true);
            }
            CUpdate::None => {}
        };

        let nz = self.nz();

        if nz == NZUpdate::Z || nz == NZUpdate::NZ {
            if computer.registers.r_buffer == 0 {
                computer.registers.set_null(true);
                computer.log(false, "Установил флаг \"нуль\"".to_string());
            } else {
                computer.registers.set_null(false);
                computer.log(false, "Убрал флаг \"нуль\"".to_string());
            }
        }
        if nz == NZUpdate::N || nz == NZUpdate::NZ {
            if bit_at(computer.registers.r_buffer as u16, 15) {
                computer.registers.set_negative(true);
                computer.log(false, "Установил флаг \"знак\"".to_string());
            } else {
                computer.registers.set_negative(false);
                computer.log(false, "Убрал флаг \"знак\"".to_string());
            }
        }

        if let Some(v) = self.output() {
            for register in v {
                computer.log(
                    register != Register::Counter && register != Register::CommandCounter,
                    format!(
                        "Перенес значение {:0>4X} из регистра БР в регистр {}",
                        computer.registers.r_buffer,
                        register.mnemonic()
                    ),
                );
                register.assign(computer, computer.registers.r_buffer.bitand(0xFFFF) as u16);
            }
        }

        ExecutionResult::Success
    }

    fn mnemonic(&self) -> String {
        if self.hlt() {
            return "Остановочка.".to_string();
        }

        let io = self
            .io()
            .iter()
            .map(|v| {
                match v {
                    IOControl::DisableInterruption => "Запретить прерывания; ",
                    IOControl::EnableInterruption => "Разрешить прерывания; ",
                    IOControl::Reset => "Сброс флагов ВУ; ",
                    IOControl::Connect => "Организация связей с ВУ; ",
                }
                .to_string()
            })
            .fold("".to_string(), |a, b| format!("{}{}", a, b));

        let c = match self.c() {
            CUpdate::Reset => "C = 0; ",
            CUpdate::Assign => "C = БР[0]; ",
            CUpdate::SetOne => "C = 1; ",
            CUpdate::None => "",
        };

        let nz = match self.nz() {
            NZUpdate::N => "N=БР < 0; ",
            NZUpdate::Z => "Z=БР == 0; ",
            NZUpdate::NZ => "N=БР < 0; Z=БР == 0; ",
            NZUpdate::None => "",
        };

        let updated = self.output().map_or("".to_string(), |vec| {
            vec.iter()
                .map(|r| r.mnemonic())
                .fold("".to_string(), |a, b| format!("{} {}", a, b))
        });

        let updated = if !updated.is_empty() {
            format!("{} = БР; ", updated.trim())
        } else {
            updated.trim().to_string()
        };

        format!("{}{}{}{}", io, c, nz, updated)
    }

    fn draw_highlight(&self, ui: &Ui) {
        let desc = "Операционная команда 1\n\n\
        Эта команда - универсальный боец. В нее пихнули все что не поместилось в другие.\n\
        Но стоит выделить, что если операционная команда 0 изменяет только регистр БР, то эта команда \
        умеет пересылать из БР в какой-нибудь другой регистр. Таким образом эти команды часто работают \
        в паре.
        ";

        let mut descriptor = MicroCommandDescriptor::new(desc);

        descriptor.range(
            15,
            14,
            "Код операции",
            "Означает, что эта команда является операционной командой 1",
        );
        descriptor.range(
            13,
            12,
            "Пустое место",
            "Это просто бесполезные биты. Не важны что тут будет. Они бесполезны.",
        );
        descriptor.bit(
            11,
            "Включить прерывания",
            "Если 1, прерывания будут разрешены.",
        );
        descriptor.bit(10, "Выключить прерывания", "Если 1, прерывания будут запрещены.\nЕсли совместить с предыдущим флагом, не произойдет ничего.");
        descriptor.bit(
            9,
            "Сброс готовности ВУ",
            "Если 1, у всех ВУ будет сброшен флаг готовности.",
        );
        descriptor.bit(
            8,
            "Запуск контролера ВУ",
            "Вот тут начинается черная магия.\n\
        Если 1:\n\
        1. Если РК не равно РД не делаем ничего\n\
        2. Иначе устанавливаем 12 бит регистра РС\n\
        3. Установка этого бита приводит в действие контролер ВУ\n\
        4. Далее читайте описание 12 бита РС",
        );
        descriptor.range(
            7,
            6,
            "Регистр С",
            "Этот бит задает вид взаимодействия с 0 битом регистра РС.\n\
        00 - нет взаимодействия\n\
        01 - если 16 бит регистра БР равен 1, то устанавливаем С в единицу и убираем 16 бит у БР\n\
        10 - устанавливаем С в 0\n\
        11 - устанавливаем С в 1",
        );
        descriptor.bit(
            5,
            "Регистр N",
            "Если 1 и если БР меньше 0, то есть 15 бит равен 1, N будет установлен в 1",
        );
        descriptor.bit(
            4,
            "Регистр Z",
            "Если 1 и если БР равен 0, Z будет установлен в 1",
        );
        descriptor.bit(3, "Остановочка", "Завершает роботу эвм. Чаще всего это говорит о том что команда из основной памяти выполнена.");
        descriptor.range(
            2,
            0,
            "Выход АЛУ",
            "Говорит о том куда пересылать содержимое БР\n\
        000 - никуда\n\
        001 - в РА\n\
        010 - в РД\n\
        011 - в РК\n\
        100 - в СК\n\
        101 - в А\n\
        110 - никуда\n\
        111 - в РА, РД, РК и А",
        );

        descriptor.make_description(ui, self);
    }

    fn opcode(&self) -> u16 {
        self.0
    }

    fn horizontal(&self) -> u32 {
        let mut result = 0;

        set_bit(&mut result, 29, self.io().contains(&IOControl::Connect));
        set_bit(
            &mut result,
            28,
            self.io().contains(&IOControl::EnableInterruption),
        );
        set_bit(
            &mut result,
            27,
            self.io().contains(&IOControl::DisableInterruption),
        );
        set_bit(&mut result, 26, self.io().contains(&IOControl::Reset));

        match self.c() {
            CUpdate::Assign => {
                set_bit(&mut result, 13, true);
            }
            CUpdate::Reset => {
                set_bit(&mut result, 16, true);
            }
            CUpdate::SetOne => {
                set_bit(&mut result, 17, true);
            }
            CUpdate::None => {}
        }

        match self.nz() {
            NZUpdate::N => {
                set_bit(&mut result, 14, true);
            }
            NZUpdate::Z => {
                set_bit(&mut result, 15, true);
            }
            NZUpdate::NZ => {
                set_bit(&mut result, 14, true);
                set_bit(&mut result, 15, true);
            }
            NZUpdate::None => {}
        }

        if self.hlt() {
            set_bit(&mut result, 0, true)
        }

        if let Some(vec) = self.output() {
            for x in vec {
                match x {
                    Register::Address => set_bit(&mut result, 18, true),
                    Register::Command => set_bit(&mut result, 20, true),
                    Register::Data => set_bit(&mut result, 19, true),
                    Register::CommandCounter => set_bit(&mut result, 21, true),
                    Register::Counter => set_bit(&mut result, 22, true),
                    _ => {
                        panic!()
                    }
                }
            }
        }

        result
    }
}

#[derive(Eq, PartialEq)]
enum Shift {
    Left,
    Right,
    None,
}

#[derive(Eq, PartialEq)]
enum Complement {
    Left,
    Right,
    None,
}

#[derive(Eq, PartialEq)]
pub enum Memory {
    Read,
    Write,
    None,
}

#[allow(clippy::enum_variant_names)]
#[derive(Eq, PartialEq)]
enum Operation {
    LeftPlusRight,
    LeftPlusRightPlusOne,
    LeftAndRight,
}

impl OperationalCommand0 {
    pub fn left_input(&self) -> Option<Register> {
        let b13 = bit_at(self.0, 13);
        let b12 = bit_at(self.0, 12);
        if b12 && b13 {
            Some(Register::Command)
        } else if b12 {
            Some(Register::Counter)
        } else if b13 {
            Some(Register::Status)
        } else {
            None
        }
    }

    pub fn right_input(&self) -> Option<Register> {
        let b9 = bit_at(self.0, 9);
        let b8 = bit_at(self.0, 8);
        if b9 && b8 {
            Some(Register::CommandCounter)
        } else if b8 {
            Some(Register::Data)
        } else if b9 {
            Some(Register::Command)
        } else {
            None
        }
    }

    fn shift(&self) -> Shift {
        let b2 = bit_at(self.0, 2);
        let b3 = bit_at(self.0, 3);

        if b2 && b3 {
            // Ну типа два вентеля открыто, выходит ничего не произойдет
            Shift::None
        } else if b2 {
            Shift::Right
        } else if b3 {
            Shift::Left
        } else {
            Shift::None
        }
    }

    pub fn memory(&self) -> Memory {
        let b0 = bit_at(self.0, 0);
        let b1 = bit_at(self.0, 1);

        if b1 && b0 {
            // Ну опять же. В методичке не уточнялось. Старые образцы ничего не делают.
            Memory::None
        } else if b0 {
            Memory::Read
        } else if b1 {
            Memory::Write
        } else {
            Memory::None
        }
    }

    fn operation(&self) -> Operation {
        let b4 = bit_at(self.0, 4);
        let b5 = bit_at(self.0, 5);

        if b4 && b5 {
            // Ну опять же. В методичке не уточнялось. Старые образцы ничего не делают.
            Operation::LeftPlusRight
        } else if b4 {
            Operation::LeftPlusRightPlusOne
        } else if b5 {
            Operation::LeftAndRight
        } else {
            Operation::LeftPlusRight
        }
    }

    fn complement(&self) -> Complement {
        let b6 = bit_at(self.0, 6);
        let b7 = bit_at(self.0, 7);

        if b6 && b7 {
            // Ну опять же. В методичке не уточнялось. Старые образцы ничего не делают.
            Complement::None
        } else if b6 {
            Complement::Left
        } else if b7 {
            Complement::Right
        } else {
            Complement::None
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum NZUpdate {
    N,
    Z,
    NZ,
    None,
}

#[derive(Eq, PartialEq)]
pub enum CUpdate {
    Assign,
    Reset,
    SetOne,
    None,
}

#[derive(Eq, PartialEq)]
pub enum IOControl {
    Connect,
    Reset,
    DisableInterruption,
    EnableInterruption,
}

impl OperationalCommand1 {
    pub fn hlt(&self) -> bool {
        bit_at(self.0, 3)
    }

    pub fn nz(&self) -> NZUpdate {
        let b4 = bit_at(self.0, 4);
        let b5 = bit_at(self.0, 5);

        if b4 && b5 {
            NZUpdate::NZ
        } else if b4 {
            NZUpdate::Z
        } else if b5 {
            NZUpdate::N
        } else {
            NZUpdate::None
        }
    }

    pub fn output(&self) -> Option<Vec<Register>> {
        let s = sub_sum(self.0, 2, 0);

        if s == 6 {
            return None;
        }

        let r = match s {
            0 => vec![],
            1 => vec![Register::Address],
            2 => vec![Register::Data],
            3 => vec![Register::Command],
            4 => vec![Register::CommandCounter],
            5 => vec![Register::Counter],
            7 => vec![
                Register::Address,
                Register::Data,
                Register::Command,
                Register::Counter,
            ],
            _ => {
                panic!()
            }
        };

        Some(r)
    }

    pub fn c(&self) -> CUpdate {
        let b6 = bit_at(self.0, 6);
        let b7 = bit_at(self.0, 7);

        if b6 && b7 {
            CUpdate::SetOne
        } else if b6 {
            CUpdate::Assign
        } else if b7 {
            CUpdate::Reset
        } else {
            CUpdate::None
        }
    }

    pub fn io(&self) -> Vec<IOControl> {
        let mut result = Vec::<IOControl>::new();

        if bit_at(self.0, 8) {
            result.push(IOControl::Connect)
        }
        if bit_at(self.0, 9) {
            result.push(IOControl::Reset)
        }
        if bit_at(self.0, 10) {
            result.push(IOControl::DisableInterruption)
        }
        if bit_at(self.0, 11) {
            result.push(IOControl::EnableInterruption)
        }

        result
    }
}

impl ControlCommand {
    pub fn needed_bit(&self) -> bool {
        bit_at(self.0, 14)
    }

    pub fn bit_location(&self) -> u16 {
        sub_sum(self.0, 11, 8)
    }

    pub fn jump_address(&self) -> u8 {
        sub_sum(self.0, 7, 0) as u8
    }

    pub fn register(&self) -> Register {
        match sub_sum(self.0, 13, 12) {
            0 => Register::Status,
            1 => Register::Data,
            2 => Register::Command,
            3 => Register::Counter,
            _ => panic!(),
        }
    }
}

impl OperationalCommand0 {
    fn expression_mnemonic(&self) -> String {
        let complement = self.complement();

        let left = self
            .left_input()
            .map(|r| {
                if complement == Complement::Left {
                    format!("!{}", r.mnemonic())
                } else {
                    r.mnemonic().into()
                }
            })
            .unwrap_or(if complement == Complement::Left {
                "!0".to_string()
            } else {
                "0".to_string()
            });

        let right = self
            .right_input()
            .map(|r| {
                if complement == Complement::Right {
                    format!("!{}", r.mnemonic())
                } else {
                    r.mnemonic().into()
                }
            })
            .unwrap_or(if complement == Complement::Right {
                "!0".to_string()
            } else {
                "0".to_string()
            });

        match self.operation() {
            Operation::LeftPlusRight => format!("БР={} + {}; ", left, right),
            Operation::LeftPlusRightPlusOne => format!("БР={} + {} + 1; ", left, right),
            Operation::LeftAndRight => format!("БР={} & {}; ", left, right),
        }
    }
}
