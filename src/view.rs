use std::cell::RefCell;
use std::io;
use std::ops::Range;
use std::rc::Rc;

use crate::styling::SYNTAXSET;

use syntect::highlighting;

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::keybinding::KeyBinding;
use crate::styling::StylingCache;
use crate::styling::STYLE;
use crate::window::Geometry;
use crate::SETTINGS;

use crate::system::{Canvas,Font,FontType};
use nanovg::Color;

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
    CRLF,
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
        min(self.start, self.end)
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
struct CharMetrics {
    advance: f32,
    height: f32,
}

impl From<Font> for CharMetrics {
    fn from(font: Font) -> Self {
        let advance =
        if let FontType::MonoSpaced(advance) = font.kind {
            advance
        } else {
            panic!("not supported");
        };
        CharMetrics {advance, height: font.line_height}
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Viewport {
    line_start: usize,
    heigth: usize,
    col_start: usize,
    width: usize,
}

impl Viewport {
    fn line_end(&self) -> usize {
        self.line_start + self.heigth
    }
    fn col_end(&self) -> usize {
        self.col_start + self.width
    }
    fn contain(&self, line: usize, col: usize) -> bool {
        line >= self.line_start && line <= self.line_end() && col >= self.col_start && col <= self.col_end()
    }
}

#[derive(Debug)]
pub struct View<'a> {
    buffer: Rc<RefCell<Buffer>>,
    cursor: Cursor,
    selection: Option<Selection>,
    undo_stack: UndoStack,
    linefeed: LineFeed,
    geometry: Geometry,
    viewport: Viewport,
    char_metrics: CharMetrics,
    styling: Option<StylingCache<'a>>,
    name: String,
}

impl<'a> View<'a> {
    /// Create a new View for the given buffer
    pub fn new(buffer: Rc<RefCell<Buffer>>) -> Self {
        let mut v = View {
            buffer: buffer.clone(),
            cursor: Cursor::new(buffer.clone()),
            selection: None,
            undo_stack: UndoStack::new(),
            linefeed: LineFeed::LF,
            geometry: Default::default(),
            viewport: Default::default(),
            char_metrics: Default::default(),
            styling: None,
            name: String::new(), // TODO: useless string allocation
        };
        v.update_name();
        v.detect_linefeed();
        v.detect_syntax();
        v
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    fn update_name(&mut self) {
        self.name = {
            match self.buffer.borrow().get_filename() {
                None => "untilted".to_owned(),
                Some(path) => path.file_name().unwrap().to_string_lossy().into_owned(),
            }
        };
    }
    /// save the underlying buffer to disk
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
        self.update_name();
        self.detect_syntax();
        Ok(())
    }

    /// return the number of line visible on screen
    pub fn page_length(&self) -> usize {
        self.viewport.heigth
    }

    /// resize the view and update the viewport accordingly
    pub fn relayout(&mut self, geometry: Geometry, canvas: &Canvas) {
        self.geometry = geometry;
        self.char_metrics = CharMetrics::from(canvas.fonts["mono"]);
        self.viewport.heigth = (self.geometry.h / self.char_metrics.height) as usize - 1;
        self.viewport.width = (self.geometry.w / self.char_metrics.advance) as usize - 1;
        let end = self.viewport.line_end();
        self.expand_styling_cache(end);
    }

    fn get_state(&self) -> State {
        State {
            buffer: self.buffer.borrow().clone(),
            cursor: self.cursor.clone(),
        }
    }

    fn push_state(&mut self) {
        let state = self.get_state();
        self.undo_stack.push(&state);
    }

    /// return the file extension or None if there is no file attached to the buffer
    pub fn get_extension(&self) -> Option<String> {
        self.buffer
            .borrow()
            .get_filename()
            .and_then(|f| f.extension())
            .and_then(|e| e.to_str())
            .map(|x| x.to_string())
    }

    /// detect language from extension
    pub fn detect_syntax(&mut self) {
        let plain_text = SYNTAXSET.find_syntax_plain_text();
        let syntax = match self.get_extension() {
            None => plain_text,
            Some(ext) => SYNTAXSET.find_syntax_by_extension(&ext).unwrap_or(plain_text),
        };
        self.styling = Some(StylingCache::new(syntax));
        let end = self.buffer.borrow().len_lines(); // self.viewport.line_end();
        self.expand_styling_cache(end);
    }

