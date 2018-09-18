extern crate clipboard2;
extern crate num;
extern crate rect_packer;
extern crate ropey;
extern crate sdl2;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate config;
extern crate directories;
extern crate syntect;
extern crate nfd;
extern crate chardet;
extern crate encoding;

mod buffer;
mod canvas;
mod commands;
mod fontcache;
mod keybinding;
mod view;
mod window;

use config::Config;
use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::RwLock;
use syntect::parsing::SyntaxSet;

use directories::ProjectDirs;

lazy_static! {
    pub static ref SETTINGS: RwLock<Config> = RwLock::new({
        let mut conf = Config::default();

        let default = include_str!("config/default.json");
        conf.merge(config::File::from_str(default, config::FileFormat::Json))
            .unwrap();

        let user_dir = ProjectDirs::from("com", "pepone42", "nonedit").unwrap();
        let mut user_config_file = PathBuf::from(user_dir.config_dir());
        user_config_file.push("setting.json");
        if let Ok(mut f) = fs::File::open(user_config_file) {
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();
            conf.merge(config::File::from_str(&contents, config::FileFormat::Json))
                .unwrap();
        }

        conf
    });
}
thread_local! {
    pub static SYNTAXSET: SyntaxSet = SyntaxSet::load_defaults_nonewlines();
}

fn main() {
    window::start(env::args().nth(1));
}
