use imgui::Ui;
use crate::model::Computer;

pub mod gui;

mod cells;
mod log;
mod controls;
mod popup;
mod window;
mod layout;
mod registers;

pub fn relative_width(width: f32, ui: &Ui) -> f32 {
    if width >= 0.0 { return width; }

    ui.content_region_avail()[0] + width
}

pub fn relative_height(height: f32, ui: &Ui) -> f32 {
    if height >= 0.0 { return height; }

    ui.content_region_avail()[1] + height
}
