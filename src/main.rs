#![windows_subsystem = "windows"]
use crate::model::Computer;

mod model;
mod parse;
mod ui;
use core::ops::{BitAnd, Shl};

pub fn bit_at(opcode: u16, pos: u8) -> bool {
    opcode.bitand(1.shl(pos as u16) as u16) != 0
}

fn main() {
    let computer = Computer::new();

    ui::gui::Gui::new(computer).run();
}
