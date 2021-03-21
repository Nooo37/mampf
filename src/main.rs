pub mod app;
pub mod config;
pub mod fm_state;
pub mod keys;
pub mod ui;
pub mod util;

use app::App;
use config::Config;
use ui::terminal_ui::TerminalUI;
use ui::UI;

pub fn main() -> Result<(), std::io::Error> {
    let config = Config::new().expect("Coudln't parse config file.");
    let mut state = App::from(config);
    let mut mytui: TerminalUI = TerminalUI::init().expect("Couldn't initalize TUI backend");
    while !state.is_exit() {
        mytui.refresh(&state).expect("Couldn't refresh");
        let keypress = mytui.get_next_keypress();
        state.handle_keybinds(keypress);
    }
    Ok(())
}
