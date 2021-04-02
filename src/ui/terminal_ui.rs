use crate::{
    app::App,
    ui::UI,
    util::{get_size, EntryStyle, PaneContent, PaneRole},
};
use std::{io::Stdout, path::PathBuf};
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, ToAlternateScreen, ToMainScreen},
};
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListState, Paragraph, Text},
    Frame, Terminal,
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

    fn get_user_input(&mut self, state: &App, question: &str) -> Result<String, std::io::Error> {
        let mut input = String::new();
        self.input_state = Some(question.to_string());
        self.refresh(state)?;
        loop {
            match self.get_next_keypress() {
                Key::Char(c) => {
                    if c == '\n' {
                        break;
                    } else {
                        input += &c.to_string()
                    }
                }
                Key::Backspace => {
                    input.pop();
                }
                _ => {}
            }
            self.input_state = Some(question.to_string() + &input);
            self.refresh(state)?;
        }
        self.input_state = None;
        Ok(input)
    }

    fn get_next_keypress(&mut self) -> Key {
        let stdin = std::io::stdin();
        for keypress in stdin.keys() {
            if let Ok(keypress) = keypress {
                return keypress;
            }
        }
        // The function will be stuck in the loop until a key is pressed.
        // This return statement shouldn't come into play
        Key::Null
    }

    fn refresh(&mut self, state: &App) -> Result<(), std::io::Error> {
        let pane = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::DarkGray));

        let textpane = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::DarkGray));

        let mut liststate = ListState::default();
        liststate.select(state.get_idx());

        let text = if let Some(input) = &self.input_state {
            input.clone()
        } else {
            get_size(state.get_current_fm_state().get_focused())
        };

        self.terminal.draw(|mut f| {
            // TODO should probably move a good bit of widgeting out
            let horizontal_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(95), Constraint::Min(2)].as_ref())
                .split(f.size());

            let mut contraints = Vec::new();
            for pane_entry in state.config.panes.iter() {
                contraints.push(Constraint::Percentage(pane_entry.width.into()));
            }
            let vertical_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(contraints.as_ref())
                .split(horizontal_split[0]);

            let info_text = Text::raw(text);
            let text_arr = [info_text];
            let info_box = Paragraph::new(text_arr.iter())
                .block(textpane)
                .alignment(Alignment::Right)
                .wrap(false);
            f.render_widget(info_box, horizontal_split[1]);
            // Self::render_content(&mut f, state.get_content_right(), vertical_split[2], pane);
            // Self::render_content(&mut f, state.get_content_left(), vertical_split[0], pane);
            for (idx, pane_config) in state.config.panes.iter().enumerate() {
                match &pane_config.role {
                    PaneRole::Current => {
                        if let Some(list) =
                            Self::create_current_widget(state.get_content_middle(), pane)
                        {
                            f.render_stateful_widget(list, vertical_split[idx], &mut liststate);
                        }
                    }
                    other => {
                        Self::render_content(
                            &mut f,
                            state.get_content(other.clone()),
                            vertical_split[idx],
                            pane,
                        );
                    }
                }
            }
        })?;
        Ok(())
    }
}

// Some helper functions to move a chunk of the widgeting out of the refresh function

impl TerminalUI {
    fn render_content<B: Backend>(f: &mut Frame<B>, content: PaneContent, rect: Rect, pane: Block) {
        match content {
            PaneContent::DirElements(ele_vec) => {
                let list = List::new(ele_vec.iter().map(|x| Self::translate_style(&x.0, &x.1)))
                    .block(pane);
                f.render_widget(list, rect);
            }
            PaneContent::Text(text) => {
                let text = Text::raw(text);
                let arr = [text];
                let para = Paragraph::new(arr.iter()).block(pane);
                f.render_widget(para, rect);
            }
            _ => {
                f.render_widget(pane, rect);
            }
        }
    }

    fn create_current_widget(
        content: PaneContent,
        pane: Block<'static>,
    ) -> Option<List<'static, std::vec::IntoIter<Text<'static>>>> {
        match content {
            PaneContent::DirElements(ele_vec) => Some(
                List::new(
                    ele_vec
                        .iter()
                        .map(|x| Self::translate_style(&x.0, &x.1))
                        .collect::<Vec<Text>>()
                        .into_iter(),
                )
                .block(pane)
                .style(Style::default().fg(Color::Blue))
                .highlight_symbol(" > ")
                .highlight_style(Style::default().fg(Color::Red).modifier(Modifier::BOLD)),
            ),
            _ => None,
        }
    }

    fn translate_style(pathb: &PathBuf, style: &EntryStyle) -> Text<'static> {
        let mut filename = pathb
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        filename = String::from(" ") + filename.as_str(); // cheap trick to get some padding to the left of the lists
        match style {
            // Not complete yet, there might be a change once more customization is introduced
            EntryStyle::Red => Text::styled(filename, Style::default().fg(Color::Red)),
            EntryStyle::Blue => Text::styled(filename, Style::default().fg(Color::Blue)),
            EntryStyle::Yellow => Text::styled(filename, Style::default().fg(Color::Yellow)),
            EntryStyle::Cyan => Text::styled(filename, Style::default().fg(Color::Cyan)),
            _ => Text::styled(filename, Style::default()),
        }
    }

    pub fn tui_app_start(&mut self) -> Result<(), std::io::Error> {
        // right before the screen switches, the AlternateScreen is overdrawn
        // with nothing thus forcing the TUI library to redraw the whole screen
        // on the next draw function call
        self.terminal.draw(|_f| {})?;
        print!("{}", ToMainScreen);
        Ok(())
    }

    pub fn tui_app_end(&mut self) -> Result<(), std::io::Error> {
        print!("{}", ToAlternateScreen);
        self.terminal.hide_cursor()?;
        Ok(())
    }
}
