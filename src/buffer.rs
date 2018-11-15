use chardet;
use encoding;
use encoding::label::encoding_from_whatwg_label;
use encoding::EncodingRef;
use encoding::{DecoderTrap, EncoderTrap};
use ropey;
use ropey::Rope;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::Write;
use std::io::Read;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// A text Buffer
#[derive(Clone)]
pub struct Buffer {
    rope: Rope,
    filename: Option<PathBuf>,
    is_dirty: bool,
    encoding: EncodingRef,
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Buffer {{rope: {:?}, filename: {:?}, is_dirty: {}, encoding: {} }}",
            self.rope,
            self.filename,
            self.is_dirty,
            self.encoding.name()
        )
    }
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Buffer {
            rope: Rope::new(),
            filename: None,
            is_dirty: false,
            encoding: encoding_from_whatwg_label("utf8").unwrap(),
        }
    }
    /// create a buffer from the given string
    pub fn from_str(text: &str) -> Self {
        Buffer {
            rope: Rope::from_str(text),
            filename: None,
            is_dirty: false,
            encoding: encoding_from_whatwg_label("utf8").unwrap(),
        }
    }
    /// create a buffer from the give file
    pub fn from_file(filename: &Path) -> Result<Self, io::Error> {
        let mut fh = io::BufReader::new(File::open(filename)?);
        let mut reader: Vec<u8> = Vec::new();

        // read file
        fh.read_to_end(&mut reader)?;

        // detect charset of the file
        let result = chardet::detect(&reader);

        // decode file into utf-8
        let encoding = chardet::charset2encoding(&result.0);
        println!("Detected Encoding: {}", encoding);
        let coder = encoding_from_whatwg_label(encoding).unwrap_or(encoding::all::UTF_8);
        let utf8reader = coder.decode(&reader, DecoderTrap::Replace).expect("Error");

        let r = Rope::from_str(&utf8reader);
        Ok(Buffer {
            rope: r,
            filename: Some(filename.to_owned()),
            is_dirty: false,
            encoding: coder,
        })
    }

    /// return the buffer current encoding
    pub fn get_encoding(&self) -> EncodingRef {
        self.encoding
    }

    /// return the filename
    pub fn get_filename(&self) -> Option<&Path> {
        match &self.filename {
            Some(p) => Some(p.as_path()),
            None => None,
        }
    }

    /// save the current buffer to disk
    pub fn save(&mut self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            if let Ok(r) = self.encoding.encode(&self.rope.to_string(), EncoderTrap::Replace) {
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(filename)?;
                file.write_all(&r)?;
                Ok(())
            } else {
                return Err(io::Error::new(io::ErrorKind::Other, "Error while encoding buffer"));
            }
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "No filename associated"));
        }
    }

    /// save the current buffer to disk with the given filename
    pub fn save_as<P: AsRef<Path>>(&mut self, filename: P) -> io::Result<()> {
        self.set_filename(filename.as_ref());
        self.save()?;
        Ok(())
    }

    /// set filename
    pub fn set_filename(&mut self, filename: &Path) {
        self.filename = Some(filename.to_owned());
    }

    /// Iterate over each char in the buffer
    pub fn chars(&self) -> ropey::iter::Chars {
        self.rope.chars()
    }
    pub fn lines(&self) -> ropey::iter::Lines {
        self.rope.lines()
    }
    /// Total number of chars in the buffer
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }
    /// Total number of lines in the buffer
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }
    /// insert ch at the given position
    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        self.rope.insert_char(char_idx, ch);
        self.is_dirty = true;
    }
    /// Insert the string at the given position
    pub fn insert<S: AsRef<str>>(&mut self, char_idx: usize, text: S) {
        self.rope.insert(char_idx, text.as_ref());
        self.is_dirty = true;
    }
    /// remove the given range from the buffer
    pub fn remove<R: Into<Range<usize>>>(&mut self, char_range: R) {
        self.rope.remove(char_range.into());
        self.is_dirty = true;
    }

    /// Returns the entire buffer as a newly allocated String.
    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }
    pub fn slice<R: Into<Range<usize>>>(&self, r: R) -> String {
        self.rope.slice(r.into()).to_string()
    }

    /// return the line of the given char
    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }
    /// return the first char of the given line
    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    pub fn line_len_no_eol(&self, line_idx: usize) -> usize {
        let l = self.rope.line(line_idx);
        l.chars().filter(|c| *c != '\n' && *c != '\r').count()
    }

    /// return the last char of the given line
    pub fn line_to_last_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx) + self.line_len_no_eol(line_idx)
    }

    /// return the len in chars of the given line
    pub fn line_len(&self, line_idx: usize) -> usize {
        self.rope.line(line_idx).len_chars()
    }
    /// convert an index to a point (line, column)
    pub fn index_to_point(&self, char_idx: usize) -> (usize, usize) {
        let l = self.char_to_line(char_idx);
        let c = char_idx - self.line_to_char(l);
        (l, c)
    }
    /// Convert a point (line, column) to an index
    pub fn point_to_index(&self, point: (usize, usize)) -> usize {
        use std::cmp::min;
        let l = min(point.0, self.len_lines() - 1);

        let c = min(point.1, self.line_len_no_eol(l));
        self.line_to_char(l) + c
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::Buffer;

    #[test]
    fn chars_iterators() {
        let buf = Buffer::from_str("Hello World");
        let res = ['H', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd'];
        for (i, c) in buf.chars().enumerate() {
            assert_eq!(c, res[i]);
        }
    }

    #[test]
    fn len_chars() {
        let buf = Buffer::from_str("Hello World");
        assert_eq!(buf.len_chars(), 11);
        let buf = Buffer::from_str("Hello World\n");
        assert_eq!(buf.len_chars(), 12);
        let buf = Buffer::from_str("Nöel");
        assert_eq!(buf.len_chars(), 4);
    }
    #[test]
    fn len_lines() {
        let buf = Buffer::from_str("Hello World");
        assert_eq!(buf.len_lines(), 1);
        let buf = Buffer::from_str("Hello\nWorld");
        assert_eq!(buf.len_lines(), 2);
    }
    #[test]
    fn remove() {
        let mut buf = Buffer::from_str("Hello World");
        buf.remove(1..3);
        assert_eq!(buf.to_string(), "Hlo World");
    }
    #[test]
    fn index_to_point() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        assert_eq!(buf.index_to_point(3), (0, 3));
        assert_eq!(buf.index_to_point(4), (0, 4));
        assert_eq!(buf.index_to_point(5), (1, 0));
        assert_eq!(buf.index_to_point(12), (2, 1));
    }
    #[test]
    fn point_to_index() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        // Normal case
        assert_eq!(buf.point_to_index((0, 3)), 3);
        assert_eq!(buf.point_to_index((0, 4)), 4);
        assert_eq!(buf.point_to_index((1, 0)), 5);
        assert_eq!(buf.point_to_index((2, 1)), 12);

        // oob case
        assert_eq!(buf.point_to_index((0, 5)), 4); // col too far
        assert_eq!(buf.point_to_index((4, 1)), 12); // line too far
        assert_eq!(buf.point_to_index((4, 6)), 17); // line too far, EOF is treated like a char
    }
    #[test]
    fn line_to_last_char() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        assert_eq!(buf.line_to_last_char(0), 4);
        assert_eq!(buf.line_to_last_char(1), 10);
        assert_eq!(buf.line_to_last_char(2), 17); // EOF is treated like à char
    }
    #[test]
    fn line_len_no_eol() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        assert_eq!(buf.line_len_no_eol(0), 4);
        assert_eq!(buf.line_len_no_eol(1), 5);
        assert_eq!(buf.line_len_no_eol(2), 6);
    }
}
