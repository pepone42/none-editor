extern crate ropey;
extern crate sdl2;
extern crate num;
extern crate clipboard2;
extern crate rect_packer;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;


mod buffer;
mod view;
mod window;
mod fontcache;
mod commands;
mod keybinding;

use std::env;

fn main() {
    let width = 800;
    let height = 600;

    window::start(width, height,env::args().nth(1));
}
