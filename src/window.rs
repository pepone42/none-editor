use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::{thread, time};

use sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use syntect::highlighting;

use crate::buffer::Buffer;
use crate::canvas;
use crate::commands;
use crate::keybinding;
use crate::keybinding::KeyBinding;
use crate::view::{Direction, View};

use crate::styling::STYLE;

#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub font_height: u32,
    pub font_advance: u32,
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

const FONT_SIZE: u16 = 13;

impl<'v> EditorWindow<'v> {
    pub fn new<P: AsRef<Path>>(geometry: Geometry, file: Option<P>) -> Self {
        assert_eq!(geometry.x, 0);
        assert_eq!(geometry.y, 0);
        let mut w = EditorWindow::init(geometry);
        w.add_new_view(file);
        w
    }
    fn init(geometry: Geometry) -> Self {
        let views = Vec::new();
        let buffers = Vec::new();
        assert_eq!(geometry.x, 0);
        assert_eq!(geometry.y, 0);
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
        geometry.h -= 15; // footer TODO calculate it
        let mut v = View::new(b.clone(), geometry);
        v.detect_syntax();

        let viewid = self.views.len();
        self.views.push(v);
        self.current_view = viewid;
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.geometry.w = width;
        self.geometry.h = height;
        let mut geometry = self.geometry;
        geometry.h -= 15; // footer TODO calculate it
        for i in 0..self.views.len() {
            self.views[i].relayout(geometry);
        }
    }
    fn draw(&mut self, screen: &mut canvas::Screen<'_, '_, '_>) {
        screen.set_font("gui");

        let footer_height = screen.get_font_metrics("gui").line_spacing;
        let fg = STYLE.theme.settings.foreground.unwrap_or(highlighting::Color::BLACK);
        let bg = STYLE.theme.settings.background.unwrap_or(highlighting::Color::WHITE);
        screen.set_color(Color::RGB(fg.r, fg.g, fg.b));
        screen.move_to(0, self.geometry.h as i32 - footer_height);
        screen.draw_rect(self.geometry.w as _, footer_height as _);
        screen.set_color(Color::RGB(bg.r, bg.g, bg.b));

        let (line, col) = self.get_current_view().cursor_as_point();
        screen.draw_str(&format!(
            "({},{})    {}    {}",
            line,
            col,
            self.get_current_view().get_syntax(),
            self.get_current_view().get_encoding()
        ));

        self.get_current_view().draw(screen);
    }
}

pub fn start<P: AsRef<Path>>(file: Option<P>) {
    let mut width = super::SETTINGS.read().unwrap().get::<usize>("width").unwrap();
    let mut height = super::SETTINGS.read().unwrap().get::<usize>("height").unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let display = video_subsystem
        .window("None", width as u32, height as u32)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let mut canvas = display.into_canvas().accelerated().present_vsync().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let mut screen = canvas::Screen::new();

    let font_data = include_bytes!("monofont/Inconsolata-Bold.ttf");
    screen.add_font_from_ubyte(&ttf_context, &texture_creator, "mono", font_data, FONT_SIZE);

    let font_data = include_bytes!("monofont/ubuntu.regular.ttf");
    screen.add_font_from_ubyte(&ttf_context, &texture_creator, "gui", font_data, 10);

    // create window. TODO: passing font_height as parameter feel off
    let font_height = screen.get_font_metrics("mono").line_spacing;
    let font_advance = screen.find_glyph_metrics("mono", ' ').unwrap().advance;
    let mut win = EditorWindow::new(
        Geometry {
            x: 0,
            y: 0,
            w: width as _,
            h: height as _,
            font_height: font_height as u32,
            font_advance: font_advance as u32,
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
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut redraw = true;
    'mainloop: loop {
        for event in event_pump.poll_iter() {
            redraw = true;

            if let Event::KeyDown {
                keycode: Some(k),
                keymod,
                ..
            } = event
            {
                let mut km = keybinding::Mod::NONE;
                if keymod.intersects(sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD) {
                    km |= keybinding::Mod::CTRL
                }
                if keymod.intersects(sdl2::keyboard::LALTMOD | sdl2::keyboard::RALTMOD) {
                    km |= keybinding::Mod::ALT
                }
                if keymod.intersects(sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD) {
                    km |= keybinding::Mod::SHIFT
                }
                // if keymod.intersects(sdl2::keyboard::NUMMOD) {
                //     km |= keybinding::Mod::NUM
                // }
                if let Some(cmdid) = view_cmd_keybinding.get(&KeyBinding::new(k, km)) {
                    view_cmd[*cmdid].as_mut().run(&mut win.views[win.current_view]);
                }
                if let Some(cmdid) = win_cmd_keybinding.get(&KeyBinding::new(k, km)) {
                    win_cmd[*cmdid].as_mut().run(&mut win);
                }
            }
            //#[cfg_attr(rustfmt, rustfmt_skip)]
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h),
                    ..
                } => {
                    width = w as _;
                    height = h as _;
                    win.resize(width as _, height as _);
                }
                Event::MouseWheel { direction, mut y, .. } => {
                    if direction == sdl2::mouse::MouseWheelDirection::Normal {
                        y *= -1;
                    }
                    if y > 0 {
                        win.views[win.current_view].move_me(Direction::Down, y * 3)
                    } else {
                        win.views[win.current_view].move_me(Direction::Up, -y * 3)
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn: sdl2::mouse::MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    win.views[win.current_view].click(x, y);
                }
                Event::TextInput { text: t, .. } => {
                    t.chars().for_each(|c| win.views[win.current_view].insert_char(c));
                }
                _ => {}
            }
        }

        // redraw only when needed
        if redraw {
            // clear
            let bg = STYLE.theme.settings.background.unwrap_or(highlighting::Color::BLACK);

            screen.clear(Color::RGB(bg.r, bg.g, bg.b));
            win.draw(&mut screen);
            screen.render(&mut canvas);
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }

        redraw = false;
    }

    super::SETTINGS.write().unwrap().set("width", width as i64).unwrap();
    super::SETTINGS.write().unwrap().set("height", height as i64).unwrap();
}
