use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::model::{Computer, Memory};
use std::cell::RefCell;
use crate::parse::mc::McParser;
use std::rc::Rc;

mod ui;
mod model;
mod parse;

#[macro_export]
macro_rules! bit_at {
    ($opcode:expr, $pos:expr) => {
        {
            use core::ops::*;
            $opcode.bitand(1.shl($pos as u16) as u16) != 0
        }
    };
}

fn main() {
    println!("Hello, World!");
    let mut computer = Computer::new();

    read_mc_data(&mut computer);


    ui::gui::Gui::new(computer).run();
}

fn read_mc_data(computer: &mut Computer) {
    let data = include_bytes!("mc.txt") as &[u8];
    for line in BufReader::new(data).lines().map(|r| r.unwrap())
    {
        let splitted = line.split(" ").collect::<Vec<&str>>();
        let address = u16::from_str_radix(splitted.get(0).unwrap(), 16).unwrap();
        let value = u16::from_str_radix(splitted.get(1).unwrap(), 16).unwrap();

        computer.mc_memory.borrow_mut().data.get_mut(address as usize).unwrap().borrow_mut().set(value);
    }
    println!("Read mc commands!");
}