use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util::{Filter, SortBy};

// FMState holds all relevant methods and fields to reproduce the state
// of a file manager. (Essentially a singleton as long as tabbing isn't a thing)

pub struct FMState {
    current_dir: PathBuf,
    focused: Option<PathBuf>,
    marked: Vec<PathBuf>,
    filters: Vec<Filter>, // filters to apply (no filters: everything is shown)
    sort_by: SortBy,
    exit: bool,
}

impl FMState {
    pub fn new() -> Self {
        let marked: Vec<PathBuf> = Vec::new();
        let mut current_dir = PathBuf::new();
        let start_dir = match std::env::var("HOME") {
            Ok(val) => val,
            Err(_e) => "/".to_string(),
        };
        current_dir.push(start_dir);
        let sort_by = SortBy::LexioInc;

        let focused = sort_by.sort(Self::list(&current_dir)).pop();
        FMState {
            current_dir,
            focused: focused,
            marked,
            filters: vec![Filter::Dotfiles],
            sort_by,
            exit: false,
        }
    }

    pub fn is_marked(&self, pathb: PathBuf) -> bool {
        self.marked.iter().any(|pathb_cmp| pathb_cmp == &pathb)
    }

    pub fn mark(&mut self, pathb: &PathBuf) {
        if !self.is_marked(pathb.to_path_buf()) {
            self.marked.push(pathb.to_path_buf());
        }
    }

    pub fn mark_current(&mut self) {
        if let Some(focused) = self.focused.clone() {
            self.mark(&focused);
        }
        self.move_down();
    }

    pub fn unmark_current(&mut self) {
        if let Some(focused) = &self.focused {
            self.marked.retain(|pathb| pathb != focused)
        }
        self.move_down();
    }

    pub fn mark_all(&mut self) {
        self.list_current()
            .iter()
            .for_each(|pathb| self.mark(pathb));
    }

    pub fn unmark_all(&mut self) {
        self.marked.clear();
    }

    pub fn get_idx(&self) -> Option<usize> {
        if let Some(pathb_focused) = self.focused.clone() {
            self.list_current()
                .iter()
                .position(|direle| direle.to_path_buf() == pathb_focused)
        } else {
            None
        }
    }

    pub fn update_by_idx(&mut self, idx: Option<usize>) {
        if let Some(idx) = idx {
            if let Some(asd) = self.list_current().get(idx) {
                self.focused = Some(asd.clone());
            }
        }
    }

    // moves out of the current dir, returns index of the former parent dir
    pub fn move_out(&mut self) {
        if let Some(dir) = self.current_dir.parent() {
            self.focused = Some(self.current_dir.clone());
            self.current_dir = dir.to_path_buf();
        }
        self.update_by_idx(self.get_idx());
    }

    // moves into the current focused dir if possible, returns new focused index
    pub fn move_in(&mut self) -> Option<()> {
        if self.focused.as_ref()?.is_dir() {
            self.current_dir = self.focused.as_ref()?.clone();
            let current_list = self.list_current();
            self.focused = Some(current_list.get(0)?.to_path_buf());
            self.update_by_idx(Some(0));
        }
        None
    }

    pub fn move_up(&mut self) -> Option<()> {
        let current_list = self.list_current();
        let mut new_idx: Option<usize> = None;
        if let Some(focused) = &self.focused {
            let a = current_list.iter().position(|direle| direle == focused)?;
            if a == 0 {
                new_idx = Some(current_list.len() - 1);
            } else {
                new_idx = Some(a - 1);
            }
        } else if !current_list.is_empty() {
            new_idx = Some(0);
        }
        self.focused = Some(current_list.get(new_idx?)?.to_path_buf());
        self.update_by_idx(new_idx);
        None
    }

    pub fn move_down(&mut self) -> Option<()> {
        let current_list = self.list_current();
        let mut new_idx: Option<usize> = None;
        if let Some(focused) = &self.focused {
            let a = current_list.iter().position(|direle| direle == focused)?;
            if a + 1 == current_list.len() {
                new_idx = Some(0);
            } else {
                new_idx = Some(a + 1);
            }
        } else if !current_list.is_empty() {
            new_idx = Some(0);
        }
        self.focused = Some(current_list.get(new_idx?)?.to_path_buf());
        self.update_by_idx(new_idx);
        None
    }

