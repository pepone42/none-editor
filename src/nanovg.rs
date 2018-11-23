use gl;
use glutin;
use glutin::GlContext;
use nanovg;

#[derive(Debug)]
pub enum DisplayList {
    Move(f32, f32),
    Color(nanovg::Color),
    Char(char),
    Rect(f32, f32),
    Clear,
}

pub struct Canvas(Vec<DisplayList>);

impl Canvas {
    pub fn new() -> Self {
        Canvas(Vec::new())
    }
    pub fn clear(&mut self, color: nanovg::Color) {
        self.0.clear();
        self.set_color(color);
        self.0.push(DisplayList::Clear);
    }

    pub fn set_color(&mut self, color: nanovg::Color) {
        self.0.push(DisplayList::Color(color));
    }
    /// Draw a rect
    pub fn draw_rect(&mut self, w: f32, h: f32) {
        self.0.push(DisplayList::Rect(w, h));
    }

    /// Draw a char
    pub fn draw_char(&mut self, c: char) {
        self.0.push(DisplayList::Char(c));
    }

    /// move the pointer to x,y
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(DisplayList::Move(x, y));
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
        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(glutin::dpi::LogicalSize::new(width as _, height as _));
        let context = glutin::ContextBuilder::new().with_vsync(true).with_srgb(true);
        let window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

        unsafe {
            window.make_current().unwrap();
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        let nvgcontext = nanovg::ContextBuilder::new()
            .build()
            .expect("Initialization of NanoVG failed!");

        let mono_font = nanovg::Font::from_memory(&nvgcontext, "Mono", include_bytes!("monofont/UbuntuMono-B.ttf"))
            .expect("Failed to load font");

        let text_option = nanovg::TextOptions {
            color: nanovg::Color::new(1.0, 1.0, 1.0, 1.0),
            size: font_size,
            align: nanovg::Alignment::new().baseline(),
            ..Default::default()
        };

        let hidpi_factor = window.get_current_monitor().get_hidpi_factor();
        println!(
            "{:?} {:?} {:?}",
            window.get_inner_size(),
            hidpi_factor,
            window.get_inner_size().unwrap().to_physical(hidpi_factor)
        );

        System {
            events_loop,
            window,
            nvgcontext,
            text_option,
            canvas: Canvas::new(),
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
        self.window.get_inner_size().unwrap().to_physical(self.hidpi_factor()).width
    }

    fn phy_height(&self) -> f64 {
        self.window.get_inner_size().unwrap().to_physical(self.hidpi_factor()).height
    }

    pub fn line_spacing(&self) -> f32 {
        let mut line_height = 0.0;

        let font = nanovg::Font::find(&self.nvgcontext, "Mono").unwrap();
        let text_option = self.text_option;
        self.nvgcontext.frame((self.log_width() as _, self.log_height() as _), self.hidpi_factor() as _, |frame| {
            line_height = frame.text_metrics(font, text_option).line_height;
        });
        line_height
    }

    pub fn char_advance(&self) -> f32 {
        let mut advance = 0.0;
        let font = nanovg::Font::find(&self.nvgcontext, "Mono").unwrap();
        let text_option = self.text_option;
        self.nvgcontext.frame((self.log_width() as _, self.log_height() as _), self.hidpi_factor() as _, |frame| {
            advance = frame.text_bounds(font, (0.0, 0.0), "_", text_option).0;
        });
        advance
    }

    pub fn render(&mut self) {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut color = nanovg::Color::from_rgb(0, 0, 0);

        let font = nanovg::Font::find(&self.nvgcontext, "Mono").unwrap();
        let mut text_option = self.text_option;

        let phy_width = self.phy_width();
        let phy_height = self.phy_height();

        self.nvgcontext.frame((self.log_width() as _, self.log_height() as _), self.hidpi_factor() as _, |frame| {
            for cmd in &self.canvas.0 {
                match *cmd {
                    DisplayList::Color(col) => color = col,
                    DisplayList::Move(to_x, to_y) => {
                        x = to_x;
                        y = to_y
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
                        x += self.char_advance();
                    }
                    DisplayList::Clear => unsafe {
                        gl::ClearColor(color.red(), color.green(), color.blue(), color.alpha());
                        gl::Viewport(0, 0, phy_width as _, phy_height as _);
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
                    },
                }
            }
        });
    }
    pub fn present(&mut self) {
        self.window.swap_buffers().unwrap();
    }
}
