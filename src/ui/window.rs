use crate::ui::gui::{PopupManager, Gui, GuiState};
use imgui::{Ui, ChildWindow, im_str, ImString, MenuItem};
use crate::model::Computer;

pub trait Tool {
    fn title(&self) -> String;

    fn draw(&mut self, ui: &Ui, state: &mut GuiState);
}

pub struct ToolsWindow {
    width: f32,
    height: f32,
    id: String,
    tool_selector: usize,
    tools: Vec<Box<dyn Tool>>,

    vertical_scroll: bool,
}


impl ToolsWindow {
    pub fn new<S: Into<String>>(id: S, width: i32, height: i32, tools: Vec<Box<dyn Tool>>) -> ToolsWindow {
        assert!(!tools.is_empty(), "Expected at least one tool");
        ToolsWindow {
            id: id.into(),
            width: width as f32,
            height: height as f32,
            tool_selector: 0,
            tools,
            vertical_scroll: false,
        }
    }


    pub fn with_vertical_scroll(&mut self) -> &mut ToolsWindow {
        self.vertical_scroll = true;
        self
    }

    fn width(&self, ui: &Ui) -> f32 {
        if self.width >= 0.0 { return self.width; }

        ui.content_region_avail()[0] + self.width
    }

    fn height(&self, ui: &Ui) -> f32 {
        if self.height >= 0.0 { return self.height; }

        ui.content_region_avail()[1] + self.height
    }

    pub fn draw(&mut self, ui: &Ui, state: &mut GuiState) {
        let token = ChildWindow::new(&self.id)
            .size([self.width(ui), self.height(ui)])
            .movable(false)
            .menu_bar(true)
            .always_vertical_scrollbar(self.vertical_scroll)
            .begin(ui);
        if token.is_none() { return; }
        let token = token.unwrap();

        ui.menu_bar(|| {
            let title = ImString::from(self.tools.get(self.tool_selector).unwrap().title());
            if self.tools.len() > 1 {
                ui.menu(title.as_ref(), true, || {
                    for i in 0..self.tools.len() {
                        let name = ImString::from(self.tools.get(i).unwrap().title());
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

        let tool = self.tools.get_mut(self.tool_selector).unwrap();

        tool.draw(ui, state);

        token.end(ui);
    }
}