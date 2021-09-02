use crate::model::{Computer, Register};
use imgui::{Ui, ChildWindow, TreeNode, im_str, ImString};
use image::flat::Error::TooLarge;
use crate::parse::mc::ExecutionResult;
use crate::ui::gui::{PopupManager, Gui, GuiState};
use crate::ui::popup::PopupHalted;
use crate::ui::window::Tool;

pub struct ControlsTool;

impl Tool for ControlsTool {

    fn draw(&mut self, ui: &Ui, gui: &mut GuiState) {
        self.draw_control(&mut gui.computer, ui);
    }
}

impl ControlsTool {
    pub fn new() -> ControlsTool {
        ControlsTool {}
    }

    fn draw_control(&mut self, computer: &mut Computer, ui: &Ui) {

        let w_token = ui.push_item_width(300.0);
        if ui.button(im_str!("Микро шаг"), [0.0, 0.0]) {
            computer.micro_step();
        }
        w_token.pop(ui);
        let w_token = ui.push_item_width(120.0);
        if ui.button(im_str!("Большой шаг"), [0.0, 0.0]) {
            while !matches!(computer.micro_step(), ExecutionResult::HALTED) {};
        }
        w_token.pop(ui);

    }

}