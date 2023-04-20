use crate::parse::general::{GeneralCommandInfo, GeneralParser};
use crate::parse::mc::{parse, ExecutionResult, McParser, MicroCommandInfo};
use crate::parse::{CommandInfo, Parser};
use core::ops::{BitAnd, BitOr, BitXor, Shl};
use std::cell::RefCell;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::SystemTime;

#[derive(Eq, PartialEq)]
pub enum Register {
    McCounter,

    Status,

    MicroCommand,

    Buffer,

    Address,
    Command,
    Data,
    CommandCounter,
    Counter,
}

impl Register {
    pub fn format(&self, computer: &Computer) -> String {
        match self {
            Register::McCounter => format!("{:0>2X}", computer.registers.r_micro_command_counter),
            Register::Buffer => format!("{:0>5X}", computer.registers.r_buffer),
            _ => format!("{:0>4X}", self.get(computer)),
        }
    }

    pub fn mnemonic(&self) -> String {
        match self {
            Register::Status => "РС",
            Register::McCounter => "СчМК",
            Register::Buffer => "БР",
            Register::MicroCommand => "РМК",
            Register::Address => "РА",
            Register::Command => "РК",
            Register::Data => "РД",
            Register::CommandCounter => "СК",
            Register::Counter => "А",
        }
        .to_string()
    }

    pub fn assign_wide(&self, computer: &mut Computer, data: u32) {
        match self {
            Register::Buffer => computer.registers.r_buffer = data.bitand(0x1FFFF),
            _ => self.assign(computer, data as u16),
        }
    }

    pub fn assign(&self, computer: &mut Computer, data: u16) {
        match self {
            Register::Status => computer.registers.r_status = data.bitand(0x1FFF),
            Register::MicroCommand => computer.registers.r_micro_command = data,
            Register::Address => computer.registers.r_address = data.bitand(0x7FF),
            Register::Command => computer.registers.r_command = data,
            Register::Data => computer.registers.r_data = data,
            Register::CommandCounter => computer.registers.r_command_counter = data.bitand(0x7FF),
            Register::Counter => computer.registers.r_counter = data,
            Register::McCounter => computer.registers.r_micro_command_counter = data as u8,
            Register::Buffer => computer.registers.r_buffer = data as u32,
        }
    }

    pub fn get(&self, computer: &Computer) -> u16 {
        match self {
            Register::Status => computer.registers.r_status,
            Register::MicroCommand => computer.registers.r_micro_command,
            Register::Address => computer.registers.r_address,
            Register::Command => computer.registers.r_command,
            Register::Data => computer.registers.r_data,
            Register::CommandCounter => computer.registers.r_command_counter,
            Register::Counter => computer.registers.r_counter,
            Register::McCounter => computer.registers.r_micro_command_counter as u16,
            Register::Buffer => computer.registers.r_buffer as u16,
        }
    }
}

#[derive(Clone)]
pub struct Registers {
    pub r_micro_command_counter: u8, // СчМК. текущая микрокомана
    pub r_status: u16,               // РС - регистр состояния. в разрядах биты статуса

    pub r_micro_command: u16, // РМК. регистр микро команды.
    // type is actually u17
    pub r_buffer: u32,  // БР. буфферный регистр. мк
    pub r_address: u16, // РА - регистр адреса. мк
    pub r_command: u16, // РК - регистр команды. мк
    pub r_data: u16,    // РД - регистр данных. мк

    pub r_command_counter: u16, // СК - счетчик команд. текущая команда эвм
    pub r_counter: u16,         // А
}

macro_rules! status_flag {
    ($pos:expr, $set:ident, $get:ident) => {
        pub fn $set(&mut self, v: bool) {
            if v {
                self.r_status = self.r_status.bitor(1u16.shl($pos as u16) as u16);
            } else {
                self.r_status = self
                    .r_status
                    .bitand(1u16.shl($pos as u16).bitxor(0xFFFF) as u16);
            }
        }
        #[allow(dead_code)]
        pub fn $get(&self) -> bool {
            self.r_status.bitand(1u16.shl($pos)) != 0u16
        }
    };
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            r_micro_command_counter: 0,
            r_status: 0,
            r_command_counter: 0,

            r_micro_command: 0,
            r_buffer: 0,
            r_address: 0,
            r_command: 0,
            r_data: 0,
            r_counter: 0,
        }
    }

    status_flag!(0, set_overflow, get_overflow);
    status_flag!(1, set_null, get_null);
    status_flag!(2, set_negative, get_negative);
    status_flag!(4, set_allow_interrupt, get_allow_interupt);
    status_flag!(5, set_interrupt, get_interupt);
    status_flag!(6, set_io_ready, get_io_ready);
    status_flag!(7, set_lever, get_lever);
    status_flag!(8, set_program_mode, get_program_mode);
    status_flag!(11, set_execute_by_tick, get_execute_by_tick);
    status_flag!(12, set_io, get_io);
}

#[derive(Clone)]
pub struct Memory<I: CommandInfo, P: Parser<I>> {
    pub parser: P,
    pub data: Vec<MemoryCell>,
    pub name: &'static str,
    phantom: PhantomData<I>,
}

#[derive(Clone)]
pub struct MemoryCell {
    data: u16,
    last_touched: SystemTime,
    pub mnemonic: Option<String>,
    pub name: Option<String>,
}

