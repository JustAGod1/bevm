use crate::model::{Computer, Register};
use core::ops::*;
use imgui::sys::igBeginChildFrame;
use std::cell::Ref;
use crate::parse::{Parser, CommandInfo};
use crate::bit_at;
use imgui::Ui;


macro_rules! sub_sum {
        ($e:expr, $left:expr ,$right:expr) => {
            {
                let mut sum = 0u16;

                for i in ($right..($left+1)).rev() {
                    sum = sum.shl(1);
                    if bit_at!($e, i) {
                        sum += 1;
                    }
                }

                sum
            }
        };
    }

pub enum ExecutionResult {
    SUCCESS,
    JUMPED,
    HALTED
}

pub trait MicroCommand  {
    fn run(&self, computer: &mut Computer) -> ExecutionResult;
    fn mnemonic(&self) -> String;
}

pub struct OperationalCommand0(u16);

pub struct OperationalCommand1(u16);

pub struct ControlCommand(u16);

pub struct MicroCommandInfo {
    command: Box<dyn MicroCommand>
}

impl MicroCommandInfo {
    fn new(command: Box<dyn MicroCommand>) -> MicroCommandInfo {
        MicroCommandInfo { command }
    }
}

impl CommandInfo for MicroCommandInfo {
    fn mnemonic(&self) -> String {
        self.command.mnemonic()
    }

