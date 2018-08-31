use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use buffer;
use buffer::Buffer;
use keybinding::KeyBinding;

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
    cursor: usize,
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

#[derive(Debug)]
pub struct View {
    pub buffer: Rc<RefCell<Buffer>>,
    cursor: usize,
    first_visible_line: usize,
    pub selection: Option<Range<usize>>,
    in_selection: bool,
    selection_start: usize,
    undo_stack: UndoStack,
    page_length: usize,
}

impl View {
    pub fn new(buffer: Rc<RefCell<Buffer>>) -> Self {
        //let us =
        View {
            buffer,
            cursor: 0,
            first_visible_line: 0,
            selection: None,
            in_selection: false,
            selection_start: 0,
            undo_stack: UndoStack::new(),
            page_length: 0,
        }
    }


    pub fn set_page_length(&mut self, page_length: usize) {
        self.page_length = page_length;
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

    pub fn insert_char(&mut self, ch: char) {
        self.push_state();
        {
            let mut b = self.buffer.borrow_mut();
            if let Some(r) = self.selection.clone() {
                self.cursor = r.start;
                b.remove(r);
            }
            b.insert_char(self.cursor, ch);
        } // unborrow buffer
        self.cursor_right();
        self.clear_selection();
        self.focus_on_cursor();
    }
    pub fn insert(&mut self, text: &str) {
        self.push_state();
        {
            let mut b = self.buffer.borrow_mut();
            if let Some(r) = self.selection.clone() {
                self.cursor = r.start;
                b.remove(r);
            }
            b.insert(self.cursor, &text);
        } // unborrow buffer
        self.cursor += text.chars().count();
        self.clear_selection();
        self.focus_on_cursor();
    }
    pub fn backspace(&mut self) {
        self.push_state();
        if let Some(r) = self.selection.clone() {
            let mut b = self.buffer.borrow_mut();
            self.cursor = r.start;
            b.remove(r);
        } else if self.cursor > 0 {
            self.cursor_left();
            let mut b = self.buffer.borrow_mut();
            b.remove(self.cursor..self.cursor + 1);
        }
        self.clear_selection();
        self.focus_on_cursor();
    }
    pub fn delete_at_cursor(&mut self) {
        self.push_state();
        {
            let mut b = self.buffer.borrow_mut();
            if let Some(r) = self.selection.clone() {
                self.cursor = r.start;
                b.remove(r);
            } else if self.cursor < b.len_chars() {
                b.remove(self.cursor..self.cursor + 1);
            }
        }
        self.clear_selection();
        self.focus_on_cursor();
    }
    pub fn to_string(&self) -> String {
        self.buffer.borrow().to_string()
    }

    pub fn undo(&mut self) {
        // let mut b = self.buffer.borrow_mut();
        // b.undo();
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

    pub fn redo(&mut self) {
        // let mut b = self.buffer.borrow_mut();
        // b.undo();
        println!("redo {:?}", self.undo_stack.stack);
        if let Some(state) = self.undo_stack.redo() {
            self.buffer.replace(state.buffer);
            self.cursor = state.cursor;
        }
        self.focus_on_cursor();
    }

    pub fn get_selection(&self) -> Option<String> {
        match self.selection.clone() {
            None => None,
            Some(s) => Some(self.buffer.borrow().slice(s).to_string()),
        }
    }

    pub fn line_idx(&self) -> usize {
        let b = self.buffer.borrow();
        let (l, _) = b.index_to_point(self.cursor);
        l
    }
    pub fn col_idx(&self) -> usize {
        let b = self.buffer.borrow();
        let (_, c) = b.index_to_point(self.cursor);
        c
    }
    fn cursor_up(&mut self) {
        let b = self.buffer.borrow();
        let (mut l, c) = b.index_to_point(self.cursor);
        if l > 0 {
            l -= 1
        };
        self.cursor = b.point_to_index((l, c));
    }
    fn cursor_down(&mut self) {
        let b = self.buffer.borrow();
        let (mut l, c) = b.index_to_point(self.cursor);
        if l < b.len_lines() - 1 {
            l += 1
        };
        self.cursor = b.point_to_index((l, c));
    }
    fn cursor_left(&mut self) {
        let b = self.buffer.borrow();
        if self.cursor > 0 {
            let line_idx = b.char_to_line(self.cursor);
            let line_idx_char = b.line_to_char(line_idx);
            // handle crlf and lf
            if self.cursor == line_idx_char {
                self.cursor = b.line_to_last_char(line_idx - 1);
            } else {
                self.cursor -= 1;
            }
        }
    }
    fn cursor_right(&mut self) {
        let b = self.buffer.borrow();
        if self.cursor < b.len_chars() {
            let line_idx = b.char_to_line(self.cursor);
            let line_idx_char = b.line_to_last_char(line_idx);
            // handle crlf and lf
            if self.cursor == line_idx_char {
                self.cursor = b.line_to_char(line_idx + 1);
            } else {
                self.cursor += 1;
            }
        }
    }

    pub fn move_cursor(&mut self, dir: Direction) {
        match dir {
            Direction::Up => self.cursor_up(),
            Direction::Down => self.cursor_down(),
            Direction::Right => self.cursor_right(),
            Direction::Left => self.cursor_left(),
        }
        if self.in_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
        self.focus_on_cursor();
    }
    pub fn move_page(&mut self, dir: Direction) {
        for _ in 0..self.page_length {
            self.move_cursor(dir);
        }
        self.focus_on_cursor();
    }
    pub fn home(&mut self) {
        let l = self.line_idx();
        self.cursor = self.buffer.borrow().line_to_char(l);
        if self.in_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
    }
    pub fn end(&mut self) {
        let l = self.line_idx();
        self.cursor = self.buffer.borrow().line_to_last_char(l);
        if self.in_selection {
            self.expand_selection();
        } else {
            self.clear_selection();
        }
    }

    pub fn index(&self) -> usize {
        self.cursor
    }
    pub fn set_index(&mut self, idx: usize) {
        assert!(idx <= self.buffer.borrow().len_chars());
        self.cursor = idx;
    }

    pub fn first_visible_line(&self) -> usize {
        self.first_visible_line
    }

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
    pub fn start_selection(&mut self) {
        self.in_selection = true;
        match self.selection.clone() {
            // if the cursor is at the start or the end of the selection
            // -> do nothing, we want to continue to expand the current selection
            Some(Range { start, end }) if start == self.cursor || end == self.cursor => (),
            _ => {
                self.selection = Some(self.cursor..self.cursor);
                self.selection_start = self.cursor;
            }
        }
    }
    pub fn end_selection(&mut self) {
        self.in_selection = false;
    }
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
    fn expand_selection(&mut self) {
        self.selection = match self.selection {
            None => Some(self.selection_start..self.cursor),
            Some(_) => {
                if self.selection_start < self.cursor {
                    Some(self.selection_start..self.cursor)
                } else {
                    Some(self.cursor..self.selection_start)
                }
            }
        }
    }
}

pub trait ViewCmd {
    fn name(&self) -> &'static str;
    fn desc(&self) ->  &'static str;
    fn keybinding(&self) -> Vec<KeyBinding>;
    fn run(&mut self,&mut View);
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
