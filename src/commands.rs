use keybinding::{KeyBinding, Mod};
use sdl2::keyboard::Keycode;
use view::{View, ViewCmd};

// struct ViewCmd2 {
//     name : &'static str,
//     desc : &'static str,
//     keybinding : KeyBinding,

// }

struct GenericViewCommand {
    name: &'static str,
    desc: &'static str,
    keybinding: KeyBinding,
    execute: fn(&mut View),
}

impl GenericViewCommand {
    pub fn new(
        name: &'static str,
        desc: &'static str,
        keybinding: KeyBinding,
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
        keybinding: KeyBinding,
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
    fn keybinding(&self) -> KeyBinding {
        self.keybinding
    }
    fn run(&mut self, view: &mut View) {
        (self.execute)(view);
    }
}

struct HomeCmd;

impl ViewCmd for HomeCmd {
    fn name(&self) -> &'static str {
        "Home"
    }
    fn desc(&self) -> &'static str {
        "go to the beginning of line"
    }
    fn keybinding(&self) -> KeyBinding {
        KeyBinding::new(Keycode::Home, Mod::NONE)
    }
    fn run(&mut self, view: &mut View) {
        view.home();
    }
}

struct EndCmd;

impl ViewCmd for EndCmd {
    fn name(&self) -> &'static str {
        "End"
    }
    fn desc(&self) -> &'static str {
        "go to the end of line"
    }
    fn keybinding(&self) -> KeyBinding {
        KeyBinding::new(Keycode::End, Mod::NONE)
    }
    fn run(&mut self, view: &mut View) {
        view.end();
    }
}

pub mod view {
    use commands::*;
    use view::ViewCmd;

    pub fn get_all() -> Vec<Box<ViewCmd>> {
        let mut v = Vec::<Box<ViewCmd>>::new();
        // v.push(Box::new(HomeCmd {}));
        // v.push(Box::new(EndCmd {}));
        v.push(GenericViewCommand::into_boxed(
            "End",
            "Go to the end of the line",
            KeyBinding::new(Keycode::End, Mod::NONE),
            |v| v.end(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Home",
            "Go to the beginning of the line",
            KeyBinding::new(Keycode::Home, Mod::NONE),
            |v| v.home(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Undo",
            "Undo the last Action",
            KeyBinding::new(Keycode::Z, Mod::CTRL),
            |v| v.undo(),
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
