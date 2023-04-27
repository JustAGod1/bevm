use crate::ui::gui::GuiState;
use crate::ui::window::Tool;
use imgui::{Io, StyleColor, Ui};

pub struct LogTool {
    show_micro: bool,
    last_size: usize,
}

impl LogTool {
    pub fn new() -> LogTool {
        LogTool {
            show_micro: false,
            last_size: 0,
        }
    }
}

impl Tool for LogTool {
    fn draw(&mut self, ui: &Ui, _io: &Io, gui: &mut GuiState) {
        ui.menu_bar(|| {
            if let Some(t) = ui.begin_menu("Фильтр") {
                if ui.menu_item_config("Показывать лог микрокоманд")
                    .selected(self.show_micro)
                    .build()
                {
                    self.show_micro = !self.show_micro;
                }
                t.end();
            }
            let token = ui.push_style_color(StyleColor::Button, [0.0,  0.0,  0.0,  0.0]);
            if ui.button("Очистить") {
                gui.computer.clear_logs();
            }
            token.pop();
        });

        let mut last_idx = 0u16;
        for l in gui
            .computer
            .logs()
            .iter()
            .filter(|l| !l.micro_command || self.show_micro)
        {
            if last_idx != l.command_counter {
                ui.separator();
                last_idx = l.command_counter;
            }
            ui.text(format!(
                "СК: {:0>3X}, СчМК: {:0>2X}, msg: {}",
                l.command_counter, l.micro_counter, l.info
            ));
        }
        if self.last_size != gui.computer.logs().len() {
            ui.set_scroll_here_y();
            self.last_size = gui.computer.logs().len();
        }
    }
}
