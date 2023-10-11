mod bind;
mod chord;
mod key;
mod parse;
use anyhow::Context;
use anyhow::Result;
use bind::*;
use parse::*;
use std::ptr;
use xcb::x;
use xcb::x::Event;
use xcb::Event::X;

/// Helper function to grab all chords from a mode.
/// Will flush the connection.
fn grab_binds(binds: &Vec<Bind>, conn: &xcb::Connection, root: x::Window) -> xcb::ConnResult<()> {
    for bind in binds {
        bind.chord.grab(conn, root);
    }
    conn.flush()
}

/// Helper function to ungrab all chords from a mode.
/// Will flush the connection.
fn ungrab_binds(binds: &Vec<Bind>, conn: &xcb::Connection, root: x::Window) -> xcb::ConnResult<()> {
    for bind in binds {
        bind.chord.ungrab(conn, root);
    }
    conn.flush()
}

/// TODO
fn mainloop(binds: Vec<Bind>) -> Result<()> {
    // The global mode, think of as the normal mode in vi.
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = conn.get_setup();
    let screen = setup
        .roots()
        .nth(screen_num as usize)
        .context("No display.")?;
    // We listen for keyboard events on the root window.
    let root = screen.root();

    // Stores all the binds we have grabbed the chord for. Initally will start
    // out as the global mode.
    // We need to grab chords to be able to listen for them.
    // Grabbed chords will NOT be available to any other application. So we
    // must ungrab them after we are done with them.
    let mut grabbed = &binds;
    grab_binds(grabbed, &conn, root)?;

    loop {
        match conn.wait_for_event()? {
            X(Event::KeyPress(event)) => {
                // Linear search over all grabbed binds to match against this
                // key press event.
                // We could replace this with a better solution that might
                // involve binary trees.
                let Some(bind) = grabbed
                    .iter()
                    .find(|v| v.chord.matches(event.state(), event.detail()))
                else {
                    continue;
                };
                // Output will have a newline.
                if let Some(output) = &bind.output {
                    println!("{output}");
                }
                match &bind.action {
                    Action::EnterMode { binds } => {
                        // Ungrab chords from previous mode and switch to
                        // this mode.
                        ungrab_binds(grabbed, &conn, root)?;
                        grabbed = binds;
                        grab_binds(grabbed, &conn, root)?;
                    }
                    Action::None => {
                        // If the current mode is the global mode, no need to
                        // change the mode to the global mode.
                        if ptr::eq(grabbed, &binds) {
                            continue;
                        }
                        ungrab_binds(grabbed, &conn, root)?;
                        grabbed = &binds;
                        grab_binds(grabbed, &conn, root)?;
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let src = std::fs::read_to_string("rxhkdrc")?;
    let src = src.as_str();
    let (parser_data, mut parser_state) = new_parser(src);
    let binds = parser_data.binds(&mut parser_state);
    mainloop(binds)
}