    fn order(&self, list: &mut Vec<PathBuf>) -> Vec<PathBuf> {
        // sort according to the sort_by property
        let mut list = self.sort_by.sort(list.to_vec());
        // remove filter if needed
        for filter in &self.filters {
            list = filter.filter(list);
        }
        list.to_vec()
    }

    fn is_filter_active(&self, filter: &Filter) -> bool {
        self.filters.iter().any(|fil| fil == filter)
    }

    pub fn toggle_filter(&mut self, filter: &Filter) {
        if self.is_filter_active(filter) {
            self.filters = Vec::new();
        } else {
            self.filters.push(filter.clone());
        }
    }

    pub fn list_current(&self) -> Vec<PathBuf> {
        let mut a = FMState::list(&self.current_dir);
        self.order(&mut a)
    }

    pub fn list_prev(&self) -> Vec<PathBuf> {
        if let Some(parent) = self.current_dir.parent() {
            let mut a = Self::list(&parent.to_path_buf());
            self.order(&mut a)
        } else {
            Vec::new()
        }
    }

    pub fn list_next(&self) -> Vec<PathBuf> {
        if let Some(focused) = &self.focused {
            if focused.is_dir() {
                let mut a = Self::list(&focused);
                self.order(&mut a)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    pub fn get_preview(&self) -> Option<String> {
        std::fs::read_to_string(self.focused.as_ref()?).ok()
    }

    pub fn list(directory_path: &PathBuf) -> Vec<PathBuf> {
        let mut list: Vec<PathBuf> = Vec::new();
        let path = Path::new(&directory_path);
        if let Ok(contents) = path.read_dir() {
            for entry in contents {
                match entry {
                    Ok(ele) => list.push(ele.path()),
                    Err(_error) => {}
                }
            }
            list
        } else {
            Vec::new()
        }
    }

    // a couple setter, getter fields to keep all fields private

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn is_exit(&self) -> bool {
        self.exit
    }

    pub fn set_sortby(&mut self, new_sortby: SortBy) {
        self.sort_by = new_sortby;
    }

    pub fn jump_to(&mut self, new_focused: PathBuf) -> Option<usize> {
        if new_focused.is_dir() {
            self.current_dir = new_focused;
            Some(0)
        } else {
            self.focused = Some(new_focused.clone());
            self.current_dir = new_focused.parent()?.to_path_buf();
            self.get_idx()
        }
    }

    pub fn get_currentdir(&self) -> PathBuf {
        self.current_dir.clone()
    }

    pub fn get_focused(&self) -> Option<PathBuf> {
        self.focused.clone()
    }

    // the following functions are to support executing shell commands

    pub fn execute_cmd(&self, mut cmd: String) -> Option<()> {
        let commands = self.format_command(&mut cmd)?;
        for command in commands {
            Self::execute_one_cmd(&command);
        }
        // TODO self.update_marked()
        Some(())
    }

    fn format_command(&self, cmd: &mut String) -> Option<Vec<String>> {
        let mut commands = Vec::new();
        let mut new_cmd = String::new();
        match &self.focused {
            Some(focused_pathb) => {
                let filename = focused_pathb.file_name()?.to_str()?;
                new_cmd = cmd
                    .replace("%f", filename)
                    .replace("%a", focused_pathb.to_str()?);
            }
            None => {
                if cmd.contains("%f") || cmd.contains("%d") {
                    return None;
                }
            }
        }
        let directory = self.current_dir.to_str()?;
        new_cmd = new_cmd.replace("%d", directory);
        if new_cmd.contains("%F") || new_cmd.contains("%D") {
            for marked_path in self.marked.iter() {
                let filename = marked_path.file_name()?.to_str()?;
                let directory = marked_path.parent()?.to_str()?;
                let temp_cmd = new_cmd.replace("%F", filename).replace("%D", directory);
                commands.push(temp_cmd);
            }
        } else {
            commands.push(new_cmd);
        }
        Some(commands)
    }

    fn execute_one_cmd(cmd: &str) -> Option<()> {
        // TODO more parsing needed to allow for grouping everything in parens
        // into one arg. Currently 'notify-send 'for example'' would parse 'for
        // and example' into two args
        let split = cmd.split(' ');
        let mut parts = split.collect::<Vec<&str>>();
        parts.reverse();
        let main = parts.pop()?;
        parts.reverse();
        let mut cmd_temp = Command::new(main);
        let command = cmd_temp.args(parts.iter());
        let _output = command.output().ok()?;
        Some(())
    }
}
