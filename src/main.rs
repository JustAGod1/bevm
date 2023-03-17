#![windows_subsystem = "windows"]
use crate::model::Computer;

mod model;
mod parse;
mod ui;
mod utils;

fn main() {
    let computer = Computer::new();

    ui::gui::Gui::new(computer).run();
}
