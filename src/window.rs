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

pub struct EditorWindow<'v> {
    views: Vec<View<'v>>,
    buffers: Vec<Rc<RefCell<Buffer>>>,
    geometry: Geometry,
    current_view: usize,
}

pub trait WindowCmd {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn keybinding(&self) -> Vec<KeyBinding>;
    fn run(&mut self, _: &mut EditorWindow<'_>);
}

const FONT_SIZE: f32 = 16.0;

impl<'v> EditorWindow<'v> {
    pub fn new<P: AsRef<Path>>(geometry: Geometry, file: Option<P>) -> Self {
        let mut w = EditorWindow::init(geometry);
        w.add_new_view(file);
        w
    }
    fn init(geometry: Geometry) -> Self {
        let views = Vec::new();
        let buffers = Vec::new();
        EditorWindow {
            views,
            buffers,
            geometry,
            current_view: 0,
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

    fn resize(&mut self, width: f32, height: f32) {
        self.geometry.w = width;
        self.geometry.h = height;
        let mut geometry = self.geometry;
        //geometry.h -= 15; // footer TODO calculate it
        for i in 0..self.views.len() {
            self.views[i].relayout(geometry);
        }
    }
    fn draw(&mut self, canvas: &mut Canvas) {
        // screen.set_font("gui");

        // let footer_height = screen.get_font_metrics("gui").line_spacing;
        // let fg = STYLE.theme.settings.foreground.unwrap_or(highlighting::Color::BLACK);
        // let bg = STYLE.theme.settings.background.unwrap_or(highlighting::Color::WHITE);
        // screen.set_color(Color::RGB(fg.r, fg.g, fg.b));
        // screen.move_to(0, self.geometry.h as i32 - footer_height);
        // screen.draw_rect(self.geometry.w as _, footer_height as _);
        // screen.set_color(Color::RGB(bg.r, bg.g, bg.b));

        // let (line, col) = self.get_current_view().cursor_as_point();
        // screen.draw_str(&format!(
        //     "({},{})    {}    {}",
        //     line,
        //     col,
        //     self.get_current_view().get_syntax(),
        //     self.get_current_view().get_encoding()
        // ));

        self.get_current_view().draw(canvas);
    }
}

pub fn start<P: AsRef<Path>>(file: Option<P>) {
    let mut width = super::SETTINGS.read().unwrap().get::<f32>("width").unwrap();
    let mut height = super::SETTINGS.read().unwrap().get::<f32>("height").unwrap();

    let mut system_window = crate::nanovg::System::new("None", width, height, FONT_SIZE);

    // create window. TODO: passing font_height as parameter feel off
    let font_height = system_window.canvas.font_metrics.line_height;
    let font_advance = system_window.canvas.font_metrics.advance;
    let mut win = EditorWindow::new(
        Geometry {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
            font_height: font_height,
            font_advance: font_advance,
        },
        file,
    );

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
    //let mut event_pump = sdl_context.event_pump().unwrap();

    let mut redraw = true;
    let mut running = true;
    while running {
        // for event in event_pump.poll_iter() {
        //     redraw = true;

        //     if let Event::KeyDown {
        //         keycode: Some(k),
        //         keymod,
        //         ..
        //     } = event
        //     {
        //         let mut km = keybinding::Mod::NONE;
        //         if keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD) {
        //             km |= keybinding::Mod::CTRL
        //         }
        //         if keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD) {
        //             km |= keybinding::Mod::ALT
        //         }
        //         if keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD) {
        //             km |= keybinding::Mod::SHIFT
        //         }
        //         // if keymod.intersects(sdl2::keyboard::NUMMOD) {
        //         //     km |= keybinding::Mod::NUM
        //         // }
        //         if let Some(cmdid) = view_cmd_keybinding.get(&KeyBinding::new(k, km)) {
        //             view_cmd[*cmdid].as_mut().run(&mut win.views[win.current_view]);
        //         }
        //         if let Some(cmdid) = win_cmd_keybinding.get(&KeyBinding::new(k, km)) {
        //             win_cmd[*cmdid].as_mut().run(&mut win);
        //         }
        //     }
        //     //#[cfg_attr(rustfmt, rustfmt_skip)]
        //     match event {
        //         Event::KeyDown {
        //             keycode: Some(Keycode::Escape),
        //             ..
        //         }
        //         | Event::Quit { .. } => break 'mainloop,
        //         Event::Window {
        //             win_event: WindowEvent::SizeChanged(w, h),
        //             ..
        //         } => {
        //             width = w as _;
        //             height = h as _;
        //             win.resize(width as _, height as _);
        //         }
        //         Event::MouseWheel { direction, mut y, .. } => {
        //             if direction == sdl2::mouse::MouseWheelDirection::Normal {
        //                 y *= -1;
        //             }
        //             if y > 0 {
        //                 win.views[win.current_view].move_me(Direction::Down, y * 3)
        //             } else {
        //                 win.views[win.current_view].move_me(Direction::Up, -y * 3)
        //             }
        //         }
        //         Event::MouseButtonDown {
        //             mouse_btn: sdl2::mouse::MouseButton::Left,
        //             x,
        //             y,
        //             ..
        //         } => {
        //             win.views[win.current_view].click(x, y);
        //         }
        //         Event::TextInput { text: t, .. } => {
        //             t.chars().for_each(|c| win.views[win.current_view].insert_char(c));
        //         }
        //         _ => {}
        //     }
        // }
        let mut resized: Option<glutin::dpi::LogicalSize> = None;
        system_window.events_loop.poll_events(|event| {
            use glutin::{Event,WindowEvent::*,MouseScrollDelta};
            if let Event::WindowEvent { event, .. } = event {
                match event {

                    CloseRequested => running = false,
                    Resized(size) => {
                        //system_window.window.resize(size.to_physical(system_window.hidpi_factor()))
                        resized = Some(size);
                    }
                    ReceivedCharacter(ch) => {
                        match ch as u32 {
                            0x00...0x1F => (),
                            0x80...0x9F => (),
                            0x7F => (),
                            _ => {
                                win.views[win.current_view].insert_char(ch);
                                redraw = true;
                            }
                        }
                        //println!("CHR '{:?}' '{:?}'", ch, ch);
                    }
                    KeyboardInput { device_id, input } => {
                        //println!("KBI {:?} {:?}", device_id, input);
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
                    MouseWheel { delta: MouseScrollDelta::LineDelta(_,y), ..} => {
                        if y>0.0 {
                            win.views[win.current_view].move_me(Direction::Up, y as _);
                        } else {
                            win.views[win.current_view].move_me(Direction::Down, -y as _);
                        }
                        redraw = true;
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
            win.resize(width as _, height as _);
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
        //screen.render(&mut canvas);
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }

        redraw = false;
    }

    super::SETTINGS.write().unwrap().set("width", width as i64).unwrap();
    super::SETTINGS.write().unwrap().set("height", height as i64).unwrap();
}
