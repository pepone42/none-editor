// extern crate gl;
// extern crate nanovg;
extern crate ropey;
extern crate sdl2;
extern crate num;
extern crate clipboard2;
extern crate rect_packer;

mod buffer;
mod view;
mod window;
mod fontcache;

use std::env;

fn main() {
    let width = 800;
    let height = 600;

    window::start(width, height,env::args().nth(1));
}
