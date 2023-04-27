use crate::ui::gui::GuiState;

use imgui::{ImString, Io, Ui};

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
        let token = ui.child_window(&self.id)
            .size([0.0, 0.0])
            .movable(false)
            .border(true)
            .menu_bar(true)
            .always_vertical_scrollbar(self.vertical_scroll)
            .begin();
        if token.is_none() {
            return;
        }
        let token = token.unwrap();

        ui.menu_bar(|| {

            let title = self.tools.get(self.tool_selector).unwrap().0;
            if self.tools.len() > 1 {
                ui.menu(title, || {
                    for i in 0..self.tools.len() {
                        let name = ImString::new(self.tools.get(i).unwrap().0);
                        if ui.menu_item_config(name)
                            .selected(i == self.tool_selector)
                            .build()
                        {
                            self.tool_selector = i;
                        }
                    }
                });
            } else {
                ui.text(title);
            }
        });

        let (_, tool) = self.tools.get_mut(self.tool_selector).unwrap();

        tool.draw(ui, io, state);

        token.end();
    }
}

impl WindowTool {
    pub fn single_tool<T>(_width: i32, _height: i32, tool_name: &'static str, tool: T) -> WindowTool
    where
        T: Tool,
        T: 'static,
    {
        Self::new(tool_name.to_string()).append(tool_name, tool)
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

    #[allow(dead_code)]
    pub fn with_vertical_scroll(&mut self) -> &mut WindowTool {
        self.vertical_scroll = true;
        self
    }
}
