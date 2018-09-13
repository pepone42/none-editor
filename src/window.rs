use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::{thread,time};
use std::collections::HashMap;

use sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use syntect::highlighting::{Theme,ThemeSet};
use syntect::highlighting;

use buffer::Buffer;
use commands;
use view::View;
use keybinding;
use keybinding::KeyBinding;
use canvas;

pub struct EditorWindow {
    views: Vec<View>,
    buffers: Vec<Rc<RefCell<Buffer>>>,
    width: usize,
    height: usize,
    font_height: usize,
    current_view: usize,
}

pub trait WindowCmd {
    fn name(&self) -> &'static str;
    fn desc(&self) ->  &'static str;
    fn keybinding(&self) -> Vec<KeyBinding>;
    fn run(&mut self,&mut EditorWindow);
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
    fn draw(&mut self, screen: &mut canvas::Screen, theme: &Theme) {
        screen.set_font("gui");

        let footer_height = screen.get_font_metrics("gui").line_spacing;
        let fg = theme.settings.foreground.unwrap_or(highlighting::Color::BLACK);
        let bg = theme.settings.background.unwrap_or(highlighting::Color::WHITE);
        screen.set_color(Color::RGB(fg.r, fg.g, fg.b));
        screen.move_to(0, (self.height-(footer_height as usize)) as i32);
        screen.draw_rect(self.width as _, footer_height as _);
        screen.set_color(Color::RGB(bg.r, bg.g, bg.b));

        let (line, col) = self.views[self.current_view].cursor_as_point();
        screen.draw_str(&format!("({},{})",line,col));

        self.views[self.current_view].draw(screen,theme,0,0,self.width as _,self.height as _);
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
    
    let mut screen = canvas::Screen::new();
    
    let font_data = include_bytes!("monofont/UbuntuMono-Regular.ttf");
    screen.add_font_from_ubyte(&ttf_context, &texture_creator,"mono",font_data, FONT_SIZE);

    let font_data = include_bytes!("monofont/ubuntu.regular.ttf");
    screen.add_font_from_ubyte(&ttf_context, &texture_creator,"gui",font_data, 10);


    // create window. TODO: passing font_height as parameter feel off
    let font_height = screen.get_font_metrics("mono").line_spacing;
    let mut win = EditorWindow::new(width, height, font_height as _, file);


    // create view and windows cmd binding
    let mut view_cmd = commands::view::get_all();
    let mut view_cmd_keybinding = HashMap::<KeyBinding,usize>::new();
    for i in 0 .. view_cmd.len() {
        for kb in view_cmd[i].keybinding() {
            view_cmd_keybinding.insert(kb, i);
        }
    }
    let mut win_cmd = commands::window::get_all();
    let mut win_cmd_keybinding = HashMap::<KeyBinding,usize>::new();
    for i in 0 .. win_cmd.len() {
        for kb in win_cmd[i].keybinding() {
            win_cmd_keybinding.insert(kb, i);
        }
    }

    // Theme
    let ts = ThemeSet::load_defaults();

    // main loop
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut redraw = true;
    'mainloop: loop {
        for event in event_pump.poll_iter() {
            redraw = true;
            
            match event { Event::KeyDown{keycode: Some(k),keymod,
                    ..} => {
                //println!("{:?} {:?}", k,keymod);
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
            let theme = &ts.themes["base16-ocean.dark"];
            let bg = theme.settings.background.unwrap_or(highlighting::Color::BLACK);
            // canvas.set_draw_color(Color::RGB(bg.r, bg.g, bg.b));
            // canvas.clear();
            screen.clear(Color::RGB(bg.r, bg.g, bg.b));
            win.draw(&mut screen , theme);
            screen.render(&mut canvas);
            //canvas.present();    
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }
        
        redraw = false;
    }

    super::SETTINGS.write().unwrap().set("width",width as i64).unwrap();
    super::SETTINGS.write().unwrap().set("height",height as i64).unwrap();
    
}
