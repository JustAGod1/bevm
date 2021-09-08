use imgui::{Ui, ImStr};
use crate::model::Computer;

pub mod gui;

mod cells;
mod log;
mod controls;
mod popup;
mod window;
mod layout;
mod registers;
mod status;
mod io;
mod highlight;
mod load_from_file;
mod help;

pub fn relative_width(width: f32, ui: &Ui) -> f32 {
    if width >= 0.0 { return width; }

    ui.content_region_avail()[0] + width
}

pub fn relative_height(height: f32, ui: &Ui) -> f32 {
    if height >= 0.0 { return height; }

    ui.content_region_avail()[1] + height
}

pub fn centralized_text(text: &ImStr, ui: &Ui) {
    let width = *(ui.calc_text_size(text, false, 0.0).get(0)).unwrap();
    //ui.set_cursor_pos([0.0, 0.0]);
    ui.text(text);
}
