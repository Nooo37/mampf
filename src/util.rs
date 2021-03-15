use std::path::PathBuf;
use termion::event::Key;

use crate::Config;
use crate::FMState;

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

impl Action {
    pub fn perform(&self, fm_state: &mut FMState) -> Option<usize> {
        match self {
            Action::Up => fm_state.move_up(),
            Action::Down => fm_state.move_down(),
            Action::In => fm_state.move_in(),
            Action::Out => fm_state.move_out(),
            Action::Mark => fm_state.mark_current(),
            Action::UnMark => fm_state.unmark_current(),
            Action::Quit => {
                fm_state.exit();
                None
            }
            Action::MarkAll => {
                fm_state.mark_all();
                None
            }
            Action::UnMarkAll => {
                fm_state.unmark_all();
                None
            }
            Action::Jump(pathb) => fm_state.jump_to(pathb.to_path_buf()),
            Action::ToggleFilter(filter) => {
                fm_state.toggle_filter(filter);
                None
            }
            Action::DoSortBy(sortby) => {
                fm_state.set_sortby(sortby.clone());
                None
            }
            Action::ShellCmd(cmd) => {
                fm_state.execute_cmd(cmd.to_string());
                None
            }
        }
    }
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
            config: config,
            number_tracker: 0,
        }
    }

    pub fn press(&mut self, key: Key) -> Option<Vec<Action>> {
        // TODO number_tracker is not as clean as it maybe Acould be
        self.pressed.clear();
        self.pressed.push(key);
        let mut action_opt: Option<Vec<Action>> = None;
        // check if the key is a number, inc the number_tracker and return
        if let Key::Char(num) = key {
            let nums = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            if nums.iter().any(|ele| ele == &num) {
                if self.number_tracker != 1 {
                    self.number_tracker *= 10;
                }
                // 48 is ascii offset for numbers
                self.number_tracker += num as usize - 48;
                return None;
            }
        }
        // compare the key to each keybind, find the corresponding action
        // add the action as often as number_tracker says to
        for keybind in self.config.keybindings.clone() {
            if keybind.keys.len() == 1 && keybind.keys.get(0)? == &key {
                let mut actions = Vec::new();
                while self.number_tracker > 1 {
                    actions.push(keybind.action.clone());
                    self.number_tracker -= 1;
                }
                self.number_tracker = 0;
                actions.push(keybind.action.clone());
                action_opt = Some(actions);
            }
        }
        action_opt
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Dotfiles,
}

impl Filter {
    // returns whether or not the PathBuf should get filtered out
    pub fn is(&self, pathb: PathBuf) -> bool {
        match self {
            Filter::Dotfiles => {
                if let Some(filename) = pathb.file_name() {
                    if let Some(filename) = filename.to_str() {
                        filename.starts_with(".")
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SortBy {
    LexioInc,
    LexioDec,
    // TODO New,
}
