use std::path::{Path, PathBuf};
use termion::event::Key;
use toml::Value;

use crate::util::{Filter, SortBy};
use crate::{
    keys::{Action, Keybind},
    util::{PaneConfig, PaneRole},
};

// The config struct is the singleton to handle the user configuration.
// Currently it holds only the keybinding but it can be extended to
// cosmetic and other components
// Other than that, all functions to parse the config file are here

#[derive(Debug, Clone)]
pub struct Config {
    pub keybindings: Vec<Keybind>,
    pub panes: Vec<PaneConfig>,
}

impl Config {
    pub fn new() -> Option<Self> {
        let config_str = get_config_string()?;
        let values = config_str.parse::<Value>().ok()?;
        let keys_table = values["keys"].as_table()?;
        let mut keybindings = Vec::new();
        for keybind in keys_table["app"].as_array()? {
            if let Some(keybind) = parse_one_app_keybind(keybind) {
                keybindings.push(keybind);
            }
        }
        for keybind in keys_table["cmd"].as_array()? {
            if let Some(keybind) = parse_one_cmd_keybind(keybind) {
                keybindings.push(keybind);
            }
        }
        for keybind in keys_table["jmp"].as_array()? {
            if let Some(keybind) = parse_one_jump_keybind(keybind) {
                keybindings.push(keybind);
            }
        }
        // TODO: Do some actual config options for these two new features
        let panes = vec![
            PaneConfig {
                role: PaneRole::Previous(2),
                width: 10,
            },
            PaneConfig {
                role: PaneRole::Previous(1),
                width: 15,
            },
            PaneConfig {
                role: PaneRole::Current,
                width: 25,
            },
            PaneConfig {
                role: PaneRole::Preview,
                width: 50,
            },
        ];
        let vim_bind = Keybind {
            keys: vec![Key::Char('v')],
            action: Action::TUICmd("nvim".to_string()),
        };
        keybindings.push(vim_bind);
        Some(Config { keybindings, panes })
    }
}

fn get_config_string() -> Option<String> {
    // TODO add different (not hard-coded) config paths  search for
    // this is a temporary solution but should work on most systems
    let cu_exe = std::env::current_exe().ok()?;
    let mut path = Path::new(&cu_exe);
    path = path.parent()?.parent()?.parent()?;
    let path = path.join("Config.toml");
    Some(std::fs::read_to_string(path).ok()?)
}

// from here on there are only helpers functions to parse a possible keybind
// from the config to an object of the Keybind struct

// TODO maybe one can put the following three into one generic function

fn parse_one_app_keybind(t: &Value) -> Option<Keybind> {
    let arr = t.as_array()?;
    let action = arr.get(1)?.clone().try_into::<String>().ok()?;
    let action = parse_app_command(&action)?;
    let keys = arr.get(0)?.clone().try_into::<String>().ok()?;
    let keys = parse_keys(&keys)?;
    Keybind::from(keys, action)
}

fn parse_one_cmd_keybind(t: &Value) -> Option<Keybind> {
    let arr = t.as_array()?;
    let action = arr.get(1)?.clone().try_into::<String>().ok()?;
    let keys = arr.get(0)?.clone().try_into::<String>().ok()?;
    let keys = parse_keys(&keys)?;
    Keybind::from(keys, Action::ShellCmd(action))
}

fn parse_one_jump_keybind(t: &Value) -> Option<Keybind> {
    let arr = t.as_array()?;
    let action = arr.get(1)?.clone().try_into::<String>().ok()?;
    let pathb = parse_jump_command(&action)?;
    let keys = arr.get(0)?.clone().try_into::<String>().ok()?;
    let keys = parse_keys(&keys)?;
    Keybind::from(keys, Action::Jump(pathb))
}

fn parse_keys(keys: &str) -> Option<Vec<Key>> {
    // TODO add support for parsing command bindings
    let keys_strs = keys.split(' ');
    let mut keys: Vec<Key> = Vec::new();

    for key_str in keys_strs {
        // invalid keys are just ignored for now
        let key = parse_one_key(&key_str.to_string())?;
        keys.push(key);
    }
    if keys.is_empty() {
        None // if no single key could be parsed, the keybind is invalid
    } else {
        Some(keys)
    }
}

// parses one keybind expression for example 'C-f'
fn parse_one_key(key: &str) -> Option<Key> {
    // TODO support for keybinds like 'control + up', probably niche anyway
    match key.to_lowercase().as_str() {
        "backspace" => Some(Key::Backspace),
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "up" => Some(Key::Up),
        "down" => Some(Key::Down),
        "pageup" => Some(Key::PageUp),
        "pagedown" => Some(Key::PageDown),
        "backtab" => Some(Key::BackTab),
        "delete" => Some(Key::Delete),
        "insert" => Some(Key::Insert),
        "null" => Some(Key::Null),
        "esc" => Some(Key::Esc),
        _ => {
            match key.len() {
                1 => Some(Key::Char(key.chars().next()?)), // single char key
                2 => {
                    // F-key
                    let mut temp = key.chars();
                    let first_char = temp.next()?;
                    if first_char != 'f' && first_char != 'F' {
                        None
                    } else {
                        let second_char = temp.next()?;
                        if second_char.is_digit(10) {
                            // 48 is ascii offset for numbers
                            let num = second_char.to_digit(10)? as u8;
                            Some(Key::F(num))
                        } else {
                            None
                        }
                    }
                }
                3 => {
                    // key with modifier
                    let mut temp = key.chars();
                    let modifier: char = temp.next()?;
                    if temp.next()? != '-' {
                        None
                    } else {
                        let ch = temp.next()?;
                        match modifier {
                            'C' => Some(Key::Ctrl(ch)),
                            'M' => Some(Key::Alt(ch)),
                            _ => None,
                        }
                    }
                }
                _ => None,
            }
        }
    }
}

fn parse_app_command(cmd: &str) -> Option<Action> {
    match cmd.to_lowercase().as_str() {
        "up" => Some(Action::Up),
        "down" => Some(Action::Down),
        "in" => Some(Action::In),
        "out" => Some(Action::Out),
        "quit" => Some(Action::Quit),
        "mark" => Some(Action::Mark),
        "unmark" => Some(Action::UnMark),
        "markall" => Some(Action::MarkAll),
        "unmarkall" => Some(Action::UnMarkAll),
        "toggledotfiles" => Some(Action::ToggleFilter(Filter::Dotfiles)),
        "sortbyinc" => Some(Action::DoSortBy(SortBy::LexioInc)),
        "sortbydec" => Some(Action::DoSortBy(SortBy::LexioDec)),
        "sortbynew" => Some(Action::DoSortBy(SortBy::New)),
        _ => None,
    }
}

fn parse_jump_command(jmp: &str) -> Option<PathBuf> {
    let home_dir = std::env::var("HOME").ok()?;
    let home_dir = home_dir.as_str();
    let mut jmp = jmp.replace("~", home_dir);
    jmp = jmp.replace("$HOME", home_dir);
    let pathb = PathBuf::from(jmp);
    if pathb.exists() {
        Some(pathb)
    } else {
        None
    }
}