    fn draw_highlight(&self, ui: &Ui) {
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

    fn rev_parse(&self, str: &str) -> Result<u16, String> {
        panic!()
    }
}

pub fn parse(opcode: u16) -> Box<dyn MicroCommand> {
    let sum = sub_sum!(opcode, 15, 14);
    match sum {
        0 => Box::new(OperationalCommand0(opcode)),
        1 => Box::new(OperationalCommand1(opcode)),
        _ => Box::new(ControlCommand(opcode))
    }
}

impl MicroCommand for ControlCommand {
    fn run(&self, computer: &mut Computer) -> ExecutionResult {
        computer.log(true, format!("Сравнил {} бит из регистра {} с {}",
                                   self.bit_location(),
                                   self.register().mnemonic(),
                                   if self.needed_bit() { 1 } else { 0 }
        ));
        if bit_at!(self.register().get(computer), self.bit_location()) == self.needed_bit() {
            computer.log(true, format!("Присвоил значение {:0>4X} регистру СчМК", self.jump_address()));
            computer.registers.r_micro_command_counter = self.jump_address();
            return ExecutionResult::JUMPED;
        }

        ExecutionResult::SUCCESS
    }
    fn mnemonic(&self) -> String {
        format!("if {}[{}] == {} GOTO {}",
                self.register().mnemonic(),
                self.bit_location(),
                if self.needed_bit() {1} else {0},
                format!("{:0>4X}", self.jump_address())
        )
    }
}

impl MicroCommand for OperationalCommand0 {
    fn run(&self, computer: &mut Computer) -> ExecutionResult{
        match self.shift() {
            Shift::Right => {
                let c = computer.registers.get_overflow();

                let overflow = bit_at!(computer.registers.r_counter, 0);
                computer.registers.r_buffer = (computer.registers.r_counter as u32).shr(1u32);
                computer.log(true, format!("Присвоил регистру БР значение {:0>4X} из сдвинутого вправо регистра А({:0>4X})", computer.registers.r_buffer, computer.registers.r_counter));
                if c {
                    computer.registers.r_buffer = computer.registers.r_buffer.bitor(0x8000);
                    computer.log(true, format!("Установил 15 бит регистра БР в 1 так как до начала сдвига был установлен флаг C"));
                }
                if overflow {
                    computer.log(true, format!("Установил 16 бит регистра БР в 1 т.к. произошло переполнение"));
                    computer.registers.r_buffer = (computer.registers.r_counter as u32).bitor(0x10000);
                    computer.registers.set_overflow(true);
                }
                return ExecutionResult::SUCCESS;
            },
            Shift::Left =>  {
                let c = computer.registers.get_overflow();
                computer.registers.r_buffer = (computer.registers.r_counter as u32).shl(1u32).bitand(0x1FFFF);
                computer.log(true, format!("Присвоил регистру БР значение {:0>4X} из сдвинутого влево регистра А({:0>4X})", computer.registers.r_buffer, computer.registers.r_counter));
                if c {
                    computer.registers.r_buffer = computer.registers.r_buffer.bitor(0x1);
                    computer.log(true, format!("Установил 0 бит регистра БР в 1 так как до начала сдвига был установлен флаг C"));
                }
                return ExecutionResult::SUCCESS;

            }
            _ => {}
        }


        match self.memory() {
            Memory::Write => {
                computer.general_memory
                    .borrow_mut()
                    .data
                    .get_mut(computer.registers.r_address.bitand(0x7FF) as usize)
                    .unwrap()
                    .set(computer.registers.r_data);
                computer.log(false, format!("Присвоил значение {:0>4X} в ячейку {:0>4X}", computer.registers.r_data, computer.registers.r_address));
            },
            Memory::Read => {
                computer.registers.r_data = computer.general_memory
                    .borrow_mut()
                    .data
                    .get_mut(computer.registers.r_address.bitand(0x7FF) as usize)
                    .unwrap()
                    .get();
                computer.log(false, format!("Прочитал значение {:0>4X} из ячейки {:0>4X}", computer.registers.r_data, computer.registers.r_address));

            },
            Memory::None => {}
        };

        if self.operation() == Operation::None { return ExecutionResult::SUCCESS; }

        let complement = self.complement();
        let left = self.left_input()
            .map(
                |r| if complement == Complement::Left
                {
                    computer.log(true, format!("Инвертировал регистр {}({:0>4X})", r.mnemonic(), r.get(computer)));
                    r.get(computer).bitxor(0xFFFF)
                } else { r.get(computer) }
            )
            .unwrap_or(0);

        let right = self.right_input()
            .map(
                |r| if complement == Complement::Right
                {
                    computer.log(true, format!("Инвертировал регистр {}({:0>4X})", r.mnemonic(), r.get(computer)));
                    r.get(computer).bitxor(0xFFFF)
                } else { r.get(computer) }
            )
            .unwrap_or(0);


        match self.operation() {
            Operation::LeftPlusRight => {
                computer.log(true, format!("Произвел операцию {}({:0>4X} + {:0>4X}) и положил в БР", self.expression_mnemonic(), left, right));
                computer.registers.r_buffer = (right as u32).wrapping_add(left as u32);
            }
            Operation::LeftPlusRightPlusOne => {
                computer.log(true, format!("Произвел операцию {}({:0>4X} + {:0>4X} + 1) и положил в БР", self.expression_mnemonic(), left, right));
                computer.registers.r_buffer = (right as u32).wrapping_add(left.wrapping_add(1) as u32);
            }
            Operation::LeftAndRight => {
                computer.log(true, format!("Произвел операцию {}({:0>4X} & {:0>4X}) и положил в БР", self.expression_mnemonic(), left, right));
                computer.registers.r_buffer = (right as u32) & (left as u32);
            },
            Operation::None => panic!()
        };

        ExecutionResult::SUCCESS

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
            Memory::None => ""
        };

        return format!("{}{}", expression, memory)

    }
}


