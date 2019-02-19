mod buffer;
mod commands;
mod cursor;
mod keybinding;
mod styling;
mod system;
mod view;
mod widget;
mod window;
mod palette;

use config;
use config::Config;
use lazy_static::lazy_static;
use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::RwLock;

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

fn main() {
    window::start(env::args().nth(1));
}
