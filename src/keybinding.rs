use std::convert::From;

use bitflags::bitflags;
use glutin::VirtualKeyCode;

bitflags! {
    pub struct Mod: u16 {
        const NONE = 0;
        const CTRL = 1;
        const SHIFT = 2;
        const ALT = 4;
        const LOGO = 8;
    }
}
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct KeyBinding {
    keycode: VirtualKeyCode,
    keymod: Mod,
}
impl KeyBinding {
    pub fn new(keycode: VirtualKeyCode, keymod: Mod) -> Self {
        KeyBinding { keycode, keymod }
    }
}

impl<'a> From<&'a str> for KeyBinding {
    fn from(keybinding: &'a str) -> Self {
        let args: Vec<&str> = keybinding.split('-').collect();
        let mut keymod = Mod::NONE;
        let mut keycode: Option<VirtualKeyCode> = None;
        for arg in args {
            match arg.to_uppercase().as_str() {
                // Mod key
                "CTRL" => keymod |= Mod::CTRL,
                "SHIFT" => keymod |= Mod::SHIFT,
                "ALT" => keymod |= Mod::ALT,
                "LOGO" => keymod |= Mod::LOGO,

                // Keycode
                "KEY1" => keycode = Some(VirtualKeyCode::Key1),
                "KEY2" => keycode = Some(VirtualKeyCode::Key2),
                "KEY3" => keycode = Some(VirtualKeyCode::Key3),
                "KEY4" => keycode = Some(VirtualKeyCode::Key4),
                "KEY5" => keycode = Some(VirtualKeyCode::Key5),
                "KEY6" => keycode = Some(VirtualKeyCode::Key6),
                "KEY7" => keycode = Some(VirtualKeyCode::Key7),
                "KEY8" => keycode = Some(VirtualKeyCode::Key8),
                "KEY9" => keycode = Some(VirtualKeyCode::Key9),
                "KEY0" => keycode = Some(VirtualKeyCode::Key0),
                "A" => keycode = Some(VirtualKeyCode::A),
                "B" => keycode = Some(VirtualKeyCode::B),
                "C" => keycode = Some(VirtualKeyCode::C),
                "D" => keycode = Some(VirtualKeyCode::D),
                "E" => keycode = Some(VirtualKeyCode::E),
                "F" => keycode = Some(VirtualKeyCode::F),
                "G" => keycode = Some(VirtualKeyCode::G),
                "H" => keycode = Some(VirtualKeyCode::H),
                "I" => keycode = Some(VirtualKeyCode::I),
                "J" => keycode = Some(VirtualKeyCode::J),
                "K" => keycode = Some(VirtualKeyCode::K),
                "L" => keycode = Some(VirtualKeyCode::L),
                "M" => keycode = Some(VirtualKeyCode::M),
                "N" => keycode = Some(VirtualKeyCode::N),
                "O" => keycode = Some(VirtualKeyCode::O),
                "P" => keycode = Some(VirtualKeyCode::P),
                "Q" => keycode = Some(VirtualKeyCode::Q),
                "R" => keycode = Some(VirtualKeyCode::R),
                "S" => keycode = Some(VirtualKeyCode::S),
                "T" => keycode = Some(VirtualKeyCode::T),
                "U" => keycode = Some(VirtualKeyCode::U),
                "V" => keycode = Some(VirtualKeyCode::V),
                "W" => keycode = Some(VirtualKeyCode::W),
                "X" => keycode = Some(VirtualKeyCode::X),
                "Y" => keycode = Some(VirtualKeyCode::Y),
                "Z" => keycode = Some(VirtualKeyCode::Z),
                "ESCAPE" => keycode = Some(VirtualKeyCode::Escape),
                "F1" => keycode = Some(VirtualKeyCode::F1),
                "F2" => keycode = Some(VirtualKeyCode::F2),
                "F3" => keycode = Some(VirtualKeyCode::F3),
                "F4" => keycode = Some(VirtualKeyCode::F4),
                "F5" => keycode = Some(VirtualKeyCode::F5),
                "F6" => keycode = Some(VirtualKeyCode::F6),
                "F7" => keycode = Some(VirtualKeyCode::F7),
                "F8" => keycode = Some(VirtualKeyCode::F8),
                "F9" => keycode = Some(VirtualKeyCode::F9),
                "F10" => keycode = Some(VirtualKeyCode::F10),
                "F11" => keycode = Some(VirtualKeyCode::F11),
                "F12" => keycode = Some(VirtualKeyCode::F12),
                "F13" => keycode = Some(VirtualKeyCode::F13),
                "F14" => keycode = Some(VirtualKeyCode::F14),
                "F15" => keycode = Some(VirtualKeyCode::F15),
                "F16" => keycode = Some(VirtualKeyCode::F16),
                "F17" => keycode = Some(VirtualKeyCode::F17),
                "F18" => keycode = Some(VirtualKeyCode::F18),
                "F19" => keycode = Some(VirtualKeyCode::F19),
                "F20" => keycode = Some(VirtualKeyCode::F20),
                "F21" => keycode = Some(VirtualKeyCode::F21),
                "F22" => keycode = Some(VirtualKeyCode::F22),
                "F23" => keycode = Some(VirtualKeyCode::F23),
                "F24" => keycode = Some(VirtualKeyCode::F24),
                "SNAPSHOT" => keycode = Some(VirtualKeyCode::Snapshot),
                "SCROLL" => keycode = Some(VirtualKeyCode::Scroll),
                "PAUSE" => keycode = Some(VirtualKeyCode::Pause),
                "INSERT" => keycode = Some(VirtualKeyCode::Insert),
                "HOME" => keycode = Some(VirtualKeyCode::Home),
                "DELETE" => keycode = Some(VirtualKeyCode::Delete),
                "END" => keycode = Some(VirtualKeyCode::End),
                "PAGEDOWN" => keycode = Some(VirtualKeyCode::PageDown),
                "PAGEUP" => keycode = Some(VirtualKeyCode::PageUp),
                "LEFT" => keycode = Some(VirtualKeyCode::Left),
                "UP" => keycode = Some(VirtualKeyCode::Up),
                "RIGHT" => keycode = Some(VirtualKeyCode::Right),
                "DOWN" => keycode = Some(VirtualKeyCode::Down),
                "BACK" => keycode = Some(VirtualKeyCode::Back),
                "RETURN" => keycode = Some(VirtualKeyCode::Return),
                "SPACE" => keycode = Some(VirtualKeyCode::Space),
                "COMPOSE" => keycode = Some(VirtualKeyCode::Compose),
                "CARET" => keycode = Some(VirtualKeyCode::Caret),
                "NUMLOCK" => keycode = Some(VirtualKeyCode::Numlock),
                "NUMPAD0" => keycode = Some(VirtualKeyCode::Numpad0),
                "NUMPAD1" => keycode = Some(VirtualKeyCode::Numpad1),
                "NUMPAD2" => keycode = Some(VirtualKeyCode::Numpad2),
                "NUMPAD3" => keycode = Some(VirtualKeyCode::Numpad3),
                "NUMPAD4" => keycode = Some(VirtualKeyCode::Numpad4),
                "NUMPAD5" => keycode = Some(VirtualKeyCode::Numpad5),
                "NUMPAD6" => keycode = Some(VirtualKeyCode::Numpad6),
                "NUMPAD7" => keycode = Some(VirtualKeyCode::Numpad7),
                "NUMPAD8" => keycode = Some(VirtualKeyCode::Numpad8),
                "NUMPAD9" => keycode = Some(VirtualKeyCode::Numpad9),
                "ABNTC1" => keycode = Some(VirtualKeyCode::AbntC1),
                "ABNTC2" => keycode = Some(VirtualKeyCode::AbntC2),
                "ADD" => keycode = Some(VirtualKeyCode::Add),
                "APOSTROPHE" => keycode = Some(VirtualKeyCode::Apostrophe),
                "APPS" => keycode = Some(VirtualKeyCode::Apps),
                "AT" => keycode = Some(VirtualKeyCode::At),
                "AX" => keycode = Some(VirtualKeyCode::Ax),
                "BACKSLASH" => keycode = Some(VirtualKeyCode::Backslash),
                "CALCULATOR" => keycode = Some(VirtualKeyCode::Calculator),
                "CAPITAL" => keycode = Some(VirtualKeyCode::Capital),
                "COLON" => keycode = Some(VirtualKeyCode::Colon),
                "COMMA" => keycode = Some(VirtualKeyCode::Comma),
                "CONVERT" => keycode = Some(VirtualKeyCode::Convert),
                "DECIMAL" => keycode = Some(VirtualKeyCode::Decimal),
                "DIVIDE" => keycode = Some(VirtualKeyCode::Divide),
                "EQUALS" => keycode = Some(VirtualKeyCode::Equals),
                "GRAVE" => keycode = Some(VirtualKeyCode::Grave),
                "KANA" => keycode = Some(VirtualKeyCode::Kana),
                "KANJI" => keycode = Some(VirtualKeyCode::Kanji),
                "LALT" => keycode = Some(VirtualKeyCode::LAlt),
                "LBRACKET" => keycode = Some(VirtualKeyCode::LBracket),
                "LCONTROL" => keycode = Some(VirtualKeyCode::LControl),
                "LSHIFT" => keycode = Some(VirtualKeyCode::LShift),
                "LWIN" => keycode = Some(VirtualKeyCode::LWin),
                "MAIL" => keycode = Some(VirtualKeyCode::Mail),
                "MEDIASELECT" => keycode = Some(VirtualKeyCode::MediaSelect),
                "MEDIASTOP" => keycode = Some(VirtualKeyCode::MediaStop),
                "MINUS" => keycode = Some(VirtualKeyCode::Minus),
                "MULTIPLY" => keycode = Some(VirtualKeyCode::Multiply),
                "MUTE" => keycode = Some(VirtualKeyCode::Mute),
                "MYCOMPUTER" => keycode = Some(VirtualKeyCode::MyComputer),
                "NAVIGATEFORWARD" => keycode = Some(VirtualKeyCode::NavigateForward),
                "NAVIGATEBACKWARD" => keycode = Some(VirtualKeyCode::NavigateBackward),
                "NEXTTRACK" => keycode = Some(VirtualKeyCode::NextTrack),
                "NOCONVERT" => keycode = Some(VirtualKeyCode::NoConvert),
                "NUMPADCOMMA" => keycode = Some(VirtualKeyCode::NumpadComma),
                "NUMPADENTER" => keycode = Some(VirtualKeyCode::NumpadEnter),
                "NUMPADEQUALS" => keycode = Some(VirtualKeyCode::NumpadEquals),
                "OEM102" => keycode = Some(VirtualKeyCode::OEM102),
                "PERIOD" => keycode = Some(VirtualKeyCode::Period),
                "PLAYPAUSE" => keycode = Some(VirtualKeyCode::PlayPause),
                "POWER" => keycode = Some(VirtualKeyCode::Power),
                "PREVTRACK" => keycode = Some(VirtualKeyCode::PrevTrack),
                "RALT" => keycode = Some(VirtualKeyCode::RAlt),
                "RBRACKET" => keycode = Some(VirtualKeyCode::RBracket),
                "RCONTROL" => keycode = Some(VirtualKeyCode::RControl),
                "RSHIFT" => keycode = Some(VirtualKeyCode::RShift),
                "RWIN" => keycode = Some(VirtualKeyCode::RWin),
                "SEMICOLON" => keycode = Some(VirtualKeyCode::Semicolon),
                "SLASH" => keycode = Some(VirtualKeyCode::Slash),
                "SLEEP" => keycode = Some(VirtualKeyCode::Sleep),
                "STOP" => keycode = Some(VirtualKeyCode::Stop),
                "SUBTRACT" => keycode = Some(VirtualKeyCode::Subtract),
                "SYSRQ" => keycode = Some(VirtualKeyCode::Sysrq),
                "TAB" => keycode = Some(VirtualKeyCode::Tab),
                "UNDERLINE" => keycode = Some(VirtualKeyCode::Underline),
                "UNLABELED" => keycode = Some(VirtualKeyCode::Unlabeled),
                "VOLUMEDOWN" => keycode = Some(VirtualKeyCode::VolumeDown),
                "VOLUMEUP" => keycode = Some(VirtualKeyCode::VolumeUp),
                "WAKE" => keycode = Some(VirtualKeyCode::Wake),
                "WEBBACK" => keycode = Some(VirtualKeyCode::WebBack),
                "WEBFAVORITES" => keycode = Some(VirtualKeyCode::WebFavorites),
                "WEBFORWARD" => keycode = Some(VirtualKeyCode::WebForward),
                "WEBHOME" => keycode = Some(VirtualKeyCode::WebHome),
                "WEBREFRESH" => keycode = Some(VirtualKeyCode::WebRefresh),
                "WEBSEARCH" => keycode = Some(VirtualKeyCode::WebSearch),
                "WEBSTOP" => keycode = Some(VirtualKeyCode::WebStop),
                "YEN" => keycode = Some(VirtualKeyCode::Yen),
                "COPY" => keycode = Some(VirtualKeyCode::Copy),
                "PASTE" => keycode = Some(VirtualKeyCode::Paste),
                "CUT" => keycode = Some(VirtualKeyCode::Cut),
                _ => unimplemented!("{}", arg),
            }
        }
        KeyBinding::new(keycode.unwrap(), keymod)
    }
}

#[cfg(test)]
mod test {
    use super::KeyBinding;
    use super::Mod;
    use glutin::VirtualKeyCode;
    use std::convert::From;
    #[test]
    fn from_str() {
        assert_eq!(
            KeyBinding::from("Ctrl-C"),
            KeyBinding::new(VirtualKeyCode::C, Mod::CTRL)
        );
        assert_eq!(
            KeyBinding::from("Ctrl-Shift-P"),
            KeyBinding::new(VirtualKeyCode::P, Mod::CTRL | Mod::SHIFT)
        );
        assert_eq!(
            KeyBinding::from("Ctrl-Return"),
            KeyBinding::new(VirtualKeyCode::Return, Mod::CTRL)
        );
    }
}
