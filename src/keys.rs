use std::path::PathBuf;
use termion::event::Key;

use crate::{
    config::Config,
    util::{Filter, SortBy},
};

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

pub struct KeyState {
    // TODO make use of 'pressed' and add support for leaderkeys
    pressed: Vec<Key>,
    config: Config,
    pub number_tracker: usize, // number tracker so that '12 x' does x 12 times
}

impl KeyState {
    pub fn new(config: Config) -> Self {
        Self {
            pressed: Vec::new(),
            config,
            number_tracker: 0,
        }
    }

    pub fn press(&mut self, key: Key) -> Vec<Action> {
        // TODO number_tracker is not as clean as it maybe Acould be
        self.pressed.clear();
        self.pressed.push(key);
        let mut actions = Vec::new();
        // check if the key is a number, inc the number_tracker and return
        if let Key::Char(num) = key {
            if num.is_digit(10) {
                if self.number_tracker != 1 {
                    self.number_tracker *= 10;
                }
                // 48 is ascii offset for numbers
                self.number_tracker += num.to_digit(10).unwrap() as usize;
                return actions;
            }
        }
        // compare the key to each keybind, find the corresponding action
        // add the action as often as number_tracker says to
        // temporary solution since it doesn't support leader keys yet
        for keybind in self.config.keybindings.clone() {
            if keybind.keys.len() == 1 && keybind.keys.get(0).unwrap_or_else(|| &Key::Null) == &key
            {
                // let mut actions = Vec::new();
                while self.number_tracker > 1 {
                    actions.push(keybind.action.clone());
                    self.number_tracker -= 1;
                }
                self.number_tracker = 0;
                actions.push(keybind.action.clone());
            }
        }
        actions
    }
}
