use std::convert::From;

use sdl2::keyboard::{Keycode};

//#[derive(PartialEq,Eq,Debug,Hash)]
bitflags! {
    pub struct Mod: u16 {
        const NONE = 0;
        const CTRL = 1;
        const SHIFT = 2;
        const ALT = 4;
        const NUM = 8;
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


impl<'a> From<&'a str> for KeyBinding {
    fn from(keybinding: &'a str) -> Self {
        let args: Vec<&str> = keybinding.split("-").collect();
        let mut keymod = Mod::NONE;
        let mut keycode:Option<Keycode> = None;
        for arg in args {
            match arg.to_uppercase().as_str() {
                // Mod key
                "CTRL" => keymod |= Mod::CTRL,
                "SHIFT" => keymod |= Mod::SHIFT,
                "ALT" => keymod |= Mod::ALT,
                "NUM" => keymod |= Mod::NUM,

                code => keycode = Keycode::from_name(&code),
            }
        }
        KeyBinding::new(keycode.unwrap(),keymod)
    }
}


#[cfg(test)]
mod test {
    use std::convert::From;
    use super::KeyBinding;
    use sdl2::keyboard::{Keycode};
    use super::Mod;
    #[test]
    fn from_str() {
        assert_eq!(KeyBinding::from("Ctrl-C"), KeyBinding::new(Keycode::C, Mod::CTRL));
        assert_eq!(KeyBinding::from("Ctrl-Shift-P"), KeyBinding::new(Keycode::P, Mod::CTRL | Mod::SHIFT ));
        assert_eq!(KeyBinding::from("Ctrl-Return"), KeyBinding::new(Keycode::Return, Mod::CTRL));
    }
}
