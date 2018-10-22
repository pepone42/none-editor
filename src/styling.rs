use buffer::Buffer;
use std::ops::Range;
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

type StyledLine = Vec<(Style, usize)>;

pub struct StyledLineIterator {
    index: usize,
    style: StyledLine,
}

impl StyledLineIterator {
    pub fn new_for(style: StyledLine) -> Self {
        StyledLineIterator { index: 1, style }
    }
}

impl Iterator for StyledLineIterator {
    type Item = Style;

    fn next(&mut self) -> Option<Style> {
        if self.style.is_empty() {
            None
        } else {
            let style = self.style[0];
            if self.index > style.1 {
                self.index = 2;
                self.style.remove(0);
                self.style.first().map(|s| s.0)
            } else {
                self.index += 1;
                Some(style.0)
            }
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

            self.result.push(
                HighlightIterator::new(&mut state.1, &v[..], &l, &highlighter)
                    .map(|x| (x.0, x.1.chars().count()))
                    .collect(),
            );
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
