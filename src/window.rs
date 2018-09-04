
use std::cell::RefCell;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;
use std::{thread,time};
use std::collections::HashMap;

use sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::ttf::Font;

use buffer::Buffer;
use fontcache::GlyphCache;
use commands;
use view::{Direction, View};
use keybinding;
use keybinding::KeyBinding;

pub enum DisplayCommand {
    Move(i32, i32),
    Char(char, Color),
    Rect(u32, u32, Color),
}
pub struct EditorWindow {
    views: Vec<View>,
    buffers: Vec<Rc<RefCell<Buffer>>>,
    width: usize,
    height: usize,
    font_height: usize,
    current_view: usize,
}

const FONT_SIZE: u16 = 13;

impl EditorWindow {
    pub fn new<P: AsRef<Path>>(width: usize, height: usize, font_height: usize, file: Option<P>) -> Self {
        let mut w = EditorWindow::init(width, height, font_height);
        w.add_new_view(file);
        return w;
    }
    fn init(width: usize, height: usize, font_height: usize) -> Self {
        let views = Vec::new();
        let buffers = Vec::new();
        let w = EditorWindow {
            views,
            buffers,
            width,
            height,
            font_height,
            //page_height: height / font_height - 1,
            current_view: 0,
        };
        return w;
    }

    fn add_new_view<P: AsRef<Path>>(&mut self, file: Option<P>) {
        let b = match file {
            None => Rc::new(RefCell::new(Buffer::new())),
            Some(file) => Rc::new(RefCell::new(Buffer::from_file(file.as_ref()).expect("File not found"))),
        };
        self.buffers.push(b.clone());
        let mut v = View::new(b.clone());
        v.set_page_length(self.height / self.font_height - 1);
        self.views.push(v);
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        let page_length = self.height / self.font_height - 1;
        self.views[self.current_view].set_page_length(page_length);
    }
    fn draw(&mut self, display_list: &mut Vec<DisplayCommand>, font: &Font) {
        let mut y = 0;
        let mut x = 0;
        let adv = font.find_glyph_metrics(' ').unwrap().advance;

        let b = self.views[self.current_view].buffer.borrow();
        let first_char = b.line_to_char(self.views[self.current_view].first_visible_line());
        let mut idx = first_char;

        for c in b.chars().skip(first_char) {
            match self.views[self.current_view].selection {
                None => (),
                Some(Range { start, end }) if start <= idx && end > idx && c != '\n' => {
                    display_list.push(DisplayCommand::Move(x, y));
                    display_list.push(DisplayCommand::Rect(
                        (adv + 1) as _,
                        font.height() as _,
                        Color::RGB(142, 132, 155),
                    ));
                }
                _ => (),
            }
            if idx == self.views[self.current_view].index() {
                display_list.push(DisplayCommand::Move(x, y));
                display_list.push(DisplayCommand::Rect(2, font.height() as _, Color::RGB(242, 232, 255)));
            }
            match c {
                '\n' => {
                    y += font.recommended_line_spacing();
                    if y > self.height as i32 {
                        break;
                    }
                    x = 0;
                }
                '\t' => {
                    x += adv * 4;
                }
                '\r' => (),
                _ => {
                    display_list.push(DisplayCommand::Move(x, y));
                    display_list.push(DisplayCommand::Char(c,Color::RGB(242, 232, 255)));
                    x += adv;
                }
            }

            idx += 1;
        }
        // cursor at eof position
        if idx == self.views[self.current_view].index() {
            display_list.push(DisplayCommand::Move(x, y));
            display_list.push(DisplayCommand::Rect(2, font.height() as _, Color::RGB(242, 232, 255)));
        }
    }
}

pub fn start<P: AsRef<Path>>(file: Option<P>) {

    let mut width = super::SETTINGS.read().unwrap().get::<usize>("width").unwrap();
    let mut height = super::SETTINGS.read().unwrap().get::<usize>("height").unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    //video_subsystem.text_input().stop();
    let display = video_subsystem
        .window("None", width as u32, height as u32)
        .position_centered()
        //.opengl()
        .resizable()
        .build()
        .unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut canvas = display.into_canvas().accelerated().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let font_data = include_bytes!("monofont/UbuntuMono-Regular.ttf");
    let mut font = ttf_context
        .load_font_from_rwops(sdl2::rwops::RWops::from_bytes(font_data).unwrap(), FONT_SIZE)
        .unwrap();
    font.set_hinting(sdl2::ttf::Hinting::Normal);
    font.set_style(sdl2::ttf::STYLE_BOLD);

    let font_height = font.recommended_line_spacing();

    let mut font_cache = GlyphCache::new(1024, font);
    font_cache.grow(&texture_creator);
    
    let mut win = EditorWindow::new(width, height, font_height as _, file);

    let mut view_cmd = commands::view::get_all();
    let mut cmd_keybinding = HashMap::<KeyBinding,usize>::new();
    for i in 0 .. view_cmd.len() {
        for kb in view_cmd[i].keybinding() {
            cmd_keybinding.insert(kb, i);
        }
    }

    let mut display_list = Vec::<DisplayCommand>::new();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut redraw = true;
    'mainloop: loop {
        for event in event_pump.poll_iter() {
            redraw = true;
            
            match event { Event::KeyDown{keycode: Some(k),keymod,
                    ..} => {
                println!("{:?} {:?}", k,keymod);
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
                if let Some(cmdid) = cmd_keybinding.get(&KeyBinding::new(k, km)) {
                    view_cmd[*cmdid].as_mut().run(&mut win.views[win.current_view]);
                }}, 
                _ => (),
            }
            #[cfg_attr(rustfmt, rustfmt_skip)]
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
                    win.resize(width, height);
                },
                Event::KeyDown { keycode: Some(Keycode::LShift), .. }
                | Event::KeyDown { keycode: Some(Keycode::RShift), .. } => win.views[win.current_view].start_selection(),
                Event::KeyUp { keycode: Some(Keycode::LShift), .. }
                | Event::KeyUp { keycode: Some(Keycode::RShift), .. } => win.views[win.current_view].end_selection(),
                
                Event::TextInput { text: t, .. } => {
                    t.chars().for_each(|c| win.views[win.current_view].insert_char(c));
                }
                _ => {}
            }
        }
        
        // redraw only when needed
        if redraw {
            // clear
            canvas.set_draw_color(Color::RGB(0, 43, 53));
            canvas.clear();
            display_list.clear();

            // process display list
            win.draw(&mut display_list, &font_cache.font);
            {
                let mut x: i32 = 0;
                let mut y: i32 = 0;
                for cmd in &display_list {
                    match *cmd {
                        DisplayCommand::Move(to_x, to_y) => {
                            x = to_x;
                            y = to_y
                        }
                        DisplayCommand::Rect(w, h, color) => {
                            canvas.set_draw_color(color);
                            canvas.fill_rect(sdl2::rect::Rect::new(x, y, w, h)).unwrap();
                        }
                        DisplayCommand::Char(c, color) => {
                            let ch = font_cache.get(c, color);
                            let tex = &font_cache.textures[ch.textureid as usize];
                            canvas
                                .copy(&tex, ch.rect, sdl2::rect::Rect::new(x, y, ch.rect.width(), ch.rect.height()))
                                .unwrap();
                        }
                    }
                }
            }
            canvas.present();    
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }
        
        redraw = false;
    }
    
    super::SETTINGS.write().unwrap().set("width",width as i64).unwrap();
    super::SETTINGS.write().unwrap().set("height",height as i64).unwrap();
    
}
