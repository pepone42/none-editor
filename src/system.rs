use gl;
use glutin;
use glutin::GlContext;
use nanovg;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub enum FontType {
    MonoSpaced(f32),
    Proportional,
}
#[derive(Debug, Copy, Clone)]
pub struct Font {
    id: usize,
    pub kind: FontType,
    pub ascender: f32,
    pub descender: f32,
    pub line_height: f32,
}

#[derive(Debug)]
pub enum DisplayList {
    Move(f32, f32),
    Translate(f32, f32),
    Color(nanovg::Color),
    Char(char),
    Rect(f32, f32),
    Clear,
    Font(&'static str),
}

pub struct Canvas {
    cmdlist: Vec<DisplayList>,
    pub fonts: HashMap<&'static str, Font>,
}

impl Canvas {
    fn new() -> Self {
        Canvas {
            cmdlist: Vec::new(),
            fonts: HashMap::new(),
        }
    }
    pub fn clear(&mut self, color: nanovg::Color) {
        self.cmdlist.clear();
        self.set_color(color);
        self.cmdlist.push(DisplayList::Clear);
    }

    pub fn set_color(&mut self, color: nanovg::Color) {
        self.cmdlist.push(DisplayList::Color(color));
    }
    /// Draw a rect
    pub fn draw_rect(&mut self, w: f32, h: f32) {
        self.cmdlist.push(DisplayList::Rect(w, h));
    }

    /// Draw a char
    pub fn draw_char(&mut self, c: char) {
        self.cmdlist.push(DisplayList::Char(c));
    }

    /// Draw a string
    pub fn draw_str(&mut self, s: &str) {
        for c in s.chars() {
            self.draw_char(c);
        }
    }

    /// move the pointer to x,y
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.cmdlist.push(DisplayList::Move(x, y));
    }

    /// translate the cursor
    pub fn translate(&mut self, x: f32, y: f32) {
        self.cmdlist.push(DisplayList::Move(x, y));
    }
}

pub struct System {
    pub events_loop: glutin::EventsLoop,
    pub window: glutin::GlWindow,
    nvgcontext: nanovg::Context,
    text_option: nanovg::TextOptions,
    pub canvas: Canvas,
}

impl System {
    pub fn new(title: &str, width: f32, height: f32, font_size: f32) -> Self {
        let log_size = glutin::dpi::LogicalSize::new(width as _, height as _);
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new().with_title(title).with_dimensions(log_size);

        let context = glutin::ContextBuilder::new().with_vsync(true).with_srgb(true);
        let window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
        window.set_cursor(glutin::MouseCursor::Default);

        unsafe {
            window.make_current().unwrap();
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        let nvgcontext = nanovg::ContextBuilder::new()
            .build()
            .expect("Initialization of NanoVG failed!");

        let mono_font = nanovg::Font::from_memory(&nvgcontext, "Mono", include_bytes!("monofont/Inconsolata-Bold.ttf"))
            .expect("Failed to load font");

        let text_option = nanovg::TextOptions {
            color: nanovg::Color::new(1.0, 1.0, 1.0, 1.0),
            size: font_size,
            align: nanovg::Alignment::new().baseline(),
            ..Default::default()
        };

        let hidpi_factor = window.get_current_monitor().get_hidpi_factor();

        let mut advance: f32 = 0.0;
        let mut text_metrics: nanovg::TextMetrics = nanovg::TextMetrics {
            ascender: 0.0,
            descender: 0.0,
            line_height: 0.0,
        };

        nvgcontext.frame(
            (log_size.width as _, log_size.height as _),
            hidpi_factor as _,
            |frame| {
                advance = frame.text_bounds(mono_font, (0.0, 0.0), "_", text_option).0;
                text_metrics = frame.text_metrics(mono_font, text_option);
            },
        );

        let mut canvas = Canvas::new();
        let font_info = Font {
            id: 0,
            kind: FontType::MonoSpaced(advance),
            ascender: text_metrics.ascender,
            descender: text_metrics.descender,
            line_height: text_metrics.line_height,
        };
        canvas.fonts.insert("mono", font_info);

        System {
            events_loop,
            window,
            nvgcontext,
            text_option,
            canvas,
        }
    }

    pub fn log_width(&self) -> f64 {
        self.window.get_inner_size().unwrap().width
    }

    pub fn log_height(&self) -> f64 {
        self.window.get_inner_size().unwrap().height
    }

    pub fn hidpi_factor(&self) -> f64 {
        self.window.get_current_monitor().get_hidpi_factor()
    }

    fn phy_width(&self) -> f64 {
        self.window
            .get_inner_size()
            .unwrap()
            .to_physical(self.hidpi_factor())
            .width
    }

    fn phy_height(&self) -> f64 {
        self.window
            .get_inner_size()
            .unwrap()
            .to_physical(self.hidpi_factor())
            .height
    }

    pub fn render(&mut self) {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut color = nanovg::Color::from_rgb(0, 0, 0);

        // default font
        let mut font = nanovg::Font::find(&self.nvgcontext, "Mono").unwrap();
        let font_metrics = self.canvas.fonts["mono"];
        let mut text_option = self.text_option;

        let log_width = self.log_width();
        let log_height = self.log_height();

        self.nvgcontext
            .frame((log_width as _, log_height as _), self.hidpi_factor() as _, |frame| {
                for cmd in &self.canvas.cmdlist {
                    match *cmd {
                        DisplayList::Color(col) => color = col,
                        DisplayList::Move(to_x, to_y) => {
                            x = to_x;
                            y = to_y
                        }
                        DisplayList::Translate(dx, dy) => {
                            x += dx;
                            y += dy;
                        }
                        DisplayList::Rect(w, h) => {
                            frame.path(
                                |p| {
                                    p.rect((x, y), (w, h));
                                    p.fill(color, Default::default());
                                },
                                Default::default(),
                            );
                        }
                        DisplayList::Char(c) => {
                            text_option.color = color;
                            frame.text(font, (x, y), c.to_string(), text_option);
                            if let FontType::MonoSpaced(advance) = font_metrics.kind {
                                x += advance;
                            } else {
                                // TODO: proportional font
                                unimplemented!();
                            }
                        }
                        DisplayList::Clear => {
                            frame.path(
                                |p| {
                                    p.rect((0.0, 0.0), (log_width as _, log_height as _));
                                    p.fill(color, Default::default());
                                },
                                Default::default(),
                            );
                        }
                        DisplayList::Font(f) => {
                            font = nanovg::Font::find(&self.nvgcontext, f).unwrap();
                        }
                    }
                }
            });
    }

    /// Clear the screen
    pub fn clear(&mut self) {
        // Without physical size, glviewport does not work correctly when hdpi_foctor != 1.0
        let (width, height): (u32, u32) = self
            .window
            .get_inner_size()
            .unwrap()
            .to_physical(self.hidpi_factor())
            .into();
        unsafe {
            gl::Viewport(0, 0, width as _, height as _);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }

    /// swap buffers
    pub fn present(&mut self) {
        self.window.swap_buffers().unwrap();
    }
}
