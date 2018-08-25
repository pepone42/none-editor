use buffer::Buffer;

pub struct Cursor {
    pub index: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor{index:0}
    }
    pub fn up(&mut self,buf: &Buffer, count: usize) {
        let mut l = self.get_line(buf);
        if (l<count) {
            l = 0;
        } else {
            l -= count;
        }
    }
    pub fn down(&mut self,buf: &Buffer, count: usize) {

    }
    pub fn left(&mut self,buf: &Buffer) {

    }
    pub fn right(&mut self,buf: &Buffer) {

    }
    pub fn get_line(&self, buf: &Buffer) -> usize {
        buf.char_to_line(self.index)
    }
    pub fn get_col(&self, buf: &Buffer) -> usize {
        self.index - buf.line_to_char(self.get_line(buf))
    }
    pub fn as_point(&self, buf: &Buffer) -> (usize, usize) {
        (self.get_line(buf), self.get_col(buf))
    }
}
#[cfg(test)]
mod tests {
    use buffer::Buffer;
    use cursor::Cursor;

    #[test]
    fn get_line_col() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        let mut curs = Cursor::new();
        curs.index = 3;
        assert_eq!(curs.get_line(&buf), 0 );
        assert_eq!(curs.get_col(&buf), 3 );
        curs.index = 4;
        assert_eq!(curs.get_line(&buf), 0 );
        assert_eq!(curs.get_col(&buf), 4 );
        curs.index = 5;
        assert_eq!(curs.get_line(&buf), 1 );
        assert_eq!(curs.get_col(&buf), 0 );
        curs.index = 12;
        assert_eq!(curs.get_line(&buf), 2 );
        assert_eq!(curs.get_col(&buf), 1 );
    }
}