use window::EditorWindow;
use keybinding::{KeyBinding, Mod};
use sdl2::keyboard::Keycode;
use view::{View, ViewCmd, Direction};
use window::WindowCmd;
use clipboard2::*;

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
    pub fn into_boxed<K>(
        name: &'static str,
        desc: &'static str,
        keybinding: &[K],
        execute: fn(&mut View),
    ) -> Box<Self> 
    where K: Clone, KeyBinding: From<K> {
        Box::new(GenericViewCommand::new(
            name,
            desc,
            keybinding.into_iter().cloned().map(|k| From::from(k) ).collect(),
            execute,
        ))
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

struct GenericWindowCommand {
    name: &'static str,
    desc: &'static str,
    keybinding: Vec<KeyBinding>,
    execute: fn(&mut EditorWindow),
}

impl GenericWindowCommand {
    pub fn new(
        name: &'static str,
        desc: &'static str,
        keybinding: Vec<KeyBinding>,
        execute: fn(&mut EditorWindow),
    ) -> Self {
        GenericWindowCommand {
            name,
            desc,
            keybinding,
            execute,
        }
    }
    pub fn into_boxed<K>(
        name: &'static str,
        desc: &'static str,
        keybinding: &[K],
        execute: fn(&mut EditorWindow),
    ) -> Box<Self> 
    where K: Clone, KeyBinding: From<K> {
        Box::new(GenericWindowCommand::new(
            name,
            desc,
            keybinding.into_iter().cloned().map(|k| From::from(k) ).collect(),
            execute,
        ))
    }
}

impl WindowCmd for GenericWindowCommand {
    fn name(&self) -> &'static str {
        self.name
    }
    fn desc(&self) -> &'static str {
        self.desc
    }
    fn keybinding(&self) -> Vec<KeyBinding> {
        self.keybinding.clone()
    }
    fn run(&mut self, window: &mut EditorWindow) {
        (self.execute)(window);
    }
}



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
    use SETTINGS;
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
            &["End","Shift-End"],
            |v| v.end(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Home",
            "Go to the beginning of the line",
            &["Home","Shift-Home"],
            |v| v.home(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Undo",
            "Undo the last action",
            &["Ctrl-Z"],
            |v| v.undo(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Redo",
            "Redo the last action",
            &["Ctrl-Y"],
            |v| v.redo(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Enter",
            "Insert the return char",
            &["Keypad Enter","Return"],
            |v| v.insert_char('\n'),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Tab",
            "Add a tabulation",
            &["Tab"],
            |v| {
                
                if SETTINGS.read().unwrap().get("indentWithSpace").unwrap() {
                    let n = SETTINGS.read().unwrap().get::<usize>("tabSize").unwrap();
                    let p = v.col_idx();
                    let cible = ((p + n)/n)*n;

                    for _ in 0.. cible - p {
                        v.insert_char(' ');
                    }
                } else {
                    v.insert_char('\t');
                }
            },
        ));
        v.push(GenericViewCommand::into_boxed(
            "Backspace",
            "delete the char at left  or the selection",
            &["Backspace"],
            |v| v.backspace(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Delete",
            "delete the char under the cursor or the selection",
            &["Delete"],
            |v| v.delete_at_cursor(),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Up",
            "Move cursor up",
            &["Up","Num-Up","Shift-Up","Shift-Num-Up"],
            |v| v.move_cursor(Direction::Up),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Down",
            "Move cursor down",
            &["Down","Num-Down","Shift-Down","Shift-Num-Down"],
            |v| v.move_cursor(Direction::Down),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Left",
            "Move cursor left",
            &["Left","Num-Left","Shift-Left","Shift-Num-Left"],
            |v| v.move_cursor(Direction::Left),
        ));
        v.push(GenericViewCommand::into_boxed(
            "Right",
            "Move cursor right",
            &["Right","Num-Right","Shift-Right","Shift-Num-Right"],
            |v| v.move_cursor(Direction::Right),
        ));
        v.push(GenericViewCommand::into_boxed(
            "PageUp",
            "Move page up",
            &["PageUp"],
            |v| v.move_page(Direction::Up),
        ));
        v.push(GenericViewCommand::into_boxed(
            "PageDown",
            "Move page down",
            &["PageDown"],
            |v| v.move_page(Direction::Down),
        ));
        v
    }
}

pub mod window {
    use SETTINGS;
    use commands::*;
    use window::WindowCmd;

    pub fn get_all() -> Vec<Box<WindowCmd>> {
        let mut v = Vec::<Box<WindowCmd>>::new();
        v.push(GenericWindowCommand::into_boxed(
            "Open",
            "Open an existing file",
            &["Ctrl-O"],
            |w| (println!("I should open something")),
        ));

        v
    }
}
