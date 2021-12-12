use std::io::Read;
use std::process::Command;
use imgui::{Ui, ImStr};

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
mod help;
mod tracing;

pub fn relative_width(width: f32, ui: &Ui) -> f32 {
    if width > 0.0 { return width; }

    ui.content_region_avail()[0] + width
}

pub fn relative_height(height: f32, ui: &Ui) -> f32 {
    if height > 0.0 { return height; }

    ui.content_region_avail()[1] + height
}

pub fn centralized_text(text: &ImStr, ui: &Ui) {
    let width = *(ui.calc_text_size(text, false, 0.0).get(0)).unwrap();
    //ui.set_cursor_pos([0.0, 0.0]);
    ui.text(text);
}

pub fn open_in_app(str: &str) -> Result<(), String> {

    let mut process = match std::env::consts::OS {
        "linux" => Command::new("sh").arg("-c").arg(format!("xdg-open {}", str)).spawn(),
        "macos" => Command::new("sh").arg("-c").arg(format!("open {}", str)).spawn(),
        "windows" => Command::new("cmd").arg("/c").arg("start").arg(str).spawn(),
        _ => {
            return Err("Операционная система не поддерживается".to_owned());
        }
    };


    match process {
        Ok(ref mut handle) => {

            let status = handle.try_wait().map(|a| a.map(|b| b.code().unwrap_or(1))).unwrap_or(Some(1));
            if let Some(code) = status {
                if code != 0 {
                    fn read_all(src: &mut dyn Read) -> String {
                        let mut buf = Vec::new();
                        let result = src.read_to_end(&mut buf);

                        if result.is_err() { return format!("Failed to read input: {}", result.unwrap_err().to_string()) }

                        String::from_utf8_lossy(buf.as_slice()).to_string()
                    }

                    return Err(format!("{}\n{}", read_all(&mut handle.stderr.take().unwrap()), read_all(&mut handle.stdout.take().unwrap())))
                }
            }
        },
        Err(e) => {
            return Err(e.to_string())
        }
    }
    Ok(())
}