impl MicroCommand for OperationalCommand1 {
    fn run(&self, computer: &mut Computer) -> ExecutionResult {
        if self.hlt() {
            computer.log(false, "Оппа, моя остановочка.".to_string());
            return ExecutionResult::HALTED;
        }

        let io = self.io();
        if io.len() > 0 {
            for cmd in io {
                match cmd {
                    IOControl::Connect => {
                        if computer.registers.r_data == computer.registers.r_address {
                            computer.log(true, "Установил флаг ВВОД-ВВЫОД и передал управление модулю взаимодействия с ВУ".to_string());
                            computer.registers.set_io(true);
                            computer.process_io_command();
                        } else {
                            computer.log(true, "Было запрошенно взаимодействие с ВУ но РД не равен РК. Запрос проигнорирован.".to_string());
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

            return ExecutionResult::SUCCESS;
        }

        match self.c() {
            CUpdate::Reset => {
                computer.log(false, "Сбросил флаг переноса".to_string());
                computer.registers.set_overflow(false);
            },
            CUpdate::Assign => {
                if computer.registers.r_buffer > 0xFFFF {
                    computer.registers.r_buffer = computer.registers.r_buffer.bitand(0xFFFF);
                    computer.registers.set_overflow(true);
                    computer.log(false, "Установил флаг переноса и убрал лишнюю единицу у БР".to_string());
                }
            },
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
            if bit_at!(computer.registers.r_buffer as u16, 15) {
                computer.registers.set_negative(true);
                computer.log(false, "Установил флаг \"знак\"".to_string());
            } else {
                computer.registers.set_negative(false);
                computer.log(false, "Убрал флаг \"знак\"".to_string());

            }
        }

        self.output().map(|v| {
            for register in v {
                computer.log(
                    register != Register::Counter && register != Register::CommandCounter,
                    format!("Перенес значение {:0>4X} из регистра БР в регистр {}", computer.registers.r_buffer, register.mnemonic())
                );
                register.assign(computer, computer.registers.r_buffer.bitand(0xFFFF) as u16);
            }
        });

        ExecutionResult::SUCCESS
    }

    fn mnemonic(&self) -> String {
        if self.hlt() {
            return "Остановочка.".to_string();
        }

        let io = self.io().iter().map(|v| {
            match v {
                IOControl::DisableInterruption => "Запретить прерывания; ",
                IOControl::EnableInterruption => "Разрешить прерывания; ",
                IOControl::Reset => "Сброс флагов ВУ; ",
                IOControl::Connect => "Организация связей с ВУ; "
            }.to_string()
        }).fold("".to_string(), |a, b| { format!("{}{}",a,b) });


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
            NZUpdate::None => ""
        };

        let updated = self.output().map(|vec| {
            vec.iter().map(|r| r.mnemonic())
                .fold("".to_string(), |a,b| format!("{} {}", a, b))
        }).unwrap_or("".to_string());

        let updated = if updated.len() > 0 { format!("{} = БР; ", updated.trim()) } else { updated.trim().to_string() };

        return format!("{}{}{}{}", io, c, nz, updated);

    }
}

#[derive(Eq, PartialEq)]
pub enum Shift {
    Left,
    Right,
    None,
}

#[derive(Eq, PartialEq)]
pub enum Complement {
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

#[derive(Eq, PartialEq)]
pub enum Operation {
    LeftPlusRight,
    LeftPlusRightPlusOne,
    LeftAndRight,
    None,
}

impl OperationalCommand0 {
    pub fn left_input(&self) -> Option<Register> {
        let b13 = bit_at!(self.0, 13);
        let b12 = bit_at!(self.0, 12);
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
        let b9 = bit_at!(self.0, 9);
        let b8 = bit_at!(self.0, 8);
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

    pub fn shift(&self) -> Shift {
        let b2 = bit_at!(self.0, 2);
        let b3 = bit_at!(self.0, 3);

        if b2 && b3 { // Ну типа два вентеля открыто, выходит ничего не произойдет
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
        let b0 = bit_at!(self.0, 0);
        let b1 = bit_at!(self.0, 1);

        if b1 && b0 { // Ну опять же. В методичке не уточнялось. Старые образцы ничего не делают.
            Memory::None
        } else if b0 {
            Memory::Read
        } else if b1 {
            Memory::Write
        } else {
            Memory::None
        }
    }

    pub fn operation(&self) -> Operation {
        let b4 = bit_at!(self.0, 4);
        let b5 = bit_at!(self.0, 5);

        if b4 && b5 { // Ну опять же. В методичке не уточнялось. Старые образцы ничего не делают.
            Operation::None
        } else if b4 {
            Operation::LeftPlusRightPlusOne
        } else if b5 {
            Operation::LeftAndRight
        } else {
            Operation::LeftPlusRight
        }
    }

    pub fn complement(&self) -> Complement {
        let b6 = bit_at!(self.0, 6);
        let b7 = bit_at!(self.0, 7);

        if b6 && b7 { // Ну опять же. В методичке не уточнялось. Старые образцы ничего не делают.
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
        bit_at!(self.0, 3)
    }

    pub fn nz(&self) -> NZUpdate {
        let b4 = bit_at!(self.0, 4);
        let b5 = bit_at!(self.0, 5);

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
        let s = sub_sum!(self.0, 2, 0);

        if s == 6 { return None; }

        let r = match s {
            0 => vec![],
            1 => vec![Register::Address],
            2 => vec![Register::Data],
            3 => vec![Register::Command],
            4 => vec![Register::CommandCounter],
            5 => vec![Register::Counter],
            7 => vec![Register::Address, Register::Data, Register::Command, Register::Counter],
            _ => { panic!() }
        };

        return Some(r);
    }

    pub fn c(&self) -> CUpdate {
        let b6 = bit_at!(self.0, 6);
        let b7 = bit_at!(self.0, 7);

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

        if bit_at!(self.0, 8) {
            result.push(IOControl::Connect)
        }
        if bit_at!(self.0, 9) {
            result.push(IOControl::Reset)
        }
        if bit_at!(self.0, 10) {
            result.push(IOControl::DisableInterruption)
        }
        if bit_at!(self.0, 11) {
            result.push(IOControl::EnableInterruption)
        }

        result
    }
}


impl ControlCommand {
    pub fn needed_bit(&self) -> bool {
        bit_at!(self.0, 14)
    }

    pub fn bit_location(&self) -> u16 {
        sub_sum!(self.0, 11,8)
    }

    pub fn jump_address(&self) -> u8 {
        sub_sum!(self.0, 7, 0) as u8
    }

    pub fn register(&self) -> Register {
        match sub_sum!(self.0, 13, 12) {
            0 => Register::Status,
            1 => Register::Data,
            2 => Register::Command,
            3 => Register::Counter,
            _ => panic!()
        }
    }
}



impl OperationalCommand0 {
    fn expression_mnemonic(&self) -> String {
        let complement = self.complement();


        let left = self.left_input()
            .map(
                |r| if complement == Complement::Left
                { format!("-{}", r.mnemonic()) } else { r.mnemonic() }
            )
            .unwrap_or("0".to_string());

        let right = self.right_input()
            .map(
                |r| if complement == Complement::Right
                { format!("-{}", r.mnemonic()) } else { r.mnemonic() }
            )
            .unwrap_or("0".to_string());

        let expression = match self.operation() {
            Operation::LeftPlusRight => format!("БР={} + {}; ", left, right),
            Operation::LeftPlusRightPlusOne => format!("БР={} + {} + 1; ", left, right),
            Operation::LeftAndRight => format!("БР={} & {}; ", left, right),
            Operation::None => "".to_string()
        };
        expression
    }
}
#[cfg(test)]
mod tests {
    use core::ops::*;
    use crate::bit_at;

    #[test]
    fn sub_sum() {
        assert_eq!(sub_sum!(0xFu16, 3, 0), 0xF);
        assert_eq!(sub_sum!(0xFu16, 2, 0), 0x7);
        assert_eq!(sub_sum!(0xCu16, 3, 0), 0xC);
        assert_eq!(sub_sum!(0xCu16, 2, 0), 0x4);
        assert_eq!(sub_sum!(0xAu16, 3, 0), 0xA);
        assert_eq!(sub_sum!(0xAu16, 2, 0), 0x2);
    }
}
