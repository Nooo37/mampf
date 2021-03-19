use crate::state::State;
use termion::event::Key;

pub mod terminal_ui;

pub trait UI {
    // There should be the possiblity to initalize the UI
    fn init() -> Result<Self, std::io::Error>
    where
        Self: Sized;

    // There should be the possibility to get String input from the user
    fn get_user_input(&mut self, state: &State, question: &str) -> Result<String, std::io::Error>;

    // For keybindings there should be a function that returns the next keypress
    fn get_next_keypress(&mut self) -> Option<Key>;

    // Refreshes the UI based on the current state
    fn refresh(&mut self, state: &State) -> Result<(), std::io::Error>;
}
