use clipboard2::*;
use crate::keybinding::KeyBinding;
use crate::view::{Direction, View, ViewCmd};
use crate::window::EditorWindow;
use crate::window::WindowCmd;

struct GenericViewCommand {
    name: &'static str,
    desc: &'static str,
    keybinding: Vec<KeyBinding>,
    execute: fn(&mut View),
}

impl GenericViewCommand {
    pub fn new(name: &'static str, desc: &'static str, keybinding: Vec<KeyBinding>, execute: fn(&mut View)) -> Self {
        GenericViewCommand {
            name,
            desc,
            keybinding,
            execute,
        }
    }
    pub fn new_box<K>(name: &'static str, desc: &'static str, keybinding: &[K], execute: fn(&mut View)) -> Box<Self>
    where
        K: Clone,
        KeyBinding: From<K>,
    {
        Box::new(GenericViewCommand::new(
            name,
            desc,
            keybinding.into_iter().cloned().map(From::from).collect(),
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
    pub fn new_box<K>(
        name: &'static str,
        desc: &'static str,
        keybinding: &[K],
        execute: fn(&mut EditorWindow),
    ) -> Box<Self>
    where
        K: Clone,
        KeyBinding: From<K>,
    {
        Box::new(GenericWindowCommand::new(
            name,
            desc,
            keybinding.into_iter().cloned().map(From::from).collect(),
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

lazy_static! {
    pub static ref CLIPBOARD: SystemClipboard = SystemClipboard::new().unwrap();
}

pub mod view {
    use crate::commands::*;
    use crate::view::ViewCmd;
    use crate::SETTINGS;

    pub fn get_all() -> Vec<Box<ViewCmd>> {
        let mut v = Vec::<Box<ViewCmd>>::new();
        v.push(GenericViewCommand::new_box(
            "Cut",
            "Cut the current selection to clipboard",
            &["Ctrl-X"],
            |v| {
                if let Some(s) = v.get_selection() {
                    CLIPBOARD.set_string_contents(s).unwrap();
                    v.delete_at_cursor();
                }
            },
        ));
        v.push(GenericViewCommand::new_box(
            "Copy",
            "Copy the current selection to clipboard",
            &["Ctrl-C"],
            |v| {
                if let Some(s) = v.get_selection() {
                    CLIPBOARD.set_string_contents(s).unwrap();
                }
            },
        ));
        v.push(GenericViewCommand::new_box(
            "Paste",
            "Paste the content of clipboard",
            &["Ctrl-V"],
            |v| {
                let s = CLIPBOARD.get_string_contents().unwrap();
                v.insert(&s);
            },
        ));
        v.push(GenericViewCommand::new_box(
            "End",
            "Go to the end of the line",
            &["End", "Shift-End"],
            |v| v.end(false),
        ));
        v.push(GenericViewCommand::new_box(
            "EndSel",
            "Go to the end of the line expanding the selection",
            &["Shift-End"],
            |v| v.end(true),
        ));
        v.push(GenericViewCommand::new_box(
            "Home",
            "Go to the beginning of the line",
            &["Home"],
            |v| v.home(false),
        ));
        v.push(GenericViewCommand::new_box(
            "HomeSel",
            "Go to the beginning of the line expanding the selection",
            &["Shift-Home"],
            |v| v.home(true),
        ));
        v.push(GenericViewCommand::new_box(
            "Undo",
            "Undo the last action",
            &["Ctrl-Z"],
            |v| v.undo(),
        ));
        v.push(GenericViewCommand::new_box(
            "Redo",
            "Redo the last action",
            &["Ctrl-Y"],
            |v| v.redo(),
        ));
        v.push(GenericViewCommand::new_box(
            "Enter",
            "Insert the return char",
            &["Keypad Enter", "Return"],
            |v| v.insert_linefeed(),
        ));
        v.push(GenericViewCommand::new_box("Tab", "Add a tabulation", &["Tab"], |v| {
            if SETTINGS.read().unwrap().get("indentWithSpace").unwrap() {
                let n = SETTINGS.read().unwrap().get::<usize>("tabSize").unwrap();
                let p = v.col_idx();
                let cible = ((p + n) / n) * n;

                for _ in 0..cible - p {
                    v.insert_char(' ');
                }
            } else {
                v.insert_char('\t');
            }
        }));
        v.push(GenericViewCommand::new_box(
            "Backspace",
            "delete the char at left  or the selection",
            &["Backspace"],
            |v| v.backspace(),
        ));
        v.push(GenericViewCommand::new_box(
            "Delete",
            "delete the char under the cursor or the selection",
            &["Delete"],
            |v| v.delete_at_cursor(),
        ));
        v.push(GenericViewCommand::new_box(
            "Up",
            "Move cursor up",
            &["Up", "Num-Up"],
            |v| v.move_cursor(Direction::Up, false),
        ));
        v.push(GenericViewCommand::new_box(
            "Down",
            "Move cursor down",
            &["Down", "Num-Down"],
            |v| v.move_cursor(Direction::Down, false),
        ));
        v.push(GenericViewCommand::new_box(
            "Left",
            "Move cursor left",
            &["Left", "Num-Left"],
            |v| v.move_cursor(Direction::Left, false),
        ));
        v.push(GenericViewCommand::new_box(
            "Right",
            "Move cursor right",
            &["Right", "Num-Right"],
            |v| v.move_cursor(Direction::Right, false),
        ));

        v.push(GenericViewCommand::new_box(
            "UpSel",
            "Move cursor up expanding selection",
            &["Shift-Up", "Shift-Num-Up"],
            |v| v.move_cursor(Direction::Up, true),
        ));
        v.push(GenericViewCommand::new_box(
            "DownSel",
            "Move cursor down expanding selection",
            &["Shift-Down", "Shift-Num-Down"],
            |v| v.move_cursor(Direction::Down, true),
        ));
        v.push(GenericViewCommand::new_box(
            "LeftSel",
            "Move cursor left expanding selection",
            &["Shift-Left", "Shift-Num-Left"],
            |v| v.move_cursor(Direction::Left, true),
        ));
        v.push(GenericViewCommand::new_box(
            "RightSel",
            "Move cursor right expanding selection",
            &["Shift-Right", "Shift-Num-Right"],
            |v| v.move_cursor(Direction::Right, true),
        ));

        v.push(GenericViewCommand::new_box(
            "PageUp",
            "Move page up",
            &["PageUp"],
            |v| v.move_page(Direction::Up, false),
        ));
        v.push(GenericViewCommand::new_box(
            "PageDown",
            "Move page down",
            &["PageDown"],
            |v| v.move_page(Direction::Down, false),
        ));

        v.push(GenericViewCommand::new_box(
            "PageUpSel",
            "Move page up expanding selection",
            &["Shift-PageUp"],
            |v| v.move_page(Direction::Up, true),
        ));
        v.push(GenericViewCommand::new_box(
            "PageDownSel",
            "Move page down expanding selection",
            &["Shift-PageDown"],
            |v| v.move_page(Direction::Down, true),
        ));
        v.push(GenericViewCommand::new_box("Save", "Save file", &["Ctrl-S"], |v| {
            v.save();
        }));
        v
    }
}

pub mod window {
    use crate::commands::*;
    use nfd;
    use crate::window::WindowCmd;

    pub fn get_all() -> Vec<Box<WindowCmd>> {
        let mut v = Vec::<Box<WindowCmd>>::new();
        v.push(GenericWindowCommand::new_box(
            "Open",
            "Open an existing file",
            &["Ctrl-O"],
            |w| {
                if let Ok(nfd::Response::Okay(file)) = nfd::open_file_dialog(None, None) {
                    w.add_new_view(Some(file))
                }
            },
        ));
        v
    }
}