impl MemoryCell {
    pub fn new() -> MemoryCell {
        MemoryCell {
            data: 0,
            last_touched: SystemTime::UNIX_EPOCH,
            mnemonic: None,
            name: None,
        }
    }

    pub fn set(&mut self, data: u16) {
        self.data = data;
        self.last_touched = SystemTime::now();
    }

    pub fn get(&self) -> u16 {
        self.data
    }
}

pub struct LogEntry {
    pub command_counter: u16,
    pub micro_counter: u8,
    pub micro_command: bool,
    pub info: String,
}

#[derive(Copy, Clone)]
pub struct IOCell {
    pub data: u8,
    pub ready: bool,
}

impl IOCell {
    fn new() -> IOCell {
        IOCell {
            data: 0,
            ready: false,
        }
    }
}

pub struct Computer {
    pub registers: Registers,
    pub general_memory: Rc<RefCell<Memory<GeneralCommandInfo, GeneralParser>>>,
    pub mc_memory: Rc<RefCell<Memory<MicroCommandInfo, McParser>>>,
    pub io_devices: [IOCell; 16],
    logs: Vec<LogEntry>,
}

impl Computer {
    fn mem(len: usize) -> Vec<MemoryCell> {
        let mut result = Vec::<MemoryCell>::new();

        for _ in 0..len {
            result.push(MemoryCell::new());
        }

        result
    }

    pub fn process_io_command(&mut self) {
        let opcode = self.registers.r_data;

        let num = opcode.bitand(0xF) as usize;
        if opcode.bitand(0x0300) == 0x0300 {
            let data = self.registers.r_counter.bitand(0xFF) as u8;
            self.log(
                false,
                format!(
                    "Перенес значение {data:0>2X} из младших разрядов аккамулятора в ВУ номер {num}"),
            );
            self.io_devices.get_mut(num).unwrap().data = data;
        } else if opcode.bitand(0x0200) == 0x0200 {
            self.registers.r_counter = self.registers.r_counter.bitand(0xFF00);
            let data = self.io_devices.get_mut(num).unwrap().data as u16;
            self.registers.r_counter = self.registers.r_counter.bitor(data);
            self.log(
                false,
                format!(
                    "Перенес значение {data:0>2X} из  ВУ номер {num} в младшие разряды аккамулятора"
                ),
            );
        } else if opcode.bitand(0x0100) == 0x0100 {
            self.registers
                .set_io_ready(self.io_devices.get(num).unwrap().ready);
            self.log(
                false,
                format!("Опросил ВУ номер {num} на предмет готовности"),
            );

            if self.registers.get_io_ready() {
                self.log(
                    false,
                    format!("ВУ номер {num} оказалось готовым. Увеличил СК на единицу"),
                );
                self.registers.r_command_counter += 1;
            }
        } else {
            self.log(false, format!("Сбросил флаг готовности ВУ номер {num}"));
            self.io_devices[num].ready = false;
        }

        self.log(true, "Сбросил флаг ВВОД-ВВЫОД".to_string());
        self.registers.set_io(false);
    }

    pub fn reset_memory(&mut self) {
        let data = include_bytes!("mc.txt") as &[u8];
        for x in &mut self.mc_memory.borrow_mut().data {
            x.data = 0;
        }
        for x in &mut self.general_memory.borrow_mut().data {
            x.data = 0;
        }
        for line in BufReader::new(data)
            .lines()
            .map(std::result::Result::unwrap)
        {
            let splitted = line.split(' ').collect::<Vec<&str>>();
            let address = u16::from_str_radix(splitted.first().unwrap(), 16).unwrap();
            let value = u16::from_str_radix(splitted.get(1).unwrap(), 16).unwrap();

            self.mc_memory
                .borrow_mut()
                .data
                .get_mut(address as usize)
                .unwrap()
                .set(value);
        }
    }

    pub fn new() -> Computer {
        let mut result = Computer {
            io_devices: [IOCell::new(); 16],
            registers: Registers::new(),
            general_memory: Rc::new(RefCell::new(Memory {
                data: Self::mem(2048),
                parser: GeneralParser::new(),
                name: "general",
                phantom: PhantomData::default(),
            })),
            mc_memory: Rc::new(RefCell::new(Memory {
                data: Self::mem(256),
                parser: McParser::new(),
                name: "mpu",
                phantom: PhantomData::default(),
            })),
            logs: Vec::<LogEntry>::new(),
        };
        result.reset_memory();

        result
    }

    pub fn log(&mut self, micro_command: bool, info: String) {
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
        self.logs.push(LogEntry {
            micro_counter: self.registers.r_micro_command_counter,
            command_counter: self.registers.r_command_counter,
            micro_command,
            info,
        });
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    pub fn logs(&self) -> &Vec<LogEntry> {
        &self.logs
    }

    pub fn micro_step(&mut self) -> ExecutionResult {
        let opcode = self
            .mc_memory
            .borrow_mut()
            .data
            .get(self.registers.r_micro_command_counter as usize)
            .unwrap()
            .get();
        let cmd = parse(opcode);
        self.registers.r_micro_command = opcode;
        let result = cmd.run(self);
        if !matches!(result, ExecutionResult::Jumped) {
            self.registers.r_micro_command_counter =
                self.registers.r_micro_command_counter.wrapping_add(1);
        }
        result
    }
}
