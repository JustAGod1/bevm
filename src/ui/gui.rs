extern crate sdl2;
extern crate imgui;
extern crate imgui_sdl2;
extern crate gl;
extern crate imgui_opengl_renderer;


use sdl2::video::{Window as SDLWindow, WindowPos};
use std::time::Instant;
use imgui::{Window, Ui, im_str, Condition};
use self::imgui::{WindowFlags, ImString, Id, ChildWindow, MenuItem, FontSource, Context, FontConfig, FontGlyphRanges, FontId, TreeNode, StyleColor};
use crate::model::{Computer, Register, MemoryCell};
use crate::ui::cells::CellsTool;
use crate::ui::log::LogTool;
use crate::ui::controls::SmartControlsTool;
use crate::ui::popup::Popup;
use crate::ui::window::{WindowTool, Tool};
use crate::ui::layout::LayoutTool;
use crate::ui::registers::RegistersTool;
use crate::ui::status::StatusTool;
use crate::ui::io::IOTool;
use crate::parse::CommandInfo;
use crate::ui::highlight::CommandHighlightTool;
use crate::ui::load_from_file::LoadFromFileTool;
use crate::ui::help::HelpTool;

pub struct PopupManager {
    popup_delayed: Vec<Box<dyn Popup>>,
}

impl PopupManager {
    fn new() -> PopupManager {
        PopupManager {
            popup_delayed: Vec::<Box<dyn Popup>>::new(),
        }
    }


    pub fn open<P>(&mut self, popup: P) where P: Popup, P: 'static {
        self.popup_delayed.push(Box::new(popup));
    }
}

pub struct GuiState {
    pub last_file_general: Option<String>,
    pub last_file_mc: Option<String>,
    pub computer: Computer,
    pub popup_manager: PopupManager,
    pub current_command: Option<Box<dyn CommandInfo>>,
}

impl GuiState {
    pub fn new(computer: Computer) -> GuiState {
        GuiState {
            last_file_general: None,
            last_file_mc: None,
            computer,
            popup_manager: PopupManager::new(),
            current_command: None,
        }
    }
}

pub struct Gui {
    popup: Option<Box<dyn Popup>>,
    content: LayoutTool,
    state: GuiState,
}

impl Gui {
    pub fn new(computer: Computer) -> Gui {
        return Gui {
            popup: None,
            content:
            LayoutTool::new_vertical("root")
                .append(
                    LayoutTool::new_horizontal("left")
                        .append(
                            WindowTool::new(
                                "mem",
                                500, 0
                            )
                                .append("Основная память", CellsTool::new((&computer.general_memory).clone(), |c| c.registers.r_command_counter))
                                .append("Память МПУ", CellsTool::new((&computer.mc_memory).clone(), |c| c.registers.r_micro_command_counter as u16))
                        )
                        .append(
                            LayoutTool::new_vertical("right")
                                .append(
                                    WindowTool::single_tool(
                                        0, 250,
                                        "Состояние ЭВМ",
                                        LayoutTool::new_horizontal("regandstat")
                                            .append(
                                                WindowTool::single_tool(
                                                    300, 0,
                                                    "Регистры",
                                                    RegistersTool::new(),
                                                )
                                            )
                                            .append(
                                                WindowTool::single_tool(
                                                    0, 0,
                                                    "Разбор регистра статуса (РС)",
                                                    StatusTool::new(),
                                                )
                                            ),
                                    )
                                )
                                .append(
                                    LayoutTool::new_horizontal("middle")
                                        .append(
                                            WindowTool::single_tool(
                                                315, 0,
                                                "Панель управления", LayoutTool::new_vertical("execandio")
                                                    .append(
                                                        WindowTool::single_tool(
                                                            0, 135,
                                                            "Управление исполнением",
                                                            SmartControlsTool::new(),
                                                        )
                                                    )
                                                    .append(
                                                        WindowTool::single_tool(
                                                            0, 0,
                                                            "Внешние устройства",
                                                            IOTool::new(),
                                                        )
                                                    ),
                                            )
                                        )
                                        .append(
                                            LayoutTool::new_vertical("infoandload")
                                                .append(
                                                    WindowTool::single_tool(
                                                        0, -240,
                                                        "Информация о команде",
                                                        CommandHighlightTool::new(),
                                                    )
                                                )
                                                .append(
                                                    WindowTool::single_tool(
                                                        0, 0,
                                                        "Загрузить из файла",
                                                        LoadFromFileTool::new(),
                                                    )
                                                )
                                                .size(315, 0)
                                        )
                                        .append(
                                            WindowTool::new("help", 0, 0)
                                                .append("Прелюдия", HelpTool::new(include_str!("../help/prelude.txt")))
                                                .append("Синтаксис", HelpTool::new(include_str!("../help/file.txt")))
                                                .append("Шпора", HelpTool::new(include_str!("../help/cheatsheet.txt")))
                                                .append("Нотация", HelpTool::new(include_str!("../help/notation.txt")))
                                        )
                                )
                        )
                        .size(0, -210)
                )
                .append(WindowTool::single_tool(
                    0, 200,
                    "Логи", LogTool::new(),
                )
                ),
            state: GuiState::new(computer),
        };
    }

