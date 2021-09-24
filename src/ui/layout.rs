use crate::ui::window::Tool;
use imgui::{Ui, ChildWindow, im_str};
use crate::ui::gui::GuiState;
use crate::ui::{relative_width, relative_height};

struct ToolContainer {
    len: f32,
    tool: Box<dyn Tool>
}

impl ToolContainer {
    fn new(len: f32, tool: impl Tool + 'static) -> ToolContainer {
        ToolContainer {
            len,
            tool: Box::new(tool)
        }
    }
}

pub struct LayoutTool {
    id: &'static str,
    vertical: bool,
    width: f32,
    height: f32,
    tools: Vec<ToolContainer>,
}

impl Tool for LayoutTool {
    fn draw(&mut self, ui: &Ui, state: &mut GuiState) {

        ChildWindow::new(self.id)
            .border(false)
            .size([0.0,0.0])
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
            tools: Vec::new(),
        }
    }
    pub fn new_vertical(id: &'static str) -> LayoutTool {
        LayoutTool {
            id,
            width: 0.0,
            height: 0.0,
            vertical: true,
            tools: Vec::new(),
        }
    }

    fn draw_tool(container: &mut ToolContainer, w: f32, h: f32, ui: &Ui, state: &mut GuiState) {
        ChildWindow::new("cont")
            .border(false)
            .size([w,h])
            .build(ui, || {
                container.tool.draw(ui, state)
            });

    }

    fn draw_vertical(&mut self, ui: &Ui, state: &mut GuiState) {
        let mut i = 0usize;
        let len = self.tools.len();
        for container in &mut self.tools {
            let id_tok = ui.push_id(i as i32);
            container.len = relative_height(container.len, ui);
            Self::draw_tool(container, 0.0, container.len, ui, state);
            if i < len - 1 && ui.button(im_str!("s"), [relative_width(0.0, ui), 2.0]) {
                container.len+=1.0;
            }
            id_tok.pop(ui);
            i+=1
        }
    }
    fn draw_horizontal(&mut self, ui: &Ui, state: &mut GuiState) {
        let mut i: usize = 0;
        let len = self.tools.len();
        for container in &mut self.tools {
            let id_tok = ui.push_id(i as i32);
            Self::draw_tool(container, container.len, 0.0, ui, state);
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

    pub fn append<T>(mut self, len: i32, tool: T) -> Self
        where T: Tool, T: 'static
    {
        self.tools.push(ToolContainer::new(len as f32, tool));
        return self;
    }
}