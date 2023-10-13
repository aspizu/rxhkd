use serde::Serialize;
use strum::EnumString;

/// All keycodes for my laptop keyboard.
/// These might or might not be the same for everyone.
#[derive(Debug, Copy, Clone, EnumString, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Key {
   Esc = 9,
   F1 = 67,
   F2 = 68,
   F3 = 69,
   F4 = 70,
   F5 = 71,
   F6 = 72,
   F7 = 73,
   F8 = 74,
   F9 = 75,
   F10 = 76,
   F11 = 95,
   F12 = 96,
   PrtSc = 107,
   Insert = 118,
   Delete = 119,
   Pause = 127,
   Star = 63,
   NumpadSlash = 106,
   Tilde = 49,
   One = 10,
   Two = 11,
   Three = 12,
   Four = 13,
   Five = 14,
   Six = 15,
   Seven = 16,
   Eight = 17,
   Nine = 18,
   Zero = 19,
   Minus = 20,
   Equal = 21,
   Backspace = 22,
   NumLock = 77,
   Plus = 86,
   NumpadMinus = 82,
   Tab = 23,
   Q = 24,
   W = 25,
   E = 26,
   R = 27,
   T = 28,
   Y = 29,
   U = 30,
   I = 31,
   O = 32,
   P = 33,
   LBrace = 34,
   RBrace = 35,
   Backslash = 51,
   Home = 79,
   NumpadUp = 80,
   PageUp = 81,
   CapsLock = 66,
   A = 38,
   S = 39,
   D = 40,
   F = 41,
   G = 42,
   H = 43,
   J = 44,
   K = 45,
   L = 46,
   Semicolon = 47,
   Quotes = 48,
   Return = 36,
   NumpadLeft = 83,
   Numpad5 = 84,
   NumpadRight = 85,
   LShift = 50,
   Z = 52,
   X = 53,
   C = 54,
   V = 55,
   B = 56,
   N = 57,
   M = 58,
   Comma = 59,
   Period = 60,
   Slash = 61,
   RShift = 62,
   End = 87,
   NumpadDown = 88,
   PageDown = 89,
   Lctrl = 37,
   Win = 133,
   Space = 65,
   Lalt = 64,
   Ralt = 108,
   Rctrl = 105,
   Left = 113,
   Up = 111,
   Down = 116,
   Right = 114,
   Numpad0 = 90,
   NumpadPeriod = 91,
   NumpadReturn = 104,
}