    /// get the current syntax
    pub fn get_syntax(&'a self) -> &'a str {
        match &self.styling {
            None => &"Plain text",
            Some(s) => &s.syntax.name,
        }
    }
    pub fn is_dirty(&self) -> bool {
        self.buffer.borrow().is_dirty()
    }

    /// get the buffer encoding
    pub fn get_encoding(&self) -> &str {
        self.buffer.borrow().get_encoding().name()
    }

    fn update_styling_cache(&mut self, r: Range<usize>) {
        if let Some(ref mut style) = self.styling {
            style.update(r, &self.buffer.borrow());
        }
    }
    fn expand_styling_cache(&mut self, end: usize) {
        if let Some(ref mut style) = self.styling {
            style.expand(end, &self.buffer.borrow());
        }
    }

    // pub fn edit(&mut self, r: Range<usize>, text: &str) {
    //     let start = self.line_idx();
    //     self.push_state();

    //     // Clear selection if any
    //     if let Some(r) = self.selection {
    //         self.cursor.set_index(r.lower());
    //         self.buffer.borrow_mut().remove(r);
    //     }

    //     if r.start != r.end {
    //         self.cursor.set_index(r.start);
    //         self.buffer.borrow_mut().remove(r.clone());
    //     }
    //     self.buffer.borrow_mut().insert(r.start, text);

    //     self.cursor.set_index(r.start + text.chars().count());
    //     self.clear_selection();
    //     self.focus_on_cursor();

    //     let end = self.viewport.line_end();
    //     self.update_styling_cache(start..end);
    // }

    /// insert the given char at the cursor position
    pub fn insert_char(&mut self, ch: char) {
        let start = self.line_idx();
        self.push_state();

        if let Some(r) = self.selection {
            self.cursor.set_index(r.lower());
            self.buffer.borrow_mut().remove(r);
        }
        self.buffer.borrow_mut().insert_char(self.cursor.get_index(), ch);

        self.cursor_right();
        self.clear_selection();
        self.focus_on_cursor();

        let end = self.viewport.line_end();
        self.update_styling_cache(start..end);
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
        let start = self.line_idx();
        self.push_state();

        if let Some(r) = self.selection {
            self.cursor.set_index(r.lower());
            self.buffer.borrow_mut().remove(r);
        }
        self.buffer.borrow_mut().insert(self.cursor.get_index(), &text);
        self.cursor.set_index(self.cursor.get_index() + text.chars().count());
        self.clear_selection();
        self.focus_on_cursor();

        let end = self.viewport.line_end();
        self.update_styling_cache(start..end);
    }

    /// delete the charater directly to the left of cursor
    pub fn backspace(&mut self) {
        let start = self.line_idx();
        self.push_state();
        if let Some(r) = self.selection {
            let mut b = self.buffer.borrow_mut();
            self.cursor.set_index(r.lower());
            b.remove(r);
        } else if self.cursor.get_index() > 0 {
            self.cursor_left();
            let mut b = self.buffer.borrow_mut();
            b.remove(self.cursor.get_index()..self.cursor.get_previous_index());
        }
        self.clear_selection();
        self.focus_on_cursor();

        let end = self.viewport.line_end();
        self.update_styling_cache(start..end);
    }

    /// delete the charater under the cursor
    pub fn delete_at_cursor(&mut self) {
        let start = self.line_idx();
        self.push_state();
        if let Some(r) = self.selection {
            self.cursor.set_index(r.lower());
            self.buffer.borrow_mut().remove(r);
        } else if self.cursor.get_index() < self.buffer.borrow().len_chars() {
            let curs = self.cursor.get_index();
            self.cursor_right();
            self.buffer
                .borrow_mut()
                .remove(self.cursor.get_previous_index()..self.cursor.get_index());
            self.cursor.set_index(curs);
        }
        self.clear_selection();
        self.focus_on_cursor();
        let end = self.viewport.line_end();
        self.update_styling_cache(start..end);
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
        let start = self.line_idx();
        let end = self.viewport.line_end();
        self.update_styling_cache(start..end);
    }

    /// redo the last undo action
    pub fn redo(&mut self) {
        if let Some(state) = self.undo_stack.redo() {
            self.buffer.replace(state.buffer);
            self.cursor = state.cursor;
        }
        self.focus_on_cursor();
        let start = self.line_idx();
        let end = self.viewport.line_end();
        self.update_styling_cache(start..end);
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
        self.buffer.borrow().index_to_point(self.cursor.get_index()).0
    }

    /// return the cursor position in column
    pub fn col_idx(&self) -> usize {
        self.buffer.borrow().index_to_point(self.cursor.get_index()).1
    }

    /// return the cursor position in line,col corrdinate
    pub fn cursor_as_point(&self) -> (usize, usize) {
        self.buffer.borrow().index_to_point(self.cursor.get_index())
    }

    fn cursor_up(&mut self) {
        self.cursor.up(1);
    }
    fn cursor_down(&mut self) {
        self.cursor.down(1);
    }
    fn cursor_left(&mut self) {
        self.cursor.left();
    }
    fn cursor_right(&mut self) {
        self.cursor.right();
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
        for _ in 0..self.page_length() {
            self.move_cursor(dir, expand_selection);
        }
        self.focus_on_cursor();
    }

    /// put the cursor at the begining of the line
    pub fn home(&mut self, expand_selection: bool) {
        // let l = self.line_idx();
        // self.cursor.set(self.buffer.borrow().line_to_char(l));
        self.cursor.goto_line_start();
        if expand_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
        self.focus_on_cursor();
    }

    /// put the cursor at the end of the line
    pub fn end(&mut self, expand_selection: bool) {
        // let l = self.line_idx();
        // self.cursor.set(self.buffer.borrow().line_to_last_char(l));
        self.cursor.goto_line_end();
        if expand_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
        self.focus_on_cursor();
    }

    // /// return the cursor position in number of chars from the begining of the buffer
    // pub fn index(&self) -> usize {
    //     self.cursor.get_index()
    // }

    // /// put the cursor at the given position
    // pub fn set_index(&mut self, idx: usize) {
    //     assert!(idx <= self.buffer.borrow().len_chars());
    //     self.cursor.set(idx);
    // }

    /// Set the cursor to the given pixel position
    pub fn click(&mut self, x: i32, y: i32, expand_selection: bool) {
        let mut col = x / self.char_metrics.advance as i32 + self.viewport.col_start as i32;
        let mut line = y / self.char_metrics.height as i32 + self.viewport.line_start as i32;

        if col < 0 {
            col = 0;
        }
        if line < 0 {
            line = 0;
        }

        let idx = self.buffer.borrow().point_to_index(line as _, col as _);
        self.cursor.set_index(idx);
        if expand_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
        self.focus_on_cursor();
    }

    /// select the word when double clicked
    pub fn double_click(&mut self, x: i32, y: i32) {
        self.select_word_under_cursor();
    }

    /// Select the word under the cursor
    pub fn select_word_under_cursor(&mut self) {
        let line = self.buffer.borrow().char_to_line(self.cursor.get_index());
        let mut start = self.buffer.borrow().line_to_char(line);
        let mut end = start;
        for c in self.buffer.borrow().chars_on_line(line) {
            match c {
                ' ' | '`' | '~' | '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '-' | '=' | '+' | '['
                | '{' | ']' | '}' | '\\' | '|' | ';' | ':' | '\'' | '"' | ',' | '.' | '<' | '>' | '/' | '?' => {
                    if start <= self.cursor.get_index() && self.cursor.get_index() < end {
                        self.selection = Some(Selection { start, end });
                        return;
                    }
                    start = end + 1;
                }
                _ => {}
            }
            end += 1;
        }
        self.selection = None;
    }

    /// scroll the view in the given direction
    pub fn scroll(&mut self, dir: Direction, amount: i32) {
        for _ in 0..amount {
            match dir {
                Direction::Up => {
                    if self.viewport.line_start > 0 {
                        self.viewport.line_start -= 1;
                    }
                }
                Direction::Down => {
                    if self.viewport.line_start < self.buffer.borrow().len_lines() {
                        self.viewport.line_start += 1;
                    }
                }
                _ => (),
            }
        }
    }

    /// Detect the carriage return type of the buffer
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
                        cr += 1;
                    }
                }
            } else if c == '\n' {
                lf += 1;
            }
        }

        self.linefeed = if cr > crlf && cr > lf {
            LineFeed::CR
        } else if lf > crlf && lf > cr {
            LineFeed::LF
        } else {
            LineFeed::CRLF
        }
    }

    pub fn detect_indentation(&self) -> Indentation {
        let b = self.buffer.borrow();
        let mut tab = 0;
        let mut spaces = Vec::<u32>::new();
        let _tab_width = 0;
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
        let pagelen = self.viewport.heigth;
        let l = self.line_idx();
        if l < self.viewport.line_start {
            self.viewport.line_start = l;
        }
        if l > self.viewport.line_end() {
            self.viewport.line_start = l - pagelen;
        }
        {
            let b = self.buffer.borrow();
            self.viewport.line_start = min(self.viewport.line_start, b.len_lines());
        }

        let pagewidth = self.viewport.width;
        let c = self.col_idx();
        if c < self.viewport.col_start {
            self.viewport.col_start = c;
        }
        if c > self.viewport.col_end() {
            self.viewport.col_start = c - pagewidth;
        }

        let end = self.viewport.line_end();
        self.expand_styling_cache(end);
    }

    /// Draw the vew on the given screen
    pub fn draw(&self, canvas: &mut Canvas) {
        let adv = self.char_metrics.advance;
        let line_spacing = self.char_metrics.height;
        let mut y = line_spacing;

        let tabsize: i32 = SETTINGS.read().unwrap().get("tabSize").unwrap();
        
        let first_visible_line = self.viewport.line_start;
        let first_visible_col = self.viewport.col_start;
        let page_len = self.viewport.heigth;

        let mut current_col = 0;

        let mut line_index = first_visible_line;
        for line in self.buffer.borrow().lines().skip(first_visible_line).take(page_len + 1) {
            let mut style = self
                .styling
                .as_ref()
                .and_then(|s| s.result.get(line_index))
                .map(|s| s.iter());
            let mut idx = self.buffer.borrow().line_to_char(line_index);

            for c in line.chars() {
                let x = (current_col - first_visible_col as i32) as f32 * adv;

                let fg = match style.as_mut().and_then(|s| s.next()) {
                    None => Color::from_rgb(255, 255, 255),
                    Some(s) => Color::from_rgb(s.foreground.r, s.foreground.g, s.foreground.b),
                };
                match self.selection {
                    Some(sel) if sel.contains(idx) => {
                        let color = STYLE.theme.settings.selection.unwrap_or(highlighting::Color::WHITE);
                        canvas.set_color(Color::from_rgb(color.r, color.g, color.b));
                        canvas.move_to(x as _, y - canvas.fonts["mono"].descender - line_spacing);
                        canvas.draw_rect(adv as _, line_spacing as _);
                    }
                    _ => (),
                }
                match c {
                    '\t' => {
                        let nbspace = ((current_col + tabsize) / tabsize) * tabsize;
                        current_col = nbspace;
                    }
                    '\0' => (),
                    '\r' => (), //idx -= 1,
                    '\n' => (),
                    // Bom hiding. TODO: rework
                    '\u{feff}' | '\u{fffe}' => (),
                    _ => {
                        canvas.move_to(x as _, y as _);
                        canvas.set_color(fg);
                        canvas.draw_char(c);
                        current_col += 1;
                    }
                }
                idx += 1;
            }
            line_index += 1;
            y += line_spacing;
            current_col = 0;
        }

        // Cursor
        let fg = STYLE.theme.settings.caret.unwrap_or(highlighting::Color::WHITE);
        let (mut line, mut col) = self.cursor_as_point();
        if self.viewport.contain(line, col) {
            line -= first_visible_line;
            col -= first_visible_col;
            canvas.move_to(
                col as f32 * adv,
                line as f32 * line_spacing - canvas.fonts["mono"].descender,
            );
            canvas.set_color(Color::from_rgb(fg.r, fg.g, fg.b));
            canvas.draw_rect(2.0, line_spacing as _);
        }
    }

    /// clear the current selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
    fn expand_selection(&mut self) {
        self.selection = if let Some(mut selection) = self.selection {
            selection.expand(self.cursor.get_index());
            Some(selection)
        } else {
            Some(Selection::new(
                self.cursor.get_previous_index(),
                self.cursor.get_index(),
            ))
        }
    }
}