    fn do_open_and_draw(&mut self, ui: &Ui) {
        if self.popup.is_some() {
            if !self.popup.as_mut().unwrap().draw(ui, &mut self.state) {
                self.popup = None;
            }
            return;
        }
        if !self.state.popup_manager.popup_delayed.is_empty() {
            let popup = self.state.popup_manager.popup_delayed.pop().unwrap();
            ui.open_popup(popup.name().as_ref());

            self.popup = Some(popup);
        }
    }

    pub fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();

        {
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 0);
        }

        let mut window = video.window("BasePC 2.0", 1500, 1000)
            .position_centered()
            .resizable()
            .opengl()
            .allow_highdpi()
            .build()
            .unwrap();

        let _gl_context = window.gl_create_context().expect("Couldn't create GL context");
        gl::load_with(|s| video.gl_get_proc_address(s) as _);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let font = self.init_font(&mut imgui);

        let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut last_frame = Instant::now();


        'outer: loop {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            for event in event_pump.poll_iter() {
                match &event {
                    Event::Quit { .. } => {
                        println!("Terminating");
                        break 'outer;
                    }
                    Event::DropFile { filename, .. } => {
                        if self.popup.is_some() {
                            self.popup.as_mut().unwrap().on_file_dropped(filename.as_str())
                        }
                    }
                    _ => {}
                }

                imgui_sdl2.handle_event(&mut imgui, &event);
                if imgui_sdl2.ignore_event(&event) { continue; }
            }


            imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

            let now = Instant::now();
            let delta = now - last_frame;
            let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
            last_frame = now;
            imgui.io_mut().delta_time = delta_s;

            let ui = imgui.frame();

            let token = ui.push_font(font);

            let closed = !self.draw_ui(&ui, &mut window);
            self.do_open_and_draw(&ui);

            token.pop(&ui);

            unsafe {
                gl::ClearColor(0.2, 0.2, 0.2, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            imgui_sdl2.prepare_render(&ui, &window);
            renderer.render(ui);

            window.gl_swap_window();

            ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 40));
            if closed {
                break;
            }
        }
    }

    fn init_font(&self, imgui: &mut Context) -> FontId {
        let font_data = FontSource::TtfData {
            data: include_bytes!("../UbuntuMono-R.ttf"),
            size_pixels: 16.0,
            config: Some(FontConfig {
                name: Some("UbuntuMono".to_string()),
                size_pixels: 16.0,
                glyph_ranges: FontGlyphRanges::cyrillic(),
                ..FontConfig::default()
            }),
        };
        imgui.fonts().add_font(&[font_data])
    }

    fn draw_ui(&mut self, ui: &Ui, sdl_window: &mut SDLWindow) -> bool {
        let mut opened = true;

        let mut window = Window::new(im_str!("Main"))
            .opened(&mut opened);

        window = window.size([sdl_window.size().0 as f32, sdl_window.size().1 as f32], Condition::Always);
        window = window.position([0.0, 0.0], Condition::Appearing);
        window = window.no_decoration();
        window = window.movable(false);


        if let Some(token) = window.begin(&ui) {
            self.content.draw(ui, &mut self.state);
            token.end(ui);
        }

        return opened;
    }
}




