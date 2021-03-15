mod tui;
mod state;
mod util;
mod config;
use config::Config;
use state::FMState;
use util::KeyState;

pub fn main() -> Result<(), std::io::Error> {
    let fm_state = FMState::new();
    let config = Config::new().expect("Coudln't parse config file.");
    let ks = KeyState::new(config.clone());
    tui::main(fm_state, ks)?;
    // println!("{:?}", config);
    Ok(())
}

