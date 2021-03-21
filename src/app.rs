use crate::{config::Config, keys::Action};
use crate::{
    fm_state::FMState,
    util::{PaneContent, Style},
};
use std::path::PathBuf;
use termion::event::Key;

// State should hold all information to recreate a session
// Tabbing will be a Vector of FMStates in the future
// Might currently look like a wrapper around FMState + KeyState

pub struct App {
    fm_state: FMState,
    keystate: KeyState,
    // TODO implement tabbing: switch to vector
    config: Config,
}

impl App {
    pub fn from(config: Config) -> Self {
        Self {
            fm_state: FMState::new(),
            config: config.clone(),
            keystate: KeyState::new(config),
        }
    }

    pub fn handle_keybinds(&mut self, keypress: Key) -> Option<()> {
        let actions = self.keystate.press(keypress)?;
        for action in actions {
            self.perform(action);
        }
        None
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::Up => {
                self.fm_state.move_up();
            }
            Action::Down => {
                self.fm_state.move_down();
            }
            Action::In => {
                self.fm_state.move_in();
            }
            Action::Out => {
                self.fm_state.move_out();
            }
            Action::Mark => {
                self.fm_state.mark_current();
            }
            Action::UnMark => {
                self.fm_state.unmark_current();
            }
            Action::Quit => {
                self.fm_state.exit();
            }
            Action::MarkAll => {
                self.fm_state.mark_all();
            }
            Action::UnMarkAll => {
                self.fm_state.unmark_all();
            }
            Action::Jump(pathb) => {
                self.fm_state.jump_to(pathb);
            }
            Action::ToggleFilter(filter) => {
                self.fm_state.toggle_filter(&filter);
            }
            Action::DoSortBy(sortby) => {
                self.fm_state.set_sortby(sortby);
            }
            Action::ShellCmd(cmd) => {
                self.fm_state.execute_cmd(cmd);
            }
        }
    }

    pub fn is_exit(&self) -> bool {
        self.fm_state.is_exit()
    }

    pub fn get_current_fm_state(&self) -> &FMState {
        &self.fm_state
    }

    pub fn list_current(&self) -> Vec<PathBuf> {
        self.fm_state.list_current()
    }

    pub fn list_next(&self) -> Vec<PathBuf> {
        self.fm_state.list_next()
    }

    pub fn list_prev(&self) -> Vec<PathBuf> {
        self.fm_state.list_prev()
    }

    pub fn get_preview(&self) -> PaneContent {
        if let Some(focused_pathb) = &self.fm_state.get_focused() {
            if focused_pathb.is_dir() {
                PaneContent::DirElements(
                    FMState::list(focused_pathb)
                        .iter()
                        .map(|x| (x.to_path_buf(), Style::Red))
                        .collect::<Vec<(PathBuf, Style)>>(),
                )
            } else {
                PaneContent::None
            }
        } else {
            PaneContent::None
        }
    }

    pub fn update_by_idx(&mut self, idx: Option<usize>) {
        self.fm_state.update_by_idx(idx)
    }

    pub fn get_idx(&self) -> Option<usize> {
        self.fm_state.get_idx()
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

    pub fn press(&mut self, key: Key) -> Option<Vec<Action>> {
        // TODO number_tracker is not as clean as it maybe Acould be
        self.pressed.clear();
        self.pressed.push(key);
        let mut action_opt: Option<Vec<Action>> = None;
        // check if the key is a number, inc the number_tracker and return
        if let Key::Char(num) = key {
            if num.is_digit(10) {
                if self.number_tracker != 1 {
                    self.number_tracker *= 10;
                }
                // 48 is ascii offset for numbers
                self.number_tracker += num.to_digit(10).unwrap() as usize;
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
