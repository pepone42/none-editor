use crate::buffer::Buffer;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::Range;
use std::slice;
use syntect::highlighting::{HighlightIterator, HighlightState, Highlighter, Style, Theme, ThemeSet};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

lazy_static! {
    pub static ref THEMESET: ThemeSet = ThemeSet::load_defaults();
    pub static ref SYNTAXSET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref STYLE: Styling<'static> = Styling::new();
}

#[derive(Debug)]
pub struct Styling<'a> {
    pub theme: &'a Theme,
}

impl<'a> Styling<'a> {
    pub fn new() -> Self {
        Styling {
            theme: &THEMESET.themes["Solarized (dark)"],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StyleSpan {
    style: Style,
    len: usize,
}

#[derive(Debug)]
pub struct StyledLine {
    inner: Vec<StyleSpan>,
}

impl StyledLine {
    pub fn new() -> Self {
        StyledLine { inner: Vec::new() }
    }
    pub fn iter(&self) -> StyledLineIterator {
        let style_span = self.inner.get(0).cloned();
        let mut style_iter = self.inner.iter();
        style_iter.next();
        StyledLineIterator {
            index: 0,
            style_span,
            style_iter,
        }
    }
}

impl Deref for StyledLine {
    type Target = Vec<StyleSpan>;

    fn deref(&self) -> &Vec<StyleSpan> {
        &self.inner
    }
}

impl FromIterator<StyleSpan> for StyledLine {
    fn from_iter<I: IntoIterator<Item = StyleSpan>>(iterator: I) -> Self {
        let mut v = StyledLine::new();
        for i in iterator {
            v.inner.push(i);
        }
        v
    }
}
#[derive(Debug)]
pub struct StyledLineIterator<'a> {
    index: usize,
    style_span: Option<StyleSpan>,
    style_iter: slice::Iter<'a, StyleSpan>,
}

impl<'a> Iterator for StyledLineIterator<'a> {
    type Item = Style;

    fn next(&mut self) -> Option<Style> {
        if let Some(span) = self.style_span {
            self.index += 1;
            if self.index > span.len {
                self.index = 1;
                self.style_span = self.style_iter.next().cloned();
            }
            self.style_span.map(|s| s.style)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct StylingCache<'a> {
    // one state by line
    pub syntax: &'a SyntaxReference,
    state: Vec<(ParseState, HighlightState)>,
    pub result: Vec<StyledLine>,
}

impl<'a> StylingCache<'a> {
    pub fn new(syntax: &'a SyntaxReference) -> StylingCache<'a> {
        StylingCache {
            syntax,
            state: Vec::new(),
            result: Vec::new(),
        }
    }
    pub fn update(&mut self, r: Range<usize>, b: &Buffer) {
        use std::cmp::min;
        let start = min(self.state.len(), r.start);
        let length = r.end - start;
        self.state.truncate(start);
        self.result.truncate(start);
        for line in b.lines().skip(start).take(length + 1) {
            let highlighter = Highlighter::new(STYLE.theme);
            let mut state = self
                .state
                .last()
                .unwrap_or(&(
                    ParseState::new(self.syntax),
                    HighlightState::new(&highlighter, ScopeStack::new()),
                )).clone();

            let l = line.to_string();
            let v = state.0.parse_line(&l, &SYNTAXSET);
            // let r = HighlightIterator::new(&mut state.1, &v[..], &l, &highlighter)
            //         .map(|x| StyleSpan{style: x.0, len: x.1.chars().count()})
            //         .collect();
            let r = HighlightIterator::new(&mut state.1, &v[..], &l, &highlighter)
                .map(|x| StyleSpan {
                    style: x.0,
                    len: x.1.chars().count(),
                }).collect();
            self.result.push(r);

            self.state.push(state);
        }
    }
    pub fn expand(&mut self, end: usize, b: &Buffer) {
        let start = self.state.len();
        if end > start {
            self.update(start..end, b);
        }
    }
}
