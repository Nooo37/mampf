use std::process::Command;

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
                    execute_tui("nvim", &mut mytui)?;
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
                    execute_cmd(&cmd, &state, &mut mytui);
                }
                Action::TUICmd(cmd) => {
                    execute_tui(&cmd, &mut mytui)?;
                }
            }
        }
    }
    Ok(())
}

pub fn execute_tui(cmd: &str, tui: &mut TerminalUI) -> Result<(), std::io::Error> {
    let split = cmd.split(' ');
    let mut parts = split.collect::<Vec<&str>>();
    parts.reverse();
    let main = parts.pop().unwrap();
    parts.reverse();
    tui.tui_app_start()?;
    let mut cmd_temp = Command::new(main);
    let command = cmd_temp.args(parts.iter());
    command.spawn()?.wait()?;
    tui.tui_app_end()?;
    Ok(())
}

pub fn execute_cmd(cmd: &str, state: &App, ui: &mut impl UI) -> Option<()> {
    let commands = format_command(state, cmd, ui)?;
    for command in commands {
        execute_one_cmd(&command)?;
    }
    Some(())
}

fn format_command(state: &App, cmd: &str, ui: &mut impl UI) -> Option<Vec<String>> {
    let fm_state = &state.fm_state;
    let mut commands = Vec::new();
    let mut new_cmd = String::new();

    // handle %f (current file) and %d (current directory)
    match &fm_state.get_focused() {
        Some(focused_pathb) => {
            let filename = focused_pathb.file_name()?.to_str()?;
            new_cmd = cmd.replace("%f", filename);
        }
        None => {
            if cmd.contains("%f") || cmd.contains("%d") {
                return None;
            }
        }
    }
    let directory = fm_state.get_currentdir();
    let directory = directory.to_str()?;
    new_cmd = new_cmd.replace("%d", directory);

    // handle %i (input by the user)
    if new_cmd.contains("%i") {
        let input = ui.get_user_input(&state, "Input: ").ok()?;
        new_cmd = new_cmd.replace("%i", &input);
        println!("{}", new_cmd);
        execute_one_cmd(&("notify-send ".to_string() + &new_cmd));
    }

    // handle %F (all marked files) and %D (all corresponding marked directories)
    if new_cmd.contains("%F") || new_cmd.contains("%D") {
        for marked_path in fm_state.get_marked().iter() {
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