pub trait ViewCmd {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn keybinding(&self) -> Vec<KeyBinding>;
    fn run(&mut self, _: &mut View<'_>);
}

#[cfg(test)]
mod tests {
    use crate::buffer::Buffer;
    use crate::view::View;
    use std::cell::RefCell;
    use std::rc::Rc;


    #[test]
    fn new_view() {
        let b = Rc::new(RefCell::new(Buffer::new()));
        let _v = View::new(b);
    }
    #[test]
    fn insert() {
        let b = Rc::new(RefCell::new(Buffer::from_str("text")));
        let mut v = View::new(b);
        v.insert_char('r');
        assert_eq!(v.to_string(), "rtext");
        v.insert_char('e');
        assert_eq!(v.to_string(), "retext");
        v.cursor.set_index(6);
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
    fn set_index_oob() {
        let b = Rc::new(RefCell::new(Buffer::from_str("text")));
        let mut v = View::new(b);
        v.cursor.set_index(5);
        assert_eq!(v.cursor.get_index(), 4);
    }

    #[test]
    fn set_index() {
        let b = Rc::new(RefCell::new(Buffer::from_str("text\nplop")));
        let mut v = View::new(b);
        v.cursor.set_index(4);
        assert_eq!(v.cursor.get_index(), 4);
        v.cursor.set_index(5);
        assert_eq!(v.cursor.get_index(), 5);
    }

    #[test]
    fn cursor_up() {
        let b = Rc::new(RefCell::new(Buffer::from_str(
            "text\nhello\nme!\nan other very long line",
        )));
        let mut v = View::new(b);
        v.cursor.set_index(11);
        v.cursor_up();
        assert_eq!(v.cursor.get_index(), 5);
        v.cursor_up();
        assert_eq!(v.cursor.get_index(), 0);
        v.cursor_up();
        assert_eq!(v.cursor.get_index(), 0);

        v.cursor.set_index(25);
        v.cursor_up();
        assert_eq!(v.cursor.get_index(), 14);
        v.cursor_up();
        assert_eq!(v.cursor.get_index(), 10);
        v.cursor_up();
        assert_eq!(v.cursor.get_index(), 4);
    }

    #[test]
    fn cursor_down() {
        let b = Rc::new(RefCell::new(Buffer::from_str(
            "a long text line\nhello\nme!\nan other very long line",
        )));
        let mut v = View::new(b);
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 17);
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 23);
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 27);
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 27);

        v.cursor.set_index(10); // on the second t of text
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 22);
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 26);
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 37);
        v.cursor_down();
        assert_eq!(v.cursor.get_index(), 37);
    }
    #[test]
    fn cursor_left() {
        let b = Rc::new(RefCell::new(Buffer::from_str("text\nhello\n")));
        let mut v = View::new(b);
        v.cursor_left();
        assert_eq!(v.cursor.get_index(), 0);

        v.cursor.set_index(5);
        v.cursor_left();
        assert_eq!(v.cursor.get_index(), 4);

        v.cursor.set_index(2);
        v.cursor_left();
        assert_eq!(v.cursor.get_index(), 1);

        v.cursor.set_index(6);
        v.cursor_left();
        assert_eq!(v.cursor.get_index(), 5);
    }
    #[test]
    fn cursor_right() {
        let b = Rc::new(RefCell::new(Buffer::from_str("tt\nh\n")));
        let mut v = View::new(b);
        v.cursor_right();
        assert_eq!(v.cursor.get_index(), 1);
        v.cursor_right();
        assert_eq!(v.cursor.get_index(), 2);
        v.cursor_right();
        assert_eq!(v.cursor.get_index(), 3);
        v.cursor_right();
        assert_eq!(v.cursor.get_index(), 4);
        v.cursor_right();
        assert_eq!(v.cursor.get_index(), 5);
        v.cursor_right();
        assert_eq!(v.cursor.get_index(), 5);
    }
    #[test]
    fn backspace() {
        let b = Rc::new(RefCell::new(Buffer::from_str("hello")));
        let mut v = View::new(b);
        v.backspace();
        assert_eq!(v.to_string(), "hello");
        v.cursor.set_index(2);
        v.backspace();
        assert_eq!(v.to_string(), "hllo");
    }
    #[test]
    fn delete_at_cursor() {
        let b = Rc::new(RefCell::new(Buffer::from_str("hello")));
        let mut v = View::new(b);
        v.delete_at_cursor();
        assert_eq!(v.to_string(), "ello");
        v.cursor.set_index(3);
        v.delete_at_cursor();
        assert_eq!(v.to_string(), "ell");
        v.delete_at_cursor();
        assert_eq!(v.to_string(), "ell");
    }
}
