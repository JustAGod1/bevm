use crate::ui::gui::GuiState;
use crate::ui::window::Tool;
use crate::ui::{relative_height, relative_width};
use imgui::{im_str, ChildWindow, Io, Ui};

struct ToolContainer {
    len: f32,
    tool: Box<dyn Tool>,
}

impl ToolContainer {
    fn new(len: f32, tool: impl Tool + 'static) -> ToolContainer {
        ToolContainer {
            len,
            tool: Box::new(tool),
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
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        ChildWindow::new(self.id)
            .border(false)
            .size([0.0, 0.0])
            .build(ui, || {
                if self.vertical {
                    self.draw_vertical(ui, io, state)
                } else {
                    self.draw_horizontal(ui, io, state)
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

    fn draw_tool(
        container: &mut ToolContainer,
        w: f32,
        h: f32,
        ui: &Ui,
        io: &Io,
        state: &mut GuiState,
    ) {
        ChildWindow::new("cont")
            .border(false)
            .size([w, h])
            .build(ui, || container.tool.draw(ui, io, state));
    }

    fn draw_vertical(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        let len = self.tools.len();
        for (i, container) in self.tools.iter_mut().enumerate() {
            let id_tok = ui.push_id(i as i32);
            container.len = relative_height(container.len, ui);
            Self::draw_tool(
                container,
                0.0,
                if i == len - 1 { 0.0 } else { container.len },
                ui,
                io,
                state,
            );
            if i < len - 1 {
                ui.button(im_str!("s"), [relative_width(0.0, ui), 4.0]);
                if ui.is_item_active() {
                    let delta = *io.mouse_delta.get(1).unwrap();
                    container.len += delta;
                    container.len = container.len.max(20.0);
                }
            }
            id_tok.pop(ui);
        }
    }
    fn draw_horizontal(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        let len = self.tools.len();
        for (i, container) in self.tools.iter_mut().enumerate() {
            let id_tok = ui.push_id(i as i32);
            Self::draw_tool(container, container.len, 0.0, ui, io, state);
            if i < len - 1 {
                ui.same_line(0.0);
                ui.button(im_str!("s"), [4.0, relative_height(0.0, ui)]);
                if ui.is_item_active() {
                    let delta = *io.mouse_delta.first().unwrap();
                    container.len += delta;
                    container.len = container.len.max(20.0);
                }
                ui.same_line(0.0);
            }
            id_tok.pop(ui);
        }
    }

    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width as f32;
        self.height = height as f32;

        self
    }

    pub fn append<T>(mut self, len: f32, tool: T) -> Self
    where
        T: Tool,
        T: 'static,
    {
        self.tools.push(ToolContainer::new(len as f32, tool));
        self
    }
}
