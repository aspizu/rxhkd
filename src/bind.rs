use crate::chord::*;

/// A bind represents a chord with an associated action.
/// All binds can have a output parameter regardless of the action they
/// trigger.
#[derive(Debug)]
pub struct Bind {
   pub chord: Chord,
   pub output: Option<String>,
   pub action: Action,
}

/// An action is a task performed when a bind is triggered.
#[derive(Debug)]
pub enum Action {
   /// Switch to a new mode.
   EnterMode { binds: Vec<Bind> },
   /// Exit to the default mode.
   None,
}
