use crate::ui::UI;
use crate::util::{get_size, Filter};
use crate::{fm_state::FMState, state::State};
use std::{io::Stdout, path::PathBuf};
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListState, Paragraph, Text},
    Terminal,
};

pub struct TerminalUI {
    pub liststate: ListState,
    pub input_state: Option<String>,
    pub terminal: Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>,
}

impl UI for TerminalUI {
    fn init() -> Result<Self, std::io::Error> {
        let stdout = std::io::stdout().into_raw_mode()?;
        let screen = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(screen);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let mut liststate = ListState::default();
        liststate.select(Some(0));

        Ok(TerminalUI {
            liststate,
            input_state: None,
            terminal,
        })
    }

    fn get_user_input(&mut self, state: &State, question: &str) -> Result<String, std::io::Error> {
        let mut input = question.to_string();
        loop {
            if let Some(keypress) = self.get_next_keypress() {
                match keypress {
                    Key::Char(c) => input += &c.to_string(),
                    Key::Backspace => break,
                    _ => {}
                }
            }
            self.input_state = Some(input.clone());
            self.refresh(state)?;
        }
        self.input_state = None;
        Ok(input)
    }

    fn get_next_keypress(&mut self) -> Option<Key> {
        let stdin = std::io::stdin();
        for keypress in stdin.keys() {
            if let Ok(keypress) = keypress {
                return Some(keypress);
            }
        }
        None
    }

    fn refresh(&mut self, state: &State) -> Result<(), std::io::Error> {
        let pane = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::DarkGray));

        let textpane = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::DarkGray));

        self.liststate.select(state.get_idx());

        let _items = Self::get_text_list(state.get_current_fm_state(), state.list_current());
        let _prevs = Self::get_text_list(state.get_current_fm_state(), state.list_prev());
        let _nexts = Self::get_text_list(state.get_current_fm_state(), state.list_next());
        let text = if let Some(input) = &self.input_state {
            input.clone()
        } else {
            get_size(state.get_current_fm_state().get_focused())
        };
        let preview = state.get_current_fm_state().get_preview();
        let mut listwid = self.liststate.clone();

        self.terminal.draw(|mut f| {
            // TODO should probably move a good bit of widgeting out
            let horizontal_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(95), Constraint::Max(2)].as_ref())
                .split(f.size());

            let vertical_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(30),
                        Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(horizontal_split[0]);

            let list = Self::create_list_widget(_items.clone(), pane);
            let prevs_list = Self::create_list_widget(_prevs.clone(), pane);
            let nexts_list = Self::create_list_widget(_nexts.clone(), pane);

            if let Some(content) = preview {
                let huhu = [Text::raw(content)];
                let thing = Paragraph::new(huhu.iter()).block(pane);
                f.render_widget(thing, vertical_split[2]);
            } else {
                f.render_widget(nexts_list, vertical_split[2]);
            }
            let info_text = Text::raw(text);
            let text_arr = [info_text];
            let info_box = Paragraph::new(text_arr.iter())
                .block(textpane)
                .alignment(Alignment::Right)
                .wrap(false);

            f.render_widget(prevs_list, vertical_split[0]);
            f.render_stateful_widget(list, vertical_split[1], &mut listwid);

            f.render_widget(info_box, horizontal_split[1]);
        })?;
        Ok(())
    }
}

impl TerminalUI {
    fn get_text_list(fm_state: &FMState, init: Vec<PathBuf>) -> Vec<Text<'static>> {
        let mut list = Vec::new();
        for dir in init {
            let text = Self::get_style(&fm_state, &dir);
            list.push(text);
        }
        list
    }

    fn create_list_widget(
        text_list: Vec<Text<'static>>,
        pane: Block<'static>,
    ) -> List<'static, std::vec::IntoIter<Text<'static>>> {
        List::new(text_list.clone().into_iter())
            .block(pane)
            .style(Style::default().fg(Color::Blue))
            .highlight_symbol(" >")
            .highlight_style(Style::default().fg(Color::Red).modifier(Modifier::BOLD))
    }

    fn get_style(fm_state: &FMState, pathb: &PathBuf) -> Text<'static> {
        let mut filename = pathb
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        filename = String::from(" ") + filename.as_str();
        if &fm_state.get_currentdir() == pathb {
            // if path is parent dir of the current dir
            Text::styled(
                filename,
                Style::default().fg(Color::Red).modifier(Modifier::BOLD),
            )
        } else if fm_state.is_marked(pathb.to_path_buf()) {
            // if path is marked
            Text::styled(filename, Style::default().fg(Color::Yellow))
        } else if Filter::Dotfiles.is((&pathb).to_path_buf()) {
            // if path is dotfile
            Text::styled(filename, Style::default().fg(Color::DarkGray))
        } else if pathb.as_path().is_dir() {
            // if path is a dir
            Text::styled(filename, Style::default().fg(Color::Cyan))
        } else {
            // if path is a normal file
            Text::styled(filename, Style::default().fg(Color::Blue))
        }
    }
}
