use crate::ui::gui::{PopupManager, Gui, GuiState};
use imgui::{Ui, ChildWindow, im_str, ImString, MenuItem, ImStr, Io};
use crate::model::Computer;
use crate::ui::{relative_width, relative_height};

pub trait Tool {
    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState);
}

pub struct WindowTool {
    id: String,
    tool_selector: usize,
    tools: Vec<(&'static str, Box<dyn Tool>)>,

    vertical_scroll: bool,
}

impl Tool for WindowTool {

    fn draw(&mut self, ui: &Ui, io: &Io, state: &mut GuiState) {
        let token = ChildWindow::new(&self.id)
            .size([0.0,0.0])
            .movable(false)
            .border(true)
            .menu_bar(true)
            .always_vertical_scrollbar(self.vertical_scroll)
            .begin(ui);
        if token.is_none() {
            return;
        }
        let token = token.unwrap();

        ui.menu_bar(|| {

            let title = ImString::new(self.tools.get(self.tool_selector).unwrap().0);
            if self.tools.len() > 1 {
                ui.menu(title.as_ref(), true, || {
                    for i in 0..self.tools.len() {
                        let name = ImString::new(self.tools.get(i).unwrap().0);
                        if MenuItem::new(name.as_ref())
                            .selected(i == self.tool_selector)
                            .build(ui)
                        {
                            self.tool_selector = i;
                        }
                    }
                })
            } else {
                ui.text(title);
            }
        });

        let (_, tool) = self.tools.get_mut(self.tool_selector).unwrap();

        tool.draw(ui, io, state);

        token.end(ui);
    }
}

impl WindowTool {

    pub fn single_tool<T>(width: i32, height: i32, tool_name: &'static str, tool: T) -> WindowTool
        where T: Tool, T: 'static
    {
        Self::new(
            tool_name.to_string(),
        ).append(tool_name, tool)
    }

    pub fn append(mut self, name: &'static str, tool: impl Tool + 'static) -> WindowTool {
        self.tools.push((name, Box::new(tool)));
        self
    }


    pub fn new<S: Into<String>>(id: S) -> WindowTool {
        WindowTool {
            id: id.into(),
            tool_selector: 0,
            tools: vec![],
            vertical_scroll: false,
        }
    }


    pub fn with_vertical_scroll(&mut self) -> &mut WindowTool {
        self.vertical_scroll = true;
        self
    }


}