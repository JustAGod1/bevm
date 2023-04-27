extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

use std::time::Instant;

use imgui::{Condition, Ui};
use sdl2::video::Window as SDLWindow;

use crate::model::Computer;
use crate::parse::CommandInfo;
use crate::ui::cells::CellsTool;
use crate::ui::controls::SmartControlsTool;
use crate::ui::help::HelpTool;
use crate::ui::highlight::CommandHighlightTool;
use crate::ui::io::IOTool;
use crate::ui::layout::LayoutTool;
use crate::ui::log::LogTool;
use crate::ui::popup::Popup;
use crate::ui::registers::RegistersTool;
use crate::ui::status::StatusTool;
use crate::ui::window::{Tool, WindowTool};

use self::imgui::sys::ImGuiKey_Backspace;
use self::imgui::{Context, FontConfig, FontGlyphRanges, FontId, FontSource, Io};
use self::sdl2::keyboard::Scancode;

use crate::ui::tracing::TraceTool;

pub struct PopupManager {
    popup_delayed: Vec<Box<dyn Popup>>,
}

impl PopupManager {
    fn new() -> PopupManager {
        PopupManager {
            popup_delayed: Vec::<Box<dyn Popup>>::new(),
        }
    }

    pub fn open<P>(&mut self, popup: P)
    where
        P: Popup,
        P: 'static,
    {
        self.popup_delayed.push(Box::new(popup));
    }
}

pub struct GuiState {
    pub last_file_general: Option<String>,
    pub last_file_mc: Option<String>,
    pub computer: Computer,
    pub editor_enabled: bool,
    pub theme_requested: Option<Theme>,
    pub popup_manager: PopupManager,
    pub current_command: Option<Box<dyn CommandInfo>>,
    pub jump_requested: bool,
}

impl GuiState {
    pub fn new(computer: Computer) -> GuiState {
        GuiState {
            editor_enabled: false,
            theme_requested: None,
            last_file_general: None,
            last_file_mc: None,
            computer,
            popup_manager: PopupManager::new(),
            current_command: None,
            jump_requested: false,
        }
    }
}

pub struct Gui {
    popup: Option<Box<dyn Popup>>,
    content: LayoutTool,
    state: GuiState,
}

pub enum Theme {
    Dark,
    Light,
    Classic,
}

