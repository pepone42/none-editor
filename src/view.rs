use std::cell::RefCell;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Range;
use std::ops::SubAssign;
use std::rc::Rc;
use std::io;

use SYNTAXSET;

use syntect::easy::HighlightLines;
use syntect::highlighting;
use syntect::highlighting::{Style, Theme};

use buffer::Buffer;
use canvas::{Color, Screen};
use keybinding::KeyBinding;
use SETTINGS;

#[derive(Debug, Clone, Copy)]
pub enum Indentation {
    Tab,
    Space(u32),
    Mixed(u32),
    Unknow,
}

#[derive(Debug, Clone, Copy)]
pub enum LineFeed {
    CR,
    LF,
    CRLF
}


#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug)]
struct State {
    buffer: Buffer,
    cursor: Cursor,
}
#[derive(Debug)]
struct UndoStack {
    stack: Vec<State>,
    index: usize,
}
impl UndoStack {
    pub fn new() -> Self {
        UndoStack {
            stack: Vec::new(),
            index: 0,
        }
    }
    pub fn is_on_top(&self) -> bool {
        self.index == self.stack.len()
    }
    pub fn push(&mut self, state: &State) {
        self.stack.truncate(self.index);
        self.stack.push(state.clone());
        self.index += 1;
    }
    pub fn push_only(&mut self, state: &State) {
        self.stack.push(state.clone());
    }
    pub fn undo(&mut self) -> Option<State> {
        if self.index == 0 {
            println!("undo stack empty");
            None
        } else {
            self.index -= 1;
            Some(self.stack[self.index].clone())
        }
    }
    pub fn redo(&mut self) -> Option<State> {
        if self.index >= self.stack.len() - 1 {
            println!("undo stack empty [redo]");
            None
        } else {
            self.index += 1;
            let r = self.stack[self.index].clone();
            Some(r)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Selection {
    start: usize,
    end: usize,
}

impl Selection {
    fn new(start: usize, end: usize) -> Self {
        Selection { start, end }
    }
    fn contains(&self, index: usize) -> bool {
        if self.start <= self.end {
            self.start <= index && self.end > index
        } else {
            self.end <= index && self.start > index
        }
    }
    fn expand(&mut self, index: usize) {
        self.end = index;
    }
    fn lower(&self) -> usize {
        use std::cmp::min;
        min(self.start,self.end)
    }
}

impl Into<Range<usize>> for Selection {
    fn into(self) -> Range<usize> {
        if self.start <= self.end {
            Range {
                start: self.start,
                end: self.end,
            }
        } else {
            Range {
                start: self.end,
                end: self.start,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Cursor {
    index: usize,
    previous: usize,
}

impl Cursor {
    fn set(&mut self, index: usize) {
        self.previous = self.index;
        self.index = index;
    }
}

impl Add<usize> for Cursor {
    type Output = Cursor;
    fn add(self, other: usize) -> Cursor {
        Cursor {
            index: self.index + other,
            previous: self.index,
        }
    }
}

impl AddAssign<usize> for Cursor {
    fn add_assign(&mut self, other: usize) {
        self.previous = self.index;
        self.index = self.index + other;
    }
}

impl SubAssign<usize> for Cursor {
    fn sub_assign(&mut self, other: usize) {
        self.previous = self.index;
        self.index = self.index - other;
    }
}

#[derive(Debug)]
pub struct View {
    buffer: Rc<RefCell<Buffer>>,
    cursor: Cursor,
    first_visible_line: usize,
    first_visible_char: usize,
    selection: Option<Selection>,
    // in_selection: bool,
    // selection_start: usize,
    undo_stack: UndoStack,
    page_length: usize,
    syntax: String,
    linefeed: LineFeed,
}

impl View {
    /// Create a new View for the given buffer
    pub fn new(buffer: Rc<RefCell<Buffer>>) -> Self {

        let mut v = View {
            buffer,
            cursor: Cursor::default(),
            first_visible_line: 0,
            first_visible_char: 0,
            selection: None,
            // in_selection: false,
            // selection_start: 0,
            undo_stack: UndoStack::new(),
            page_length: 0,
            syntax: "Plain Text".to_owned(),
            linefeed: LineFeed::LF,
        };
        v.detect_linefeed();
        v
    }

    pub fn save(&mut self) -> io::Result<()> {
        {
            let mut b = self.buffer.borrow_mut();
            if b.get_filename().is_some() {
                b.save()?;
            } else {
                use nfd;
                if let Ok(nfd::Response::Okay(file)) = nfd::open_save_dialog(None, None) {
                    b.save_as(file)?;
                }
            }
        }
        self.detect_syntax();
        Ok(())
    }

    /// set the page length of the view
    pub fn set_page_length(&mut self, page_length: usize) {
        self.page_length = page_length;
    }
    pub fn page_length(&self) -> usize {
        self.page_length
    }

    fn get_state(&self) -> State {
        State {
            buffer: self.buffer.borrow().clone(),
            cursor: self.cursor,
        }
    }

    fn push_state(&mut self) {
        let state = self.get_state();
        self.undo_stack.push(&state);
    }

    /// detect language from extension
    pub fn detect_syntax(&mut self) {
        let b = self.buffer.borrow();
        self.syntax = b
            .get_filename()
            .and_then(|f| f.extension())
            .and_then(|e| e.to_str())
            .and_then(|e| SYNTAXSET.with(|s| s.find_syntax_by_extension(e).map(|sd| sd.name.clone())))
            .unwrap_or_else(|| "Plain Text".to_owned());
    }

    /// get the current syntax
    pub fn get_syntax(&self) -> &str {
        &self.syntax
    }

    /// get the buffer encoding
    pub fn get_encoding(&self) -> &str {
        self.buffer.borrow().get_encoding().name()
    }

    /// insert the given char at the cursor position
    pub fn insert_char(&mut self, ch: char) {
        self.push_state();
        {
            let mut b = self.buffer.borrow_mut();
            if let Some(r) = self.selection {
                self.cursor.set(r.lower());
                b.remove(r);
            }
            b.insert_char(self.cursor.index, ch);
        } // unborrow buffer
        self.cursor_right();
        self.clear_selection();
        self.focus_on_cursor();
    }

    pub fn insert_linefeed(&mut self) {
        match self.linefeed {
            LineFeed::CRLF => self.insert("\r\n"),
            LineFeed::CR => self.insert_char('\r'),
            LineFeed::LF => self.insert_char('\n'),
        }
    }

    /// insert the given string at the cursor position
    pub fn insert(&mut self, text: &str) {
        self.push_state();
        {
            let mut b = self.buffer.borrow_mut();
            if let Some(r) = self.selection {
                self.cursor.set(r.lower());
                b.remove(r);
            }
            b.insert(self.cursor.index, &text);
        } // unborrow buffer
        self.cursor += text.chars().count();
        self.clear_selection();
        self.focus_on_cursor();
    }

    /// delete the charater directly to the left of cursor
    pub fn backspace(&mut self) {
        self.push_state();
        if let Some(r) = self.selection {
            let mut b = self.buffer.borrow_mut();
            self.cursor.set(r.lower());
            b.remove(r);
        } else if self.cursor.index > 0 {
            self.cursor_left();
            let mut b = self.buffer.borrow_mut();
            b.remove(self.cursor.index..self.cursor.index + 1);
        }
        self.clear_selection();
        self.focus_on_cursor();
    }

    /// delete the charater under the cursor
    pub fn delete_at_cursor(&mut self) {
        self.push_state();
        {
            let mut b = self.buffer.borrow_mut();
            if let Some(r) = self.selection {
                self.cursor.set(r.lower());
                b.remove(r);
            } else if self.cursor.index < b.len_chars() {
                b.remove(self.cursor.index..self.cursor.index + 1);
            }
        }
        self.clear_selection();
        self.focus_on_cursor();
    }

    /// return a newly allocated string of the buffer
    pub fn to_string(&self) -> String {
        self.buffer.borrow().to_string()
    }

    /// undo the last action
    pub fn undo(&mut self) {
        if self.undo_stack.is_on_top() && !self.undo_stack.stack.is_empty() {
            // push the current state in case we redo
            let st = self.get_state();
            self.undo_stack.push_only(&st);
        }
        if let Some(state) = self.undo_stack.undo() {
            self.buffer.replace(state.buffer);
            self.cursor = state.cursor;
        }
        self.focus_on_cursor();
    }

    /// redo the last undo action
    pub fn redo(&mut self) {
        if let Some(state) = self.undo_stack.redo() {
            self.buffer.replace(state.buffer);
            self.cursor = state.cursor;
        }
        self.focus_on_cursor();
    }

    /// return the currently selection
    pub fn get_selection(&self) -> Option<String> {
        match self.selection {
            None => None,
            Some(s) => Some(self.buffer.borrow().slice(s).to_string()),
        }
    }

    /// return the cursor position in line
    pub fn line_idx(&self) -> usize {
        let b = self.buffer.borrow();
        let (l, _) = b.index_to_point(self.cursor.index);
        l
    }

    /// return the cursor position in column
    pub fn col_idx(&self) -> usize {
        let b = self.buffer.borrow();
        let (_, c) = b.index_to_point(self.cursor.index);
        c
    }

    /// return the cursor position in line,col corrdinate
    pub fn cursor_as_point(&self) -> (usize, usize) {
        let b = self.buffer.borrow();
        b.index_to_point(self.cursor.index)
    }

    fn cursor_up(&mut self) {
        let b = self.buffer.borrow();
        let (mut l, c) = b.index_to_point(self.cursor.index);
        if l > 0 {
            l -= 1
        };
        self.cursor.set(b.point_to_index((l, c)));
    }
    fn cursor_down(&mut self) {
        let b = self.buffer.borrow();
        let (mut l, c) = b.index_to_point(self.cursor.index);
        if l < b.len_lines() - 1 {
            l += 1
        };
        self.cursor.set(b.point_to_index((l, c)));
    }
    fn cursor_left(&mut self) {
        let b = self.buffer.borrow();
        if self.cursor.index > 0 {
            let line_idx = b.char_to_line(self.cursor.index);
            let line_idx_char = b.line_to_char(line_idx);
            // handle crlf and lf
            if self.cursor.index == line_idx_char {
                self.cursor.set(b.line_to_last_char(line_idx - 1));
            } else {
                self.cursor -= 1;
            }
        }
    }
    fn cursor_right(&mut self) {
        let b = self.buffer.borrow();
        if self.cursor.index < b.len_chars() {
            let line_idx = b.char_to_line(self.cursor.index);
            let line_idx_char = b.line_to_last_char(line_idx);
            // handle crlf and lf
            if self.cursor.index == line_idx_char {
                self.cursor.set(b.line_to_char(line_idx + 1));
            } else {
                self.cursor += 1;
            }
        }
    }

    /// move the cursor in the given direction
    pub fn move_cursor(&mut self, dir: Direction, expand_selection: bool) {
        match dir {
            Direction::Up => self.cursor_up(),
            Direction::Down => self.cursor_down(),
            Direction::Right => self.cursor_right(),
            Direction::Left => self.cursor_left(),
        }
        if expand_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
        self.focus_on_cursor();
    }

    /// move one page in the given direction
    pub fn move_page(&mut self, dir: Direction, expand_selection: bool) {
        for _ in 0..self.page_length {
            self.move_cursor(dir, expand_selection);
        }
        self.focus_on_cursor();
    }

    /// put the cursor at the begining of the line
    pub fn home(&mut self, expand_selection: bool) {
        let l = self.line_idx();
        self.cursor.set(self.buffer.borrow().line_to_char(l));
        if expand_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
    }

    /// put the cursor at the end of the line
    pub fn end(&mut self, expand_selection: bool) {
        let l = self.line_idx();
        self.cursor.set(self.buffer.borrow().line_to_last_char(l));
        if expand_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
    }

    /// return the cursor position in number of chars from the begining of the buffer
    pub fn index(&self) -> usize {
        self.cursor.index
    }

    /// put the cursor at the given position
    pub fn set_index(&mut self, idx: usize) {
        assert!(idx <= self.buffer.borrow().len_chars());
        self.cursor.set(idx);
    }

    /// the first visible line in the view
    pub fn first_visible_line(&self) -> usize {
        self.first_visible_line
    }

    pub fn detect_linefeed(&mut self) {
        #[cfg(target_os = "windows")]
        let linefeed = LineFeed::CRLF;
        #[cfg(not(target_os = "windows"))]
        let linefeed = LineFeed::LF;

        let b = self.buffer.borrow();
        if b.len_chars() == 0 {
            self.linefeed = linefeed;
            return;
        }
        
        let mut cr = 0;
        let mut lf = 0;
        let mut crlf = 0;
        
        let mut chars = b.chars().take(1000);
        while let Some(c) = chars.next() {
            if c == '\r' {
                if let Some(c2) = chars.next() {
                    if c2 == '\n' {
                        crlf += 1;
                    } else {
                        cr +=1;
                    }
                }
            } else if c == '\n' {
                lf+=1;
            }
        }
        
        self.linefeed = if cr>crlf && cr>lf {
            LineFeed::CR
        }
        else if lf>crlf && lf>cr {
            LineFeed::LF
        }
        else {
            LineFeed::CRLF
        }
    }

    pub fn detect_indentation(&self) -> Indentation {
        let b = self.buffer.borrow();
        let mut tab = 0;
        let mut spaces = Vec::<u32>::new();
        let tab_width = 0;
        let mut contigus_space = 0;

        fn gcd(a: u32, b: u32) -> u32 {
            if b == 0 {
                b
            } else {
                gcd(b, a % b)
            }
        }

        for l in b.lines().take(100) {
            for c in l.chars() {
                println!("{}", c);
                match c {
                    '\t' => {
                        tab += 1;
                        break;
                    }
                    ' ' => contigus_space += 1,
                    _ => {
                        if contigus_space > 0 {}
                        spaces.push(contigus_space);
                        contigus_space = 0;
                    }
                }
            }
        }
        Indentation::Space(4)
    }

    /// move the view so that the cursor is visible
    pub fn focus_on_cursor(&mut self) {
        use std::cmp::min;
        let l = self.line_idx();
        if l < self.first_visible_line {
            self.first_visible_line = l;
        }
        if l > self.first_visible_line + self.page_length {
            self.first_visible_line = l - self.page_length;
        }
        let b = self.buffer.borrow();
        self.first_visible_line = min(self.first_visible_line, b.len_lines());
    }

    pub fn draw(&self, screen: &mut Screen, theme: &Theme, x: i32, y: i32, w: u32, h: u32) {
        let mut y = 0;
        let mut x = 0;

        let adv = screen.find_glyph_metrics("mono", ' ').unwrap().advance;
        let line_spacing = screen.get_font_metrics("mono").line_spacing;
        let tabsize: i32 = SETTINGS.read().unwrap().get("tabSize").unwrap();

        let first_visible_line = self.first_visible_line();
        let last_visible_line = first_visible_line + self.page_length();

        screen.set_font("mono");
        SYNTAXSET.with(|s| {
            let synthax_definition = s.find_syntax_by_name(&self.syntax).unwrap();

            let mut highlighter = HighlightLines::new(synthax_definition, theme);

            let mut current_col = 0;
            for (line_index, l) in self.buffer.borrow().lines().take(last_visible_line).enumerate() {
                let line = l.to_string(); // TODO: optimize
                let ranges: Vec<(Style, &str)> = highlighter.highlight(&line);

                if line_index >= first_visible_line {
                    let mut idx = self.buffer.borrow().line_to_char(line_index);

                    for (style, text) in ranges {
                        let fg = Color::RGB(style.foreground.r, style.foreground.g, style.foreground.b);
                        for c in text.chars() {
                            match self.selection {
                                Some(sel) if sel.contains(idx) => {
                                    let color = theme.settings.selection.unwrap_or(highlighting::Color::WHITE);
                                    screen.set_color(Color::RGB(color.r, color.g, color.b));
                                    screen.move_to(x, y);
                                    screen.draw_rect(adv as _, line_spacing as _);
                                }
                                _ => (),
                            }
                            match c {
                                '\t' => {
                                    let nbspace = ((current_col + tabsize) / tabsize) * tabsize;
                                    current_col = nbspace;
                                    x = adv * nbspace;
                                }
                                '\0' => (),
                                '\r' => (), //idx -= 1,
                                '\n' => (),
                                // Bom hiding. TODO: rework
                                '\u{feff}' | '\u{fffe}' => (),
                                _ => {
                                    screen.move_to(x, y);
                                    screen.set_color(fg);
                                    screen.draw_char(c);
                                    x += adv;
                                    current_col += 1;
                                }
                            }
                            idx += 1;
                        }
                    }
                    y += line_spacing;
                    x = 0;
                    current_col = 0;
                }
            }
        });

        // Cursor
        let fg = theme.settings.caret.unwrap_or(highlighting::Color::WHITE);
        let (mut line, col) = self.cursor_as_point();
        line -= first_visible_line;
        screen.move_to(col as i32 * adv, line as i32 * line_spacing);
        screen.set_color(Color::RGB(fg.r, fg.g, fg.b));
        screen.draw_rect(2, line_spacing as _);
    }

    /// clear the current selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
    fn expand_selection(&mut self) {
        self.selection = if let Some(mut selection) = self.selection {
            selection.expand(self.cursor.index);
            Some(selection)
        } else {
            Some(Selection::new(self.cursor.previous, self.cursor.index))
        }
    }
}

pub trait ViewCmd {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn keybinding(&self) -> Vec<KeyBinding>;
    fn run(&mut self, &mut View);
}

#[cfg(test)]
mod tests {
    use buffer::Buffer;
    use std::cell::RefCell;
    use std::rc::Rc;
    use view::View;

    #[test]
    fn new_view() {
        let b = Rc::new(RefCell::new(Buffer::new()));
        let v = View::new(b);
    }
    #[test]
    fn insert() {
        let b = Rc::new(RefCell::new(Buffer::from_str("text")));
        let mut v = View::new(b);
        v.insert_char('r');
        assert_eq!(v.to_string(), "rtext");
        v.insert_char('e');
        assert_eq!(v.to_string(), "retext");
        v.set_index(6);
        v.insert_char('e');
        assert_eq!(v.to_string(), "retexte");
        v.insert_char('f');
        assert_eq!(v.to_string(), "retextef");
    }
    #[test]
    fn multiple_view() {
        let buf = Rc::new(RefCell::new(Buffer::from_str("text")));
        let mut v1 = View::new(buf.clone());
        let mut v2 = View::new(buf.clone());
        v1.insert_char('r');
        assert_eq!(v2.to_string(), "rtext");
        v2.insert_char('e');
        assert_eq!(v1.to_string(), "ertext");
    }

    #[test]
    #[should_panic]
    fn set_index_oob() {
        let b = Rc::new(RefCell::new(Buffer::from_str("text")));
        let mut v = View::new(b);
        v.set_index(5);
    }

    #[test]
    fn cursor_up() {
        let b = Rc::new(RefCell::new(Buffer::from_str(
            "text\nhello\nme!\nan other very long line",
        )));
        let mut v = View::new(b);
        v.set_index(11);
        v.cursor_up();
        assert_eq!(v.index(), 5);
        v.cursor_up();
        assert_eq!(v.index(), 0);
        v.cursor_up();
        assert_eq!(v.index(), 0);

        v.set_index(25);
        v.cursor_up();
        assert_eq!(v.index(), 14);
        v.cursor_up();
        assert_eq!(v.index(), 8);
        v.cursor_up();
        assert_eq!(v.index(), 3);
    }

    #[test]
    fn cursor_down() {
        let b = Rc::new(RefCell::new(Buffer::from_str(
            "a long text line\nhello\nme!\nan other very long line",
        )));
        let mut v = View::new(b);
        v.cursor_down();
        assert_eq!(v.index(), 17);
        v.cursor_down();
        assert_eq!(v.index(), 23);
        v.cursor_down();
        assert_eq!(v.index(), 27);
        v.cursor_down();
        assert_eq!(v.index(), 27);

        v.set_index(10); // on the second t of text
        v.cursor_down();
        assert_eq!(v.index(), 22);
        v.cursor_down();
        assert_eq!(v.index(), 26);
        v.cursor_down();
        assert_eq!(v.index(), 30);
        v.cursor_down();
        assert_eq!(v.index(), 30);
    }
    #[test]
    fn cursor_left() {
        let b = Rc::new(RefCell::new(Buffer::from_str("text\nhello\n")));
        let mut v = View::new(b);
        v.cursor_left();
        assert_eq!(v.index(), 0);

        v.set_index(5);
        v.cursor_left();
        assert_eq!(v.index(), 4);

        v.set_index(2);
        v.cursor_left();
        assert_eq!(v.index(), 1);

        v.set_index(6);
        v.cursor_left();
        assert_eq!(v.index(), 5);
    }
    #[test]
    fn cursor_right() {
        let b = Rc::new(RefCell::new(Buffer::from_str("tt\nh\n")));
        let mut v = View::new(b);
        v.cursor_right();
        assert_eq!(v.index(), 1);
        v.cursor_right();
        assert_eq!(v.index(), 2);
        v.cursor_right();
        assert_eq!(v.index(), 3);
        v.cursor_right();
        assert_eq!(v.index(), 4);
        v.cursor_right();
        assert_eq!(v.index(), 5);
        v.cursor_right();
        assert_eq!(v.index(), 5);
    }
    #[test]
    fn backspace() {
        let b = Rc::new(RefCell::new(Buffer::from_str("hello")));
        let mut v = View::new(b);
        v.backspace();
        assert_eq!(v.to_string(), "hello");
        v.set_index(2);
        v.backspace();
        assert_eq!(v.to_string(), "hllo");
    }
    #[test]
    fn delete_at_cursor() {
        let b = Rc::new(RefCell::new(Buffer::from_str("hello")));
        let mut v = View::new(b);
        v.delete_at_cursor();
        assert_eq!(v.to_string(), "ello");
        v.set_index(3);
        v.delete_at_cursor();
        assert_eq!(v.to_string(), "ell");
        v.delete_at_cursor();
        assert_eq!(v.to_string(), "ell");
    }
}
