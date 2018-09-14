use std::collections::HashMap;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use rect_packer;

#[derive(Debug, Clone, Copy)]
pub struct CachedGlyph {
    pub textureid: u32,
    pub rect: rect::Rect,
}

// simple font atlas
pub struct GlyphCache<'t, 'ttf_context, 'rwops> {
    //texture_creator: TextureCreator<WindowContext>,
    pub textures: Vec<Texture<'t>>,
    packers: Vec<rect_packer::DensePacker>,
    glyphs: HashMap<(char, Color), CachedGlyph>,
    size: u32,
    pub font: Font<'ttf_context, 'rwops>,
}

impl<'t, 'ttf_context, 'rwops> GlyphCache<'t, 'ttf_context, 'rwops> {
    pub fn new(size: u32, font: Font<'ttf_context, 'rwops>) -> Self {
        GlyphCache {
            //texture_creator: texture_creator,
            textures: Vec::new(),
            packers: Vec::new(),
            glyphs: HashMap::new(),
            size,
            font,
        }
    }
    // cache if necessary and return a glyph
    pub fn get(&mut self, c: char, color: Color) -> CachedGlyph {
        if !self.glyphs.contains_key(&(c, color)) {
            let s = self.font.render_char(c).blended(color).unwrap();
            self.insert(c, color, &s)
        }
        self.glyphs[&(c, color)]
    }
    // insert a new glyph in the cache
    pub fn insert(&mut self, c: char, color: Color, src: &Surface) {
        // panic if there is no texture available
        assert!(!self.textures.is_empty());
        let last = self.textures.len() - 1;
        if let Some(rect) = self.packers[last].pack(src.width() as _, src.height() as _, false) {
            let rect = rect::Rect::new(rect.x, rect.y, rect.width as _, rect.height as _);
            self.textures[last]
                .with_lock(rect, |buffer: &mut [u8], pitch: usize| {
                    // assume that the source surface and the dest texture have the same color format
                    let pixel = src.without_lock().unwrap();
                    for y in 0..src.height() {
                        for x in 0..src.pitch() {
                            buffer[(y * pitch as u32 + x) as usize] = pixel[(y * src.pitch() + x) as usize];
                        }
                    }
                }).unwrap();
            self.glyphs.insert(
                (c, color),
                CachedGlyph {
                    textureid: last as _,
                    rect,
                },
            );
        } else {
            // Grow, TODO
        }
    }
    pub fn grow<'s, 'tc: 't>(&'s mut self, texture_creator: &'tc TextureCreator<WindowContext>) {
        let mut tex = texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, self.size, self.size)
            .unwrap();
        tex.set_blend_mode(BlendMode::Blend);
        self.textures.push(tex);
        let packer = rect_packer::DensePacker::new(self.size as _, self.size as _);
        self.packers.push(packer);
    }
}
