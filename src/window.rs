use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::{thread, time};

use syntect::highlighting;

use crate::buffer::Buffer;
use crate::commands;
use crate::keybinding;
use crate::keybinding::KeyBinding;
use crate::system::Canvas;
use crate::view::{Direction, View, Indentation};
use nanovg::Color;

use crate::styling::STYLE;

#[derive(Debug, Clone, Copy, Default)]
pub struct Geometry {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct StatusBar {
    geometry: Geometry,
}

impl StatusBar {
    pub fn new() -> Self {
        StatusBar {
            geometry: Default::default(),
        }
    }
    pub fn relayout(&mut self, geometry: Geometry, _canvas: &Canvas) {
        self.geometry = geometry;
    }
    pub fn min_size(&self, canvas: &Canvas) -> (f32, f32) {
        (10.0, canvas.fonts["mono"].line_height)
    }
    pub fn draw(
        &self,
        canvas: &mut Canvas,
        line: usize,
        col: usize,
        filename: &str,
        encoding: &str,
        syntax: &str,
        is_dirty: bool,
        indentation: Indentation,
    ) {
        let bg_color = STYLE.theme.settings.foreground.unwrap_or(highlighting::Color::WHITE);
        let fg_color = STYLE.theme.settings.background.unwrap_or(highlighting::Color::BLACK);
        canvas.set_color(Color::from_rgb(bg_color.r, bg_color.g, bg_color.b));

        canvas.move_to(self.geometry.x, self.geometry.y);
        canvas.draw_rect(self.geometry.w, self.geometry.h);
        canvas.set_color(Color::from_rgb(fg_color.r, fg_color.g, fg_color.b));

        canvas.move_to(self.geometry.x, self.geometry.y + self.geometry.h + canvas.fonts["mono"].descender);

        canvas.draw_str(
            &format! {"{}{} | {} | {} | {} | ({},{})",filename,if is_dirty {"*"} else {""}, syntax, encoding, indentation, line, col},
        );
    }
}

pub struct EditorWindow<'v> {
    views: Vec<View<'v>>,
    buffers: Vec<Rc<RefCell<Buffer>>>,
    geometry: Geometry,
    current_view: usize,
    statusbar: StatusBar,
}

pub trait WindowCmd {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn keybinding(&self) -> Vec<KeyBinding>;
    fn run(&mut self, _: &mut EditorWindow<'_>);
}

const FONT_SIZE: f32 = 14.0;

impl<'v> EditorWindow<'v> {
    pub fn new<P: AsRef<Path>>(file: Option<P>) -> Self {
        let mut w = EditorWindow::init();
        w.add_new_view(file);
        w
    }
    fn init() -> Self {
        let views = Vec::new();
        let buffers = Vec::new();
        let statusbar = StatusBar::new();
        EditorWindow {
            views,
            buffers,
            geometry: Default::default(),
            current_view: 0,
            statusbar,
        }
    }

    pub fn get_current_view(&self) -> &View<'_> {
        &self.views[self.current_view]
    }
    pub fn get_current_view_mut(&mut self) -> &'v mut View<'_> {
        &mut self.views[self.current_view]
    }

    pub fn add_new_view<P: AsRef<Path>>(&mut self, file: Option<P>) {
        let b = match file {
            None => Rc::new(RefCell::new(Buffer::new())),
            Some(file) => Rc::new(RefCell::new(Buffer::from_file(file.as_ref()).expect("File not found"))),
        };
        self.buffers.push(b.clone());
        let v = View::new(b.clone());

        let viewid = self.views.len();
        self.views.push(v);
        self.current_view = viewid;
    }

    fn relayout(&mut self, geometry: Geometry, canvas: &Canvas) {
        self.geometry = geometry;

        let status_height = self.statusbar.min_size(canvas).1;
        self.statusbar.relayout(
            Geometry {
                x: 0.0,
                y: self.geometry.h - status_height,
                w: self.geometry.w,
                h: status_height,
            },
            canvas,
        );
        for i in 0..self.views.len() {
            let g = Geometry {
                x: 0.0,
                y: 0.0,
                w: self.geometry.w,
                h: self.geometry.h - status_height,
            };
            self.views[i].relayout(g, canvas);
        }
    }
    fn draw(&mut self, canvas: &mut Canvas) {
        let v = self.get_current_view();
        v.draw(canvas);
        self.statusbar.draw(
            canvas,
            v.line_idx(),
            v.col_idx(),
            v.get_name(),
            v.get_encoding(),
            v.get_syntax(),
            v.is_dirty(),
            v.get_indentation(),
        );
    }
}

