use crate::model::Computer;
use imgui::{Ui, im_str, ChildWindow, MenuItem};
use crate::ui::window::Tool;
use crate::ui::gui::PopupManager;

pub struct LogTool {
    show_micro: bool,
    last_size: usize
}

impl LogTool {
    pub fn new() -> LogTool {
        LogTool {
            show_micro: false,
            last_size: 0
        }
    }

}

impl Tool for LogTool {
    fn title(&self) -> String {
        "Лог".to_string()
    }

    fn draw(&mut self, computer: &mut Computer, ui: &Ui, manager: &mut PopupManager) {
        ui.menu_bar(|| {
            if let Some(t) = ui.begin_menu(im_str!("Фильтр"), true) {
                if MenuItem::new(im_str!("Показывать лог микрокоманд"))
                    .selected(self.show_micro)
                    .build(ui) {
                    self.show_micro = !self.show_micro
                }
                t.end(ui);
            }
        });

        let mut last_idx = 0u16;
        for l in computer.logs().iter()
            .filter(|l| !l.micro_command || self.show_micro)
        {
            if last_idx != l.command_counter {
                ui.separator();
                last_idx = l.command_counter;
            }
            ui.text(format!("СК: {:0>3X}, СчМК: {:0>2X}, msg: {}", l.command_counter, l.micro_counter, l.info))
        }
        if self.last_size != computer.logs().len() {
            ui.set_scroll_here_y();
            self.last_size = computer.logs().len();
        }
    }
}