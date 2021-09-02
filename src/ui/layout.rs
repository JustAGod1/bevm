use crate::ui::window::Tool;
use imgui::{Ui, ChildWindow};
use crate::ui::gui::GuiState;
use crate::ui::{relative_width, relative_height};

pub struct LayoutTool {
    id: &'static str,
    vertical: bool,
    width: f32,
    height: f32,
    tools: Vec<Box<dyn Tool>>,
}

impl Tool for LayoutTool {
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {
        ChildWindow::new(self.id)
            .border(false)
            .size([relative_width(self.width, ui), relative_height(self.height, ui)])
            .build(ui, || {
                if self.vertical {
                    self.draw_vertical(ui, state)
                } else {
                    self.draw_horizontal(ui, state)
                }

            });
    }
}

impl LayoutTool {
    pub fn new_horizontal(id: &'static str) -> LayoutTool {
        LayoutTool {
            id,
            width: 0.0,
            height: 0.0,
            vertical: false,
            tools: Vec::<Box<dyn Tool>>::new(),
        }
    }
    pub fn new_vertical(id: &'static str) -> LayoutTool {
        LayoutTool {
            id,
            width: 0.0,
            height: 0.0,
            vertical: true,
            tools: Vec::<Box<dyn Tool>>::new(),
        }
    }

    fn draw_vertical(&mut self, ui: &Ui, state: &mut GuiState) {
        let mut i = 0;
        for tool in &mut self.tools {
            let id_tok = ui.push_id(i);
            tool.draw(ui, state);
            id_tok.pop(ui);
        }
    }
    fn draw_horizontal(&mut self, ui: &Ui, state: &mut GuiState) {
        let mut i: usize = 0;
        let len = self.tools.len();
        for tool in &mut self.tools {
            let id_tok = ui.push_id(i as i32);
            tool.draw(ui, state);
            if i < len - 1 {
                ui.same_line(0.0);
            }
            i += 1;
            id_tok.pop(ui);
        }
    }

    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width as f32;
        self.height = height as f32;

        self
    }

    pub fn append<T>(mut self, tool: T) -> Self
        where T: Tool, T: 'static
    {
        self.tools.push(Box::new(tool));
        return self;
    }
}