use crate::styling::STYLE;
use crate::system::Canvas;
use crate::window::{Geometry, EventResult};
use glutin::WindowEvent;
use nanovg::Color;
use syntect::highlighting;

// pub enum PaletteEventResult {
//     Redraw,
//     Continue,
//     Cancel,
//     Ok(usize),
// }

pub struct Palette {
    input: String,
    list: Vec<String>,
    selected_item: usize,
}
impl Palette {
    pub fn new(input: Vec<String>) -> Self {
        Palette {
            input: String::new(),
            list: input,
            selected_item: Default::default(),
        }
    }
    pub fn relayout(&mut self, geometry: Geometry, _canvas: &Canvas) {}
    pub fn min_size(&self, canvas: &Canvas) -> (f32, f32) {
        (100.0, 100.0)
    }
    pub fn draw(&self, canvas: &mut Canvas) {
        let fg = STYLE.theme.settings.foreground.unwrap_or(highlighting::Color::WHITE);
        let bg = STYLE.theme.settings.background.unwrap_or(highlighting::Color::WHITE);
        canvas.set_color(Color::from_rgb(bg.r, bg.g, bg.b));
        canvas.move_to(0.0,0.0);
        canvas.draw_rect(100.0,100.0);
        canvas.move_to(1.0 , canvas.fonts["mono"].line_height + canvas.fonts["mono"].descender );
        canvas.set_color(Color::from_rgb(fg.r, fg.g, fg.b));
        canvas.draw_str(&self.input);
    }
    pub fn process_event(&mut self, event: &WindowEvent) -> EventResult<usize> {
        use glutin::WindowEvent::*;
        match *event {
            ReceivedCharacter(c) => {
                self.input.push(c);
                EventResult::Redraw
            }
            KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(k),
                        ..
                    },
                ..
            } => {
                use glutin::VirtualKeyCode::*;
                match k {
                    Back => {
                        self.input.pop();
                        EventResult::Redraw
                    }
                    Return | NumpadEnter => EventResult::Ok(self.selected_item),
                    Escape => (EventResult::Cancel),
                    _ => (EventResult::Continue),
                }
            }
            _ => (EventResult::Continue),
        }
    }
}
