use keybinding::{KeyBinding, Mod};
use sdl2::keyboard::Keycode;
use view::{View, ViewCmd};
use clipboard2::*;

// struct ViewCmd2 {
//     name : &'static str,
//     desc : &'static str,
//     keybinding : KeyBinding,

// }

struct GenericViewCommand {
    name: &'static str,
    desc: &'static str,
    keybinding: Vec<KeyBinding>,
    execute: fn(&mut View),
}

impl GenericViewCommand {
    pub fn new(
        name: &'static str,
        desc: &'static str,
        keybinding: Vec<KeyBinding>,
        execute: fn(&mut View),
    ) -> Self {
        GenericViewCommand {
            name,
            desc,
            keybinding,
            execute,
        }
    }
    pub fn into_boxed(
        name: &'static str,
        desc: &'static str,
        keybinding: Vec<KeyBinding>,
        execute: fn(&mut View),
    ) -> Box<Self> {
        Box::new(GenericViewCommand {
            name,
            desc,
            keybinding,
            execute,
        })
    }
}

impl ViewCmd for GenericViewCommand {
    fn name(&self) -> &'static str {
        self.name
    }
    fn desc(&self) -> &'static str {
        self.desc
    }
    fn keybinding(&self) -> Vec<KeyBinding> {
        self.keybinding.clone()
    }
    fn run(&mut self, view: &mut View) {
        (self.execute)(view);
    }
}

// struct HomeCmd;

// impl ViewCmd for HomeCmd {
//     fn name(&self) -> &'static str {
//         "Home"
//     }
//     fn desc(&self) -> &'static str {
//         "go to the beginning of line"
//     }
//     fn keybinding(&self) -> KeyBinding {
//         KeyBinding::new(Keycode::Home, Mod::NONE)
//     }
//     fn run(&mut self, view: &mut View) {
//         view.home();
//     }
// }

// struct EndCmd;

// impl ViewCmd for EndCmd {
//     fn name(&self) -> &'static str {
//         "End"
//     }
//     fn desc(&self) -> &'static str {
//         "go to the end of line"
//     }
//     fn keybinding(&self) -> KeyBinding {
//         KeyBinding::new(Keycode::End, Mod::NONE)
//     }
//     fn run(&mut self, view: &mut View) {
//         view.end();
//     }
// }
lazy_static!{
    pub static ref CLIPBOARD : SystemClipboard = SystemClipboard::new().unwrap();
}

struct CopyCmd;
impl ViewCmd for CopyCmd {
    fn name(&self) -> &'static str {
        "Copy"
    }
    fn desc(&self) -> &'static str {
        "Copy the current selection to clipboard"
    }
    fn keybinding(&self) -> Vec<KeyBinding> {
        vec![KeyBinding::new(Keycode::C, Mod::CTRL)]
    }
    fn run(&mut self, view: &mut View) {
        if let Some(s) = view.get_selection() {
            CLIPBOARD.set_string_contents(s).unwrap();
        }
    }
}
struct PasteCmd;
impl ViewCmd for PasteCmd {
    fn name(&self) -> &'static str {
        "Paste"
    }
    fn desc(&self) -> &'static str {
        "Paste the content of clipboard"
    }
    fn keybinding(&self) -> Vec<KeyBinding> {
        vec![KeyBinding::new(Keycode::V, Mod::CTRL)]
    }
    fn run(&mut self, view: &mut View) {
        let s = CLIPBOARD.get_string_contents().unwrap();
        view.insert(&s);
    }
}

struct CutCmd;
impl ViewCmd for CutCmd {
    fn name(&self) -> &'static str {
        "Cut"
    }
    fn desc(&self) -> &'static str {
        "Cut the current selection to clipboard"
    }
    fn keybinding(&self) -> Vec<KeyBinding> {
        vec![KeyBinding::new(Keycode::X, Mod::CTRL)]
    }
    fn run(&mut self, view: &mut View) {
        if let Some(s) = view.get_selection() {
            CLIPBOARD.set_string_contents(s).unwrap();
            view.delete_at_cursor();
        }
    }
}


pub mod view {
    use commands::*;
    use view::ViewCmd;

    pub fn get_all() -> Vec<Box<ViewCmd>> {
        let mut v = Vec::<Box<ViewCmd>>::new();
        // v.push(Box::new(HomeCmd {}));
        v.push(Box::new(CopyCmd {}));
        v.push(Box::new(CutCmd {}));
        v.push(Box::new(PasteCmd {}));
        v.push(GenericViewCommand::into_boxed(
            "End",
            "Go to the end of the line",
            vec![KeyBinding::new(Keycode::End, Mod::NONE),KeyBinding::new(Keycode::End, Mod::SHIFT)],
            |v| v.end(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Home",
            "Go to the beginning of the line",
            vec![KeyBinding::new(Keycode::Home, Mod::NONE),KeyBinding::new(Keycode::Home, Mod::SHIFT)],
            |v| v.home(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Undo",
            "Undo the last action",
            vec![KeyBinding::new(Keycode::Z, Mod::CTRL)],
            |v| v.undo(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Redo",
            "Redo the last action",
            vec![KeyBinding::new(Keycode::Y, Mod::CTRL)],
            |v| v.redo(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Enter",
            "Insert the return char",
            vec![KeyBinding::new(Keycode::KpEnter, Mod::NONE),KeyBinding::new(Keycode::Return, Mod::NONE)],
            |v| v.insert_char('\n'),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Tab",
            "Add a tabulation",
            vec![KeyBinding::new(Keycode::Tab, Mod::NONE)],
            |v| v.insert("    "),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Backspace",
            "delete the char at left  or the selection",
            vec![KeyBinding::new(Keycode::Backspace, Mod::NONE)],
            |v| v.backspace(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Delete",
            "delete the char under the cursor or the selection",
            vec![KeyBinding::new(Keycode::Delete, Mod::NONE)],
            |v| v.delete_at_cursor(),
        ));
        v
    }
}
// lazy_static! {
//     pub static ref VIEW_CMDS : Vec<Box<ViewCmd>> = {
//         let mut v = Vec::<Box<ViewCmd>>::new();
//         v.push(Box::new(HomeCmd{}));
//         v
//     };
// }