impl Gui {
    pub fn new(computer: Computer) -> Gui {
        Gui {
            popup: None,
            content: LayoutTool::new_vertical("root")
                .append(
                    -210.,
                    LayoutTool::new_horizontal("main")
                        .append(
                            250.,
                            WindowTool::new("mem")
                                .append(
                                    "Основная память",
                                    CellsTool::new(computer.general_memory.clone(), |c| {
                                        c.registers.r_command_counter
                                    }),
                                )
                                .append(
                                    "Память МПУ",
                                    CellsTool::new(computer.mc_memory.clone(), |c| {
                                        c.registers.r_micro_command_counter as u16
                                    }),
                                ),
                        )
                        .append(
                            0.,
                            LayoutTool::new_vertical("right")
                                .append(
                                    250.,
                                    WindowTool::single_tool(
                                        0,
                                        250,
                                        "Состояние ЭВМ",
                                        LayoutTool::new_horizontal("regandstat")
                                            .append(
                                                300.,
                                                WindowTool::single_tool(
                                                    300,
                                                    0,
                                                    "Регистры",
                                                    RegistersTool::new(),
                                                ),
                                            )
                                            .append(
                                                0.,
                                                WindowTool::single_tool(
                                                    0,
                                                    0,
                                                    "Разбор регистра статуса (РС)",
                                                    StatusTool::new(),
                                                ),
                                            ),
                                    ),
                                )
                                .append(
                                    0.,
                                    LayoutTool::new_horizontal("middle")
                                        .append(
                                            335.,
                                            WindowTool::single_tool(
                                                315,
                                                0,
                                                "Панель управления",
                                                LayoutTool::new_vertical("execandio")
                                                    .append(
                                                        135.,
                                                        WindowTool::single_tool(
                                                            0,
                                                            135,
                                                            "Управление исполнением",
                                                            SmartControlsTool::new(),
                                                        ),
                                                    )
                                                    .append(
                                                        0.,
                                                        WindowTool::single_tool(
                                                            0,
                                                            0,
                                                            "Внешние устройства",
                                                            IOTool::new(),
                                                        ),
                                                    ),
                                            )
                                            .append("Таблица трассировки", TraceTool::new()),
                                        )
                                        .append(
                                            350.,
                                            LayoutTool::new_vertical("infoandload")
                                                .append(
                                                    0.,
                                                    WindowTool::single_tool(
                                                        0,
                                                        0,
                                                        "Информация о команде",
                                                        CommandHighlightTool::new(),
                                                    ),
                                                )
                                                .size(315, 0),
                                        )
                                        .append(
                                            0.,
                                            WindowTool::new("help")
                                                .append(
                                                    "Прелюдия",
                                                    HelpTool::new(include_str!(
                                                        "../help/prelude.txt"
                                                    )),
                                                )
                                                .append(
                                                    "Синтаксис",
                                                    HelpTool::new(include_str!("../help/file.txt")),
                                                )
                                                .append(
                                                    "Шпора",
                                                    HelpTool::new(include_str!(
                                                        "../help/cheatsheet.txt"
                                                    )),
                                                )
                                                .append(
                                                    "Нотация",
                                                    HelpTool::new(include_str!(
                                                        "../help/notation.txt"
                                                    )),
                                                )
                                                .append(
                                                    "Да как остановить епт",
                                                    HelpTool::new(include_str!(
                                                        "../help/run_and_stop.txt"
                                                    )),
                                                ),
                                        ),
                                ),
                        ),
                )
                .append(
                    200.,
                    WindowTool::new("bottom").append("Логи", LogTool::new()),
                ),
            state: GuiState::new(computer),
        }
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
            ui.open_popup(popup.name());

            self.popup = Some(popup);
        }
    }

    pub fn run(&mut self) {
        let sdl_context = sdl2::init().expect("Expect to get sdl_context");
        let video = sdl_context
            .video()
            .expect("Expect to initialize video system");

        {
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 0);
        }

        let mut window = video
            .window("BasePC 2.0", 1500, 1000)
            .position_centered()
            .resizable()
            .opengl()
            .allow_highdpi()
            .build()
            .unwrap();

        let _gl_context = window
            .gl_create_context()
            .expect("Couldn't create GL context");
        gl::load_with(|s| video.gl_get_proc_address(s) as _);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let font = Self::init_font(&mut imgui);

        imgui.io_mut().key_map[ImGuiKey_Backspace as usize] = Scancode::Backspace as u32;
        imgui.style_mut().use_classic_colors();

        let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

        let renderer =
            imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut last_frame = Instant::now();

        'outer: loop {
            use sdl2::event::Event;

            for event in event_pump.poll_iter() {
                if event.is_keyboard() {}

                imgui_sdl2.handle_event(&mut imgui, &event);

                match &event {
                    Event::Quit { .. } => {
                        println!("Terminating");
                        break 'outer;
                    }
                    Event::DropFile { filename, .. } => {
                        if self.popup.is_some() {
                            self.popup
                                .as_mut()
                                .unwrap()
                                .on_file_dropped(filename.as_str())
                        }
                    }
                    Event::KeyDown { scancode: _, .. } | Event::KeyUp { scancode: _, .. } => {
                        break;
                    }
                    _ => {}
                }
            }

            if let Some(theme) = &self.state.theme_requested {
                match theme {
                    Theme::Dark => imgui.style_mut().use_dark_colors(),
                    Theme::Light => imgui.style_mut().use_light_colors(),
                    Theme::Classic => imgui.style_mut().use_classic_colors(),
                };
                self.state.theme_requested = None;
            };

            imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

            let now = Instant::now();
            let delta = (now - last_frame).as_secs_f32();
            last_frame = now;
            imgui.io_mut().delta_time = delta;

            let io = unsafe { &mut *(imgui.io_mut() as *mut Io) };
            let ui = imgui.frame();

            let token = ui.push_font(font);

            let closed = !self.draw_ui(ui, io, &mut window);
            self.do_open_and_draw(ui);

            token.pop();

            unsafe {
                gl::ClearColor(0.2, 0.2, 0.2, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            imgui_sdl2.prepare_render(ui, &window);
            renderer.render(&mut imgui);

            window.gl_swap_window();

            ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 40));
            if closed {
                break;
            }
        }
    }

    fn init_font(imgui: &mut Context) -> FontId {
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

    fn draw_ui(&mut self, ui: &Ui, io: &Io, sdl_window: &mut SDLWindow) -> bool {
        let mut opened = true;

        let mut window = ui.window("Main").opened(&mut opened);

        window = window.size(
            [sdl_window.size().0 as f32, sdl_window.size().1 as f32],
            Condition::Always,
        );
        window = window.no_decoration();
        window = window.movable(false);
        if self.state.editor_enabled {
            let mut style = ui.clone_style();
            ui.window("Editor")
                .build(|| ui.show_style_editor(&mut style));
        }

        window = window.position([0.0, 0.0], Condition::Appearing);

        if let Some(token) = window.begin() {
            self.content.draw(ui, io, &mut self.state);
            token.end();
        }

        opened
    }
}
