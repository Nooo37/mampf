use std::path::Path;
use termion::event::Key;
use toml::Value;

use crate::util::{Keybind, Action, SortBy, Filter};

// The config struct is the singleton to handle the user configuration.
// Currently it holds only the keybinding but it can be extended to
// cosmetic and other components

#[derive(Debug, Clone)]
pub struct Config {
    pub keybindings: Vec<Keybind>
}

impl Config {
    pub fn new() -> Option<Self> {
        let config_str = get_config_string()?;
        let values = config_str.parse::<Value>().ok()?;
        let keys_table = values["keys"].as_table()?;
        let mut keybindings = Vec::new();
        for keybind in keys_table["app"].as_array()? {
            if let Some(keybind) = parse_one_keybind(keybind) {
                keybindings.push(keybind);
            }
         }
        Some(Config { keybindings })
    }
}

fn get_config_string() -> Option<String> {
    // TODO add different (not hard-coded) config paths to search for
    // this is a temporary solution but should work on most systems 
    let cu_exe = std::env::current_exe().ok()?;
    let mut path = Path::new(&cu_exe);
    path = path.parent()?.parent()?.parent()?;
    let path = path.join("Config.toml");
    Some(std::fs::read_to_string(path).ok()?)
}

// from here on there are only helpers functions to parse a possible keybind
// from the config to an object of the Keybind struct

fn parse_one_keybind(t: &Value) -> Option<Keybind> {
    let arr = t.as_array()?;
    let action = arr.get(1)?.clone().try_into::<String>().ok()?;
    let action = parse_app_command(&action)?;
    let keys = arr.get(0)?.clone().try_into::<String>().ok()?; 
    let keys = parse_keys(&keys)?;
    Keybind::from(keys, action)
}

fn parse_keys(keys: &String) -> Option<Vec<Key>> {
    // TODO add support for parsing command bindings
    let keys_strs = keys.split(" ");
    let mut keys : Vec<Key> = Vec::new();

    for key_str in keys_strs {
        // invalid keys are just ignored for now
        let key = parse_one_key(&key_str.to_string())?;
        keys.push(key);
    }
    if keys.len() == 0 {
        None // if no single key could be parsed, the keybind is invalid
    } else {
        Some(keys) 
    }
}

// parses one keybind expression for example 'C-f'
fn parse_one_key(key: &String) -> Option<Key> {
    // TODO F-keys support 
    // TODO support for keybinds like 'control + up', probably niche anyway
    match key.len() {
        1 => Some(Key::Char(key.chars().next()?)),
        3 => {
          let mut temp = key.chars();
          let modifier: char = temp.next()?;
          if temp.next()? != '-' {
              None
          } else {
              let ch = temp.next()?;
              match modifier {
                  'C' => Some(Key::Ctrl(ch)),
                  'M' => Some(Key::Alt(ch)),
                  _ => None
              }
          }
        },
        _ => { // everything else is one of the special keys
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
                _ => None
            }
        }
    }
    
}

fn parse_app_command(cmd: &String) -> Option<Action> {
    match cmd.to_lowercase().as_str() {
        "up"   => Some(Action::Up),
        "down" => Some(Action::Down),
        "in"   => Some(Action::In),
        "out"  => Some(Action::Out),
        "quit" => Some(Action::Quit),
        "mark" => Some(Action::Mark),
        "unmark" => Some(Action::UnMark),
        "markall" => Some(Action::MarkAll),
        "unmarkall" => Some(Action::UnMarkAll),
        "toggledotfiles" => Some(Action::ToggleFilter(Filter::Dotfiles)),
        "sortbyinc" => Some(Action::DoSortBy(SortBy::LexioInc)),
        "sortbydec" => Some(Action::DoSortBy(SortBy::LexioDec)),
        _ => None
    }
}

