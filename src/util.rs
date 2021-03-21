use std::{path::PathBuf, time::SystemTime};

// The filter struct, the sortby struct and some helper function can be found here

#[derive(Debug, Clone)]
pub enum Style {
    Red,
    Yellow,
    Green,
    Blue,
    Magenta,
    Black,
    Gray,
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
                        filename.starts_with('.')
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    pub fn filter(&self, mut list: Vec<PathBuf>) -> Vec<PathBuf> {
        list.retain(|ele| !self.is(ele.to_path_buf()));
        list
    }
}

// TODO: Every element should have a boolean field
// that says whether to sort by increasing or decreasing
// as to prevent doubling all element with an Inc and Dec version

#[derive(Debug, Clone)]
pub enum SortBy {
    LexioInc,
    LexioDec,
    New,
}

impl SortBy {
    pub fn sort(&self, mut list: Vec<PathBuf>) -> Vec<PathBuf> {
        list.sort_by(match self {
            SortBy::LexioInc => |x: &PathBuf, y: &PathBuf| {
                x.to_str()
                    .unwrap()
                    .partial_cmp(&y.to_str().unwrap())
                    .unwrap()
            },
            SortBy::LexioDec => |x: &PathBuf, y: &PathBuf| {
                y.to_str()
                    .unwrap()
                    .partial_cmp(&x.to_str().unwrap())
                    .unwrap()
            },
            SortBy::New => {
                |x: &PathBuf, y: &PathBuf| get_modified(y).partial_cmp(&get_modified(x)).unwrap()
            }
        });
        list
    }
}

#[derive(Debug, Clone)]
pub enum PaneContent {
    DirElements(Vec<(PathBuf, Style)>),
    Text(String),
    Image(PathBuf),
    None,
}

// some usefull helper functions

pub fn get_size(pathb: Option<PathBuf>) -> String {
    match pathb {
        Some(pathbuf) => match std::fs::metadata(pathbuf) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    return String::from("dir");
                }
                match metadata.len() {
                    0..=1000 => metadata.len().to_string() + "B",
                    1001..=1000000 => (metadata.len() / 1000).to_string() + "KB",
                    1000001..=1000000000 => (metadata.len() / 1000000).to_string() + "MB",
                    _ => String::from("very large"),
                }
            }
            Err(_) => String::from("no metadata found"),
        },
        None => String::from(""),
    }
}

pub fn get_modified(pathb: &PathBuf) -> Option<u64> {
    let metadata = std::fs::metadata(pathb).ok()?;
    let systime = metadata.modified().ok()?;
    match systime.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => Some(n.as_secs()),
        Err(_) => Some(0),
    }
}
