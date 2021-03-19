use std::path::PathBuf;
use termion::event::Key;

use crate::util::{Filter, SortBy};

// Multiple smaller structs that are used all across the code, are declared here

#[derive(Debug, Clone)]
pub enum Action {
    Up,
    Down,
    In,
    Out,
    Quit,
    Mark,
    MarkAll,
    UnMark,
    UnMarkAll,
    Jump(PathBuf),
    ToggleFilter(Filter),
    DoSortBy(SortBy),
    ShellCmd(String),
}

#[derive(Debug, Clone)]
pub struct Keybind {
    // to allow for leader keys, the actual keycombination is a vector
    pub keys: Vec<Key>,
    pub action: Action,
}

impl Keybind {
    pub fn from(keys: Vec<Key>, action: Action) -> Option<Self> {
        Some(Keybind { keys, action })
    }
}
