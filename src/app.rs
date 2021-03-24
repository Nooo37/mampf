use crate::{
    config::Config,
    fm_state::FMState,
    util::{EntryStyle, PaneContent},
};
use std::path::PathBuf;

// State should hold all information to recreate a session
// Tabbing will be a Vector of FMStates in the future
// Might currently look like a wrapper around FMState + KeyState

pub struct App {
    pub fm_state: FMState,
    // TODO implement tabbing: switch to vector
    // TODO implement UI customization
    pub config: Config,
}

impl App {
    pub fn from(config: Config) -> Self {
        Self {
            fm_state: FMState::new(),
            config: config.clone(),
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

    pub fn get_style(&self, pathb: &PathBuf) -> (PathBuf, EntryStyle) {
        // TODO: distinguish directories etc
        let mut style = EntryStyle::Blue;
        if let Some(focused_path) = self.fm_state.get_focused() {
            if &focused_path == pathb {
                style = EntryStyle::Red;
            }
        }
        if pathb.is_dir() {
            if pathb == &self.fm_state.get_currentdir() {
                style = EntryStyle::Red;
            } else {
                style = EntryStyle::Cyan;
            }
        }
        if self.fm_state.get_marked().contains(pathb) {
            style = EntryStyle::Yellow;
        }
        (pathb.to_path_buf(), style)
    }

    pub fn get_content_middle(&self) -> PaneContent {
        PaneContent::DirElements(
            self.fm_state
                .list_current()
                .iter()
                .map(|x| self.get_style(x))
                .collect::<Vec<(PathBuf, EntryStyle)>>(),
        )
    }

    pub fn get_content_left(&self) -> PaneContent {
        PaneContent::DirElements(
            self.fm_state
                .list_prev()
                .iter()
                .map(|x| self.get_style(&x))
                .collect::<Vec<(PathBuf, EntryStyle)>>(),
        )
    }

    pub fn get_content_right(&self) -> PaneContent {
        if let Some(focused_pathb) = &self.fm_state.get_focused() {
            if focused_pathb.is_dir() {
                PaneContent::DirElements(
                    self.fm_state
                        .list_next()
                        .iter()
                        .map(|x| self.get_style(&x))
                        .collect::<Vec<(PathBuf, EntryStyle)>>(),
                )
            } else {
                match std::fs::read_to_string(focused_pathb) {
                    Ok(text) => PaneContent::Text(text),
                    Err(_) => PaneContent::None,
                }
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