pub fn start<P: AsRef<Path>>(file: Option<P>) {
    let mut width = super::SETTINGS.read().unwrap().get::<f32>("width").unwrap();
    let mut height = super::SETTINGS.read().unwrap().get::<f32>("height").unwrap();

    let mut system_window = crate::system::System::new("None", width, height, FONT_SIZE);

    let mut win = EditorWindow::new(file);

    // create view and windows cmd binding
    let mut view_cmd = commands::view::get_all();
    let mut view_cmd_keybinding = HashMap::<KeyBinding, usize>::new();
    for i in 0..view_cmd.len() {
        for kb in view_cmd[i].keybinding() {
            view_cmd_keybinding.insert(kb, i);
        }
    }
    let mut win_cmd = commands::window::get_all();
    let mut win_cmd_keybinding = HashMap::<KeyBinding, usize>::new();
    for i in 0..win_cmd.len() {
        for kb in win_cmd[i].keybinding() {
            win_cmd_keybinding.insert(kb, i);
        }
    }

    // main loop
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MouseState {
        Clicked,
        DoubleClicked,
        Released,
    }
    use std::time::{Duration, Instant};
    let mut redraw = true;
    let mut running = true;
    let mut mousex = 0.0;
    let mut mousey = 0.0;
    let mut mouse_state = MouseState::Released;
    let mut last_click_instant = Instant::now();
    while running {
        let mut resized: Option<glutin::dpi::LogicalSize> = None;
        system_window.events_loop.poll_events(|event| {
            use glutin::{dpi::LogicalPosition, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent::*};

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    Refresh => redraw = true,
                    CloseRequested => running = false,
                    Resized(size) => {
                        resized = Some(size);
                    }
                    ReceivedCharacter(ch) => match ch as u32 {
                        0x00...0x1F => (),
                        0x80...0x9F => (),
                        0x7F => (),
                        _ => {
                            win.views[win.current_view].insert_char(ch);
                            redraw = true;
                        }
                    },
                    KeyboardInput { input, .. } => {
                        if input.state == glutin::ElementState::Pressed {
                            if let Some(k) = input.virtual_keycode {
                                let mut km = keybinding::Mod::NONE;
                                if input.modifiers.ctrl {
                                    km |= keybinding::Mod::CTRL
                                }
                                if input.modifiers.alt {
                                    km |= keybinding::Mod::ALT
                                }
                                if input.modifiers.shift {
                                    km |= keybinding::Mod::SHIFT
                                }
                                if input.modifiers.logo {
                                    km |= keybinding::Mod::LOGO
                                }
                                if let Some(cmdid) = view_cmd_keybinding.get(&KeyBinding::new(k, km)) {
                                    view_cmd[*cmdid].as_mut().run(&mut win.views[win.current_view]);
                                }
                                if let Some(cmdid) = win_cmd_keybinding.get(&KeyBinding::new(k, km)) {
                                    win_cmd[*cmdid].as_mut().run(&mut win);
                                }
                                redraw = true;
                            }
                        }
                    }
                    MouseWheel {
                        delta: MouseScrollDelta::LineDelta(_, y),
                        ..
                    } => {
                        let y = y as i32;
                        if y > 0 {
                            win.views[win.current_view].scroll(Direction::Up, y * 3);
                        } else {
                            win.views[win.current_view].scroll(Direction::Down, -y * 3);
                        }
                        redraw = true;
                    }
                    CursorMoved {
                        position: LogicalPosition { x, y },
                        ..
                    } => {
                        mousex = x;
                        mousey = y;
                        if mouse_state == MouseState::Clicked {
                            win.views[win.current_view].click(mousex as _, mousey as _, true);
                            redraw = true;
                        }
                    }
                    MouseInput {
                        button: MouseButton::Left,
                        state: ElementState::Pressed,
                        modifiers,
                        ..
                    } => {
                        let duration = last_click_instant.elapsed();
                        if duration < Duration::from_millis(500) {
                            mouse_state = MouseState::DoubleClicked;
                            win.views[win.current_view].double_click(mousex as _, mousey as _);
                        } else {
                            mouse_state = MouseState::Clicked;
                            win.views[win.current_view].click(mousex as _, mousey as _, modifiers.shift);
                        }
                        last_click_instant = Instant::now();
                        redraw = true;
                    }
                    MouseInput {
                        button: MouseButton::Left,
                        state: ElementState::Released,
                        ..
                    } => {
                        mouse_state = MouseState::Released;
                    }
                    _ => {}
                }
            }
        });
        if let Some(size) = resized {
            system_window
                .window
                .resize(size.to_physical(system_window.hidpi_factor()));
            width = system_window.log_width() as _;
            height = system_window.log_height() as _;
            win.relayout(
                Geometry {
                    x: 0.0,
                    y: 0.0,
                    w: width,
                    h: height,
                },
                &system_window.canvas,
            );
            redraw = true;
        }

        // redraw only when needed
        if redraw {
            // ugly
            win.relayout(win.geometry, &system_window.canvas);

            // clear
            let bg = STYLE.theme.settings.background.unwrap_or(highlighting::Color::BLACK);

            system_window.canvas.clear(nanovg::Color::from_rgb(bg.r, bg.g, bg.b));
            win.draw(&mut system_window.canvas);
            system_window.render();
            system_window.present();
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }

        redraw = false;
    }

    super::SETTINGS.write().unwrap().set("width", width as i64).unwrap();
    super::SETTINGS.write().unwrap().set("height", height as i64).unwrap();
}
