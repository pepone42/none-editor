use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render;
use sdl2::render::TextureCreator;
use sdl2::rwops;
use sdl2::ttf;
use sdl2::video::WindowContext;
use std::collections::HashMap;

use fontcache::GlyphCache;

pub enum DisplayCommand {
    Move(i32, i32),
    Color(Color),
    Char(char),
    Rect(u32, u32),
    Font(usize),
}

pub struct FontMetrics {
    pub line_spacing: i32,
}

pub struct GlyphMetrics {
    pub advance: i32,
}

pub struct Screen<'t, 'ttf_context, 'rwops> {
    cmd_list: Vec<DisplayCommand>,
    fonts: Vec<GlyphCache<'t, 'ttf_context, 'rwops>>,
    font_name: HashMap<String, usize>,
    //ttf_context: ttf::Sdl2TtfContext,
}

impl<'t, 'ttf_context, 'rwops> Screen<'t, 'ttf_context, 'rwops> {
    pub fn new() -> Self {
        Screen {
            cmd_list: Vec::new(),
            fonts: Vec::new(),
            font_name: HashMap::new(),
            //ttf_context: ttf::init().unwrap(),
        }
    }

    //pub fn add_font_from_ubyte<'tc: 't>(&'ttf_context mut self,texture_creator: &'tc TextureCreator<WindowContext>, font_name: &str,data: &'rwops [u8], size: u16) {
    pub fn add_font_from_ubyte<'tc: 't>(
        &mut self,
        ttf_context: &'ttf_context ttf::Sdl2TtfContext,
        texture_creator: &'tc TextureCreator<WindowContext>,
        font_name: &str,
        data: &'rwops [u8],
        size: u16,
    ) {
        let rwops = rwops::RWops::from_bytes(data).unwrap();
        //let ttf : &'s ttf::Sdl2TtfContext = &self.ttf_context;
        let mut font = ttf_context.load_font_from_rwops(rwops, size).unwrap();

        font.set_hinting(ttf::Hinting::Normal);
        //font.set_style(sdl2::ttf::STYLE_BOLD);

        let id = self.fonts.len();

        let mut font_cache = GlyphCache::new(1024, font);
        font_cache.grow(&texture_creator);

        self.fonts.push(font_cache);
        self.font_name.insert(font_name.to_owned(), id);
    }

    /// Clear the screen
    pub fn clear(&mut self) {
        self.cmd_list.clear();
    }

    pub fn get_font_metrics(&self, font_name: &str) -> FontMetrics {
        let fontid = self.font_name[font_name];
        let cache = &self.fonts[fontid];
        FontMetrics {
            line_spacing: cache.font.recommended_line_spacing(),
        }
    }

    pub fn find_glyph_metrics(&self, font_name: &str, c: char) -> Option<GlyphMetrics> {
        let fontid = self.font_name[font_name];
        let cache = &self.fonts[fontid];
        cache
            .font
            .find_glyph_metrics(c)
            .map(|x| GlyphMetrics { advance: x.advance })
    }

    /// Draw a rect
    pub fn draw_rect(&mut self, w: u32, h: u32) {
        self.cmd_list.push(DisplayCommand::Rect(w, h));
    }

    /// Draw a char
    pub fn draw_char(&mut self, c: char) {
        self.cmd_list.push(DisplayCommand::Char(c));
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.cmd_list.push(DisplayCommand::Move(x, y));
    }

    pub fn set_color(&mut self, color: Color) {
        self.cmd_list.push(DisplayCommand::Color(color));
    }

    pub fn set_font(&mut self, font_name: &str) {
        let id = self.font_name[font_name];
        self.cmd_list.push(DisplayCommand::Font(id));
    }

    /// render the screen
    pub fn render<T: render::RenderTarget>(&mut self, sdl2_canvas: &mut render::Canvas<T>) {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut fontid: usize = 0;
        let mut color: Color = Color::RGB(0, 0, 0);
        for cmd in &self.cmd_list {
            match *cmd {
                DisplayCommand::Color(col) => color = col,
                DisplayCommand::Move(to_x, to_y) => {
                    x = to_x;
                    y = to_y
                }
                DisplayCommand::Rect(w, h) => {
                    sdl2_canvas.set_draw_color(color);
                    sdl2_canvas.fill_rect(Rect::new(x, y, w, h)).unwrap();
                }
                DisplayCommand::Char(c) => {
                    let ch = self.fonts[fontid].get(c, color);
                    let tex = &self.fonts[fontid].textures[ch.textureid as usize];
                    sdl2_canvas
                        .copy(&tex, ch.rect, Rect::new(x, y, ch.rect.width(), ch.rect.height()))
                        .unwrap();
                }
                DisplayCommand::Font(id) => {
                    fontid = id;
                }
            }
        }
        sdl2_canvas.present();
    }
}
