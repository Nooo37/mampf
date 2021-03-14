use std::path::{Path, PathBuf};

use crate::util::{SortBy, Filter};

// FMState holds all relevant methods and fields to reproduce the state
// of a file manager. (Essentially a singleton as long as tabbing isn't a thing)

pub struct FMState {
    current_dir: PathBuf,
    focused: Option<PathBuf>,
    marked: Vec<PathBuf>,
    filters: Vec<Filter>, // filters to apply (no filters: everything is shown)
    sort_by: SortBy,
    exit: bool
}

impl FMState {

    pub fn new() -> Self {
        let marked : Vec<PathBuf> = Vec::new();
        let mut current_dir = PathBuf::new();
        let start_dir = match std::env::var("HOME") {
            Ok(val) => val,
            Err(_e) => "/".to_string()
        };
        current_dir.push(start_dir);
        let focused = FMState::list(&current_dir).pop().unwrap();
        FMState {
            current_dir,
            focused: Some(focused),
            marked,
            filters: vec![Filter::Dotfiles],
            sort_by : SortBy::LexioDec,
            exit : false
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
    
    pub fn mark_current(&mut self) -> Option<usize> {
        if let Some(focused) = self.focused.clone() {
            self.mark(&focused);
        }
        self.move_down()
    }

    pub fn unmark_current(&mut self) -> Option<usize> {
        if let Some(focused) = &self.focused {
            self.marked.retain(|pathb| pathb != focused)
        }
        self.move_down()
    }

    pub fn mark_all(&mut self) {
        self.list_current().iter().for_each(|pathb| self.mark(pathb));
    }

    pub fn unmark_all(&mut self) {
        self.marked.clear();
    }

    pub fn update_by_idx(&mut self, idx: Option<usize>) {
        if let Some(idx) = idx {
            if let Some(asd) = self.list_current().get(idx) {
                self.focused = Some(asd.clone());
            }
        }
    }
    
    // moves out of the current dir, returns index of the former parent dir
    pub fn move_out(&mut self) -> Option<usize> {
        if let Some(dir) = self.current_dir.parent() {
            self.focused = Some(self.current_dir.clone());
            self.current_dir = dir.to_path_buf();
        }
        if let Some(focused) = &self.focused {
            self.list_current().iter().position(|direle| { &direle == &focused })
        } else {
            None
        }
    }

    // moves into the current focused dir if possible, returns new focused index
    pub fn move_in(&mut self) -> Option<usize> { 
        if self.focused.as_ref()?.is_dir() {
            self.current_dir = self.focused.as_ref()?.clone();
            let current_list = self.list_current();
            self.focused = Some(current_list.get(0)?.to_path_buf());
            Some(0)
        } else {
            None
        }
    }

    pub fn move_up(&mut self) -> Option<usize> {
        let current_list = self.list_current();
        let mut new_idx : Option<usize> = None;
        if let Some(focused) = &self.focused {
            let a = current_list.iter().position(|direle| { &direle == &focused })?;
            if a == 0 {
                new_idx = Some(current_list.len() - 1);
            } else {
                new_idx = Some(a - 1);
            }
        } else if current_list.len() > 0 {
            new_idx = Some(0);
        } 
        self.focused = Some(current_list.get(new_idx?)?.to_path_buf());
        new_idx
    }

    pub fn move_down(&mut self) -> Option<usize> {
        let current_list = self.list_current();
        let mut new_idx : Option<usize> = None;
        if let Some(focused) = &self.focused {
            let a = current_list.iter().position(|direle| { &direle == &focused })?;
            if a + 1 == current_list.len() {
                new_idx = Some(0);
            } else {
                new_idx = Some(a + 1);
            }
        } else if current_list.len() > 0 {
            new_idx = Some(0);
        } 
        self.focused = Some(current_list.get(new_idx?)?.to_path_buf());
        new_idx
    }

    fn order(&self, list: &mut Vec<PathBuf>) -> Vec<PathBuf> {
        // sort according to the sort_by property
        list.sort_by( match self.sort_by {
            SortBy::LexioInc => {
                |x: &PathBuf, y: &PathBuf| {
                    x.to_str().unwrap().partial_cmp(&y.to_str().unwrap()).unwrap()
                }
            }
            SortBy::LexioDec => {
                |x: &PathBuf, y: &PathBuf| {
                    y.to_str().unwrap().partial_cmp(&x.to_str().unwrap()).unwrap()
                }
            }
        });
        // remove filter if needed
        for filter in &self.filters {
            match filter {
                Filter::Dotfiles => list.retain(|ele| !Filter::Dotfiles.is(ele.to_path_buf()))
            }
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
        if let Some(content) = std::fs::read_to_string(self.focused.as_ref()?).ok() {
            Some(content)
        } else {
            None
        }
    }
    
    pub fn list(directory_path: &PathBuf) -> Vec<PathBuf> {
        let mut list : Vec<PathBuf> = Vec::new();
        let path = Path::new(&directory_path);
        if let Ok(contents) = path.read_dir() {
            for entry in contents {
                match entry {
                    Ok(ele) => list.push(ele.path()),
                    Err(_error) => {},
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

    // currently not in use
    pub fn _set_currentdir(&mut self, new_dir: PathBuf) {
        self.current_dir = new_dir;
    }

    pub fn get_currentdir(&self) -> PathBuf {
        self.current_dir.clone()
    }
   
    // TODO command support
    pub fn execute_cmd(&mut self, cmd: String) -> Option<usize> {
        None
    }
}

