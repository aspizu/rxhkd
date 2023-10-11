use crate::bind::*;
use crate::chord::*;
use crate::key::*;
use std::str;
use std::str::FromStr;
use xcb::x;

#[derive(Debug)]
pub struct Token<'a> {
    pub slice: &'a [u8],
    pub i: usize,
}

impl<'a> From<Token<'a>> for &'a str {
    fn from(val: Token<'a>) -> Self {
        str::from_utf8(val.slice).unwrap()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ParserData<'a> {
    pub src: &'a [u8],
}

#[derive(Debug)]
pub struct ParserState {
    i: usize,
}

pub fn new_parser(src: &str) -> (ParserData, ParserState) {
    (
        ParserData {
            src: src.as_bytes(),
        },
        ParserState { i: 0 },
    )
}

impl<'a> ParserData<'a> {
    fn backtrack<T, F>(&self, s: &mut ParserState, f: F) -> Option<T>
    where
        F: FnOnce(&mut ParserState) -> Option<T>,
    {
        let i = s.i;
        let v = f(s);
        if v.is_none() {
            s.i = i;
        }
        v
    }

    fn string(&self, s: &mut ParserState, string: &str) -> bool {
        let string = string.as_bytes();
        for (i, &c) in string.iter().enumerate() {
            if self.src.len() <= s.i + i || self.src[s.i + i] != c {
                return false;
            }
        }
        s.i += string.len();
        true
    }

    fn identifier(&self, s: &mut ParserState) -> Option<Token> {
        let mut i = 0;
        loop {
            if self.src.len() <= s.i + i {
                break;
            }
            let c = self.src[s.i + i];
            if c != b'_' && c != b'-' && !c.is_ascii_alphanumeric() {
                break;
            }
            i += 1;
        }
        if i == 0 {
            return None;
        }
        s.i += i;
        Some(Token {
            slice: &self.src[(s.i - i)..s.i],
            i: s.i - i,
        })
    }

    fn comment(&self, s: &mut ParserState) -> Option<Token> {
        if self.src.len() <= s.i || self.src[s.i] != b'#' {
            return None;
        }
        let i = s.i;
        while self.src.len() > s.i && self.src[s.i] != b'\n' {
            s.i += 1;
        }
        if self.src.len() > s.i {
            s.i += 1;
        }
        Some(Token {
            slice: &self.src[i..(s.i - 1)],
            i,
        })
    }

    fn whitespace(&self, s: &mut ParserState) {
        while self.src.len() > s.i {
            let c = self.src[s.i];
            if c != b' ' {
                break;
            }
            s.i += 1;
        }
    }

    fn newlines(&self, s: &mut ParserState) {
        while self.src.len() > s.i && self.src[s.i] == b'\n' {
            s.i += 1;
        }
    }

    fn line(&self, s: &mut ParserState) -> Option<Token> {
        if self.src.len() <= s.i {
            return None;
        }
        let i = s.i;
        while self.src.len() > s.i && self.src[s.i] != b'\n' {
            s.i += 1;
        }
        if self.src.len() > s.i {
            s.i += 1;
        }
        Some(Token {
            slice: &self.src[i..(s.i - 1)],
            i,
        })
    }

    fn offside(&self, s: &mut ParserState) -> usize {
        assert!(s.i == 0 || self.src[s.i - 1] == b'\n');
        let i = s.i;
        while s.i < self.src.len() && self.src[s.i] == b' ' {
            s.i += 1;
        }
        s.i - i
    }

    fn offside_rule(&self, s: &mut ParserState, m: usize, mm: usize) -> bool {
        m < mm && (s.i >= self.src.len() || self.src[s.i] != b'\n')
    }

    fn multiline(&self, s: &mut ParserState) -> String {
        let i = s.i;
        let mm = self.offside(s);
        s.i = i;
        let mut v = format!("");
        loop {
            let i = s.i;
            let m = self.offside(s);
            if self.offside_rule(s, m, mm) {
                s.i = i;
                break v;
            }
            while s.i < self.src.len() && self.src[s.i] != b'\n' {
                s.i += 1;
            }
            if s.i < self.src.len() {
                s.i += 1;
            }
            v.extend(str::from_utf8(&self.src[(i + m)..s.i]).unwrap().chars());
        }
    }

    fn modifier(self, s: &mut ParserState) -> x::ModMask {
        let mut modmask = x::ModMask::empty();
        loop {
            if self.string(s, "shift") {
                modmask |= x::ModMask::SHIFT;
            } else if self.string(s, "caps-lock") {
                modmask |= x::ModMask::LOCK;
            } else if self.string(s, "ctrl") {
                modmask |= x::ModMask::CONTROL;
            } else if self.string(s, "alt") {
                modmask |= x::ModMask::N1;
            } else if self.string(s, "num-lock") {
                modmask |= x::ModMask::N2;
            } else if self.string(s, "mod3") {
                modmask |= x::ModMask::N3;
            } else if self.string(s, "super") {
                modmask |= x::ModMask::N4;
            } else if self.string(s, "mod5") {
                modmask |= x::ModMask::N5;
            } else {
                self.whitespace(s);
                self.string(s, "+");
                self.whitespace(s);
                break;
            }
            self.whitespace(s);
            self.string(s, "+");
            self.whitespace(s);
        }
        modmask
    }

    fn key(self, s: &mut ParserState) -> Option<Key> {
        Key::from_str(self.identifier(s)?.into()).ok()
    }

    fn chord(&self, s: &mut ParserState) -> Option<Chord> {
        self.backtrack(s, |s| {
            let modifier = self.modifier(s);
            let key = self.key(s)?;
            self.whitespace(s);
            Some(Chord {
                modifiers: modifier,
                key,
            })
        })
    }

    fn bind(&self, s: &mut ParserState) -> Option<Bind> {
        self.backtrack(s, |s| {
            let chord = self.chord(s)?;
            self.whitespace(s);
            if !self.string(s, ":") {
                return None;
            }
            let mut output = if self.string(s, "\n") {
                self.multiline(s)
            } else {
                str::from_utf8(self.line(s)?.slice).unwrap().to_owned()
            };
            output.truncate(output.trim_end().len());
            Some(Bind {
                chord,
                output: Some(output),
                action: Action::None,
            })
        })
    }

    fn mode(&self, s: &mut ParserState) -> Option<Bind> {
        self.backtrack(s, |s| {
            if !self.string(s, "mode") {
                return None;
            }
            self.whitespace(s);
            let chord = self.chord(s)?;
            self.whitespace(s);
            if !self.string(s, ":\n") {
                return None;
            }
            let binds = self.binds(s);
            Some(Bind {
                chord,
                output: None,
                action: Action::EnterMode { binds },
            })
        })
    }

    pub fn binds(&self, s: &mut ParserState) -> Vec<Bind> {
        let i = s.i;
        let mm = self.offside(s);
        s.i = i;
        let mut binds = vec![];
        loop {
            let i = s.i;
            let m = self.offside(s);
            if self.offside_rule(s, m, mm) {
                s.i = i;
                break binds;
            }
            let Some(bind) = self.mode(s).or_else(|| self.bind(s)) else {
                break binds;
            };
            binds.push(bind);
        }
    }
}
