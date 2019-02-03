use std::sync::RwLock;
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
use crate::nanovg::Canvas;
use crate::view::{Direction, View};
use nanovg::Color;

use crate::styling::STYLE;

#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub font_height: f32,
    pub font_advance: f32,
}

pub struct StatusBar {
    geometry: Geometry,
    left: HashMap<String,String>,
    right: HashMap<String,String>,
}

impl StatusBar {
    pub fn new(geometry: Geometry) -> Self {
        StatusBar { geometry, left: HashMap::new(), right: HashMap::new() }
    }
    pub fn draw(&self, canvas: &mut Canvas) {
        let bg_color = STYLE.theme.settings.foreground.unwrap_or(highlighting::Color::WHITE);
        let fg_color = STYLE.theme.settings.background.unwrap_or(highlighting::Color::BLACK);
        canvas.set_color(Color::from_rgb(bg_color.r, bg_color.g, bg_color.b));

        canvas.move_to(self.geometry.x, self.geometry.y);
        canvas.draw_rect(self.geometry.w, self.geometry.h);
        canvas.set_color(Color::from_rgb(fg_color.r, fg_color.g, fg_color.b));

        canvas.move_to(self.geometry.x, self.geometry.y + self.geometry.h - 1.0);
        for info in  self.left.values() {
            canvas.draw_str(info);
            canvas.draw_str("  ");
        }
        
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
    pub fn new<P: AsRef<Path>>(geometry: Geometry, file: Option<P>) -> Self {
        let mut w = EditorWindow::init(geometry);
        w.add_new_view(file);
        w
    }
    fn init(geometry: Geometry) -> Self {
        let views = Vec::new();
        let buffers = Vec::new();
        let mut statusbar = StatusBar::new(geometry);
        statusbar.geometry.h = geometry.font_height;
        statusbar.geometry.y = geometry.h - geometry.font_height;
        EditorWindow {
            views,
            buffers,
            geometry,
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
        let mut geometry = self.geometry;
        //geometry.h -= 15; // footer TODO calculate it
        let mut v = View::new(b.clone(), geometry);
        v.detect_syntax();

        let viewid = self.views.len();
        self.views.push(v);
        self.current_view = viewid;
    }

    fn resize(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.geometry.w = w;
        self.geometry.h = h - self.geometry.font_height;
        self.statusbar.geometry.y = self.geometry.h;
        self.statusbar.geometry.w = w;
        self.relayout();
    }
    fn relayout(&mut self) {
        for i in 0..self.views.len() {
            self.views[i].relayout(self.geometry);
        }
    }
    fn draw(&mut self, canvas: &mut Canvas) {
        self.get_current_view().draw(canvas);
        self.statusbar.draw(canvas);
    }
}

pub fn start<P: AsRef<Path>>(file: Option<P>) {
    let mut width = super::SETTINGS.read().unwrap().get::<f32>("width").unwrap();
    let mut height = super::SETTINGS.read().unwrap().get::<f32>("height").unwrap();

    let mut system_window = crate::nanovg::System::new("None", width, height, FONT_SIZE);

    // create window. TODO: passing font_height as parameter feel off
    let font_height = system_window.canvas.font_metrics.line_height;
    let font_advance = system_window.canvas.font_metrics.advance;

    let mut win = EditorWindow::new(Geometry {
        x: 0.0,
        y: 0.0,
        w: width,
        h: height,
        font_height: font_height,
        font_advance: font_advance,
    }, file);

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
                        modifiers,
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
            win.resize(0.0,0.0,width,height);
            redraw = true;
        }

        // redraw only when needed
        if redraw {
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
