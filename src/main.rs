pub mod app;
pub mod config;
pub mod fm_state;
pub mod keys;
pub mod ui;
pub mod util;

use app::App;
use config::Config;
use keys::{Action, KeyState};
use ui::terminal_ui::TerminalUI;
use ui::UI;

pub fn main() -> Result<(), std::io::Error> {
    let config = Config::new().expect("Coudln't parse config file.");
    let mut keystate = KeyState::new(config.clone());
    let mut state = App::from(config);
    let mut mytui: TerminalUI = TerminalUI::init().expect("Couldn't initalize TUI backend");

    // main loop
    while !state.is_exit() {
        mytui.refresh(&state).expect("Couldn't refresh");
        let keypress = mytui.get_next_keypress();
        let actions = keystate.press(keypress);
        for action in actions {
            match action {
                Action::Up => {
                    state.fm_state.move_up();
                }
                Action::Down => {
                    state.fm_state.move_down();
                }
                Action::In => {
                    state.fm_state.move_in();
                }
                Action::Out => {
                    state.fm_state.move_out();
                }
                Action::Mark => {
                    state.fm_state.mark_current();
                }
                Action::UnMark => {
                    state.fm_state.unmark_current();
                }
                Action::Quit => {
                    state.fm_state.exit();
                }
                Action::MarkAll => {
                    state.fm_state.mark_all();
                }
                Action::UnMarkAll => {
                    state.fm_state.unmark_all();
                }
                Action::Jump(pathb) => {
                    state.fm_state.jump_to(pathb);
                }
                Action::ToggleFilter(filter) => {
                    state.fm_state.toggle_filter(&filter);
                }
                Action::DoSortBy(sortby) => {
                    state.fm_state.set_sortby(sortby);
                }
                Action::ShellCmd(cmd) => {
                    state.fm_state.execute_cmd(cmd);
                }
            }
        }
    }
    Ok(())
}
