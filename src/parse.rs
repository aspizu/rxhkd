use crate::bind::*;
use crate::chord::*;
use crate::key::*;
use std::str;
use std::str::FromStr;
use xcb::x;

/// This is a backtracking recursive descent parser to parse the configuration file
/// of rxhkd.
/// There is no error reporting, the parser simply discards any invalid structure.

/// A token is a reference to a slice from the parser's source string.
#[derive(Debug)]
pub struct Token<'a> {
   pub slice: &'a [u8],
   pub i: usize,
}

/// The parser can only index ASCII strings.
impl<'a> From<Token<'a>> for &'a str {
   fn from(val: Token<'a>) -> Self {
      str::from_utf8(val.slice).unwrap()
   }
}

/// Parser state and data need to be stored in separate variables because of Rust's
/// mutability semantics.
/// Parser data is immutable.
#[derive(Debug, Copy, Clone)]
pub struct ParserData<'a> {
   pub src: &'a [u8],
}

/// Parser state is mutable.
#[derive(Debug)]
pub struct ParserState {
   /// Index of character to be parsed next.
   i: usize,
}

/// Helper function to create a new parser.
pub fn new_parser(src: &str) -> (ParserData, ParserState) {
   (ParserData { src: src.as_bytes() }, ParserState { i: 0 })
}

impl<'a> ParserData<'a> {
   /// Helper function to create a backtracking descent function, where if the function
   /// returns None, parser's position will be reset.
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

   /// Returns true and advances the parser's position when string is matched.
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

   /// Advances the parser's position and returns a token when matches an identifier.
   fn identifier(&self, s: &mut ParserState) -> Option<Token> {
      let mut i = 0;
      loop {
         if self.src.len() <= s.i + i {
            break;
         }
         let c = self.src[s.i + i];
         // The characters which are allowed to be in an identifier.
         if c != b'_' && c != b'-' && !c.is_ascii_alphanumeric() {
            break;
         }
         i += 1;
      }
      if i == 0 {
         return None;
      }
      s.i += i;
      Some(Token { slice: &self.src[(s.i - i)..s.i], i: s.i - i })
   }

   /// Advances the parser's position and returns a token when matches a comment.
   /// The token will include the newline.
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
      Some(Token { slice: &self.src[i..(s.i - 1)], i })
   }

   /// Advance and ignore whitespaces.
   fn whitespace(&self, s: &mut ParserState) {
      while self.src.len() > s.i {
         let c = self.src[s.i];
         if c != b' ' {
            break;
         }
         s.i += 1;
      }
   }

   /// Advance and ignore newlines.
   fn newlines(&self, s: &mut ParserState) {
      while self.src.len() > s.i && self.src[s.i] == b'\n' {
         s.i += 1;
      }
   }

   /// Match a single line. Includes the newline.
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
      Some(Token { slice: &self.src[i..(s.i - 1)], i })
   }

   /// Advance and return the number of whitespaces from the beginning of a line.
   /// Can be called only when the parser is at the beginning of a line.
   fn offside(&self, s: &mut ParserState) -> usize {
      assert!(s.i == 0 || self.src[s.i - 1] == b'\n');
      let i = s.i;
      while s.i < self.src.len() && self.src[s.i] == b' ' {
         s.i += 1;
      }
      s.i - i
   }

   /// Returns true of the offside rule fails. If the current indentation `m` is
   /// smaller than the indentation of the block we are inside, then return true.
   /// If the current character is a newline, ignore this line as it is a blank
   /// line.
   fn offside_rule(&self, s: &mut ParserState, m: usize, mm: usize) -> bool {
      m < mm && (s.i >= self.src.len() || self.src[s.i] != b'\n')
   }

   /// Matches a indented multi-line string. Indentation will be stripped off.
   fn multiline(&self, s: &mut ParserState) -> String {
      let i = s.i;
      let mm = self.offside(s);
      s.i = i;
      let mut v = "".to_owned();
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
         v.push_str(str::from_utf8(&self.src[(i + m)..s.i]).unwrap());
      }
   }

   /// Parse an modifier sequence.
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
            // We ignore the + separator, this will parse invalid files.
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

   /// Parse a key identifier.
   fn key(self, s: &mut ParserState) -> Option<Key> {
      Key::from_str(self.identifier(s)?.into()).ok()
   }

   /// Parse a chord.
   fn chord(&self, s: &mut ParserState) -> Option<Chord> {
      self.backtrack(s, |s| {
         let modifier = self.modifier(s);
         let key = self.key(s)?;
         self.whitespace(s);
         Some(Chord { modifiers: modifier, key })
      })
   }

   /// Parse a key bind.
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
         // Remove trailing newlines and whitespaces from the end of the command string.
         output.truncate(output.trim_end().len());
         Some(Bind { chord, output: Some(output), action: Action::None })
      })
   }

   /// Parse a mode bind.
   fn mode(&self, s: &mut ParserState) -> Option<Bind> {
      self.backtrack(s, |s| {
         if !self.string(s, "mode") {
            return None;
         }
         self.whitespace(s);
         let chord = self.chord(s)?;
         self.whitespace(s);
         // Trailing whitespace after the colon will cause problems.
         if !self.string(s, ":\n") {
            return None;
         }
         let binds = self.binds(s);
         Some(Bind { chord, output: None, action: Action::EnterMode { binds } })
      })
   }

   /// Parse the insides of a mode block or the entire file.
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
