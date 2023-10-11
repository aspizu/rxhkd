use crate::key::*;
use xcb::x;

/// A chord represents a combination of modifier keys and a single key.
#[derive(Debug, Copy, Clone)]
pub struct Chord {
    pub modifiers: x::ModMask,
    pub key: Key,
}

impl Chord {
    pub fn grab(self, conn: &xcb::Connection, root: x::Window) -> xcb::VoidCookieChecked {
        conn.send_request_checked(&x::GrabKey {
            owner_events: true,
            grab_window: root,
            modifiers: self.modifiers,
            key: self.key as u8,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
        })
    }

    pub fn ungrab(self, conn: &xcb::Connection, root: x::Window) -> xcb::VoidCookieChecked {
        conn.send_request_checked(&x::UngrabKey {
            key: self.key as u8,
            grab_window: root,
            modifiers: self.modifiers,
        })
    }

    /// Match this chord against a key event.
    pub fn matches(self, state: x::KeyButMask, detail: x::Keycode) -> bool {
        detail == self.key as u8 && key_but_mask_to_mod_mask(state) == self.modifiers
    }
}

/// Key events return a KeyButMask which includes flags for the mouse buttons
/// in addition to the keyboard modifier keys.
/// Converts a KeyButMask into a ModMask discarding the rest of the flags.
fn key_but_mask_to_mod_mask(state: x::KeyButMask) -> x::ModMask {
    let mut mask = x::ModMask::empty();
    if state.contains(x::KeyButMask::SHIFT) {
        mask |= x::ModMask::SHIFT;
    }
    if state.contains(x::KeyButMask::LOCK) {
        mask |= x::ModMask::LOCK;
    }
    if state.contains(x::KeyButMask::CONTROL) {
        mask |= x::ModMask::CONTROL;
    }
    if state.contains(x::KeyButMask::MOD1) {
        mask |= x::ModMask::N1;
    }
    if state.contains(x::KeyButMask::MOD2) {
        mask |= x::ModMask::N2;
    }
    if state.contains(x::KeyButMask::MOD3) {
        mask |= x::ModMask::N3;
    }
    if state.contains(x::KeyButMask::MOD4) {
        mask |= x::ModMask::N4;
    }
    if state.contains(x::KeyButMask::MOD5) {
        mask |= x::ModMask::N5;
    }
    mask
}
