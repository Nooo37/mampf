pub mod config;
pub mod fm_state;
pub mod keys;
pub mod state;
pub mod ui;
pub mod util;

use config::Config;
use state::State;
use ui::terminal_ui::TerminalUI;
use ui::UI;

pub fn main() -> Result<(), std::io::Error> {
    let config = Config::new().expect("Coudln't parse config file.");
    let mut state = State::from(config);
    let mut mytui: TerminalUI = TerminalUI::init().expect("Couldn't initalize TUI backend");
    while !state.is_exit() {
        mytui.refresh(&state).expect("Couldn't refresh");
        let keypress = mytui.get_next_keypress();
        state.handle_keybinds(keypress.expect("Couldn't unwrap keybind"));
    }
    Ok(())
}
