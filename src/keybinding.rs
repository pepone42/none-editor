use sdl2::keyboard::{Keycode};

//#[derive(PartialEq,Eq,Debug,Hash)]
bitflags! {
    pub struct Mod: u16 {
        const NONE = 0;
        const CTRL = 1;
        const SHIFT = 2;
        const ALT = 4;
    }
}
#[derive(PartialEq,Eq,Debug,Hash,Clone,Copy)]
pub struct KeyBinding {
    keycode: Keycode,
    keymod: Mod,
}
impl KeyBinding {
    pub fn new(keycode: Keycode, keymod: Mod) -> Self {
        KeyBinding {keycode,keymod}
    }
}
// pub struct ParseKeyBindingError;


// impl FromStr for KeyBinding {
//     type Err = ParseKeyBindingError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let args: Vec<&str> = s.split("-").collect();


//         let x_fromstr = coords[0].parse::<i32>()?;
//         let y_fromstr = coords[1].parse::<i32>()?;

//         Ok(Point { x: x_fromstr, y: y_fromstr })
//     }
// }