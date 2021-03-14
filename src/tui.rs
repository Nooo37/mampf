use std::process::Command;
use std::path::PathBuf;
use termion::{event::Key, input::TermRead, raw::IntoRawMode,
              screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style, Modifier};
use tui::widgets::{Text, List, ListState, Block, Borders, BorderType, Paragraph};

use crate::util::{KeyState, Filter};
use crate::state::FMState;


fn get_text_list(fm_state: &FMState, init: Vec<PathBuf>) -> Vec<Text<'static>>{
    let mut list = Vec::new();
    for dir in init {
        let text = get_style(&fm_state, &dir);
        list.push(text);
    }
    list
}

fn handle_keybinds(c: Key, fm_state: &mut FMState, ks: &mut KeyState,
                   list_state: &mut ListState) -> Option<()> {
    let actions = ks.press(c)?;
    for action in actions {
        Command::new("notify-send").arg("'huhu'");
        let new_idx = action.perform(fm_state);
        if let Some(new_idx) = new_idx {
            list_state.select(Some(new_idx));
        }
    }
    Some(())
}

fn create_list_widget(text_list: Vec<Text<'static>>, pane: Block<'static>)
                      -> List<'static, std::vec::IntoIter<Text<'static>>> {
    List::new(text_list.clone().into_iter())
        .block(pane)
        .style(Style::default().fg(Color::Blue))
        .highlight_symbol(" > ")
        .highlight_style(Style::default().fg(Color::Red).modifier(Modifier::BOLD))
}

fn get_style(fm_state: &FMState, pathb: &PathBuf) -> Text<'static> {
    let filename = pathb.file_name().unwrap().to_os_string().into_string().unwrap();
    if &fm_state.get_currentdir() == pathb { // if path is parent dir of the current dir 
        return Text::styled(filename, Style::default().fg(Color::Red).modifier(Modifier::BOLD))
    } else if fm_state.is_marked(pathb.to_path_buf()) { // if path is marked
        return Text::styled(filename, Style::default().fg(Color::Yellow))
    } else if Filter::Dotfiles.is((&pathb).to_path_buf()) { // if path is dotfile 
        return Text::styled(filename, Style::default().fg(Color::DarkGray));
    } else if pathb.as_path().is_dir() { // if path is a dir 
        return Text::styled(filename, Style::default().fg(Color::Cyan));
    } else { // if path is a normal file
        return Text::styled(filename, Style::default().fg(Color::Blue));
    }
}


pub fn main(mut fm_state: FMState, mut ks: KeyState) -> Result<(), std::io::Error> {

    // all the terminal backend initalization
    let stdout = std::io::stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // some widget declaration that can happen outside the main loop
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let mut _items;
    let mut _prevs;
    let mut _nexts;

    let pane = Block::default()
        .borders( Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::DarkGray));

    // main loop
    while !fm_state.is_exit() {

        fm_state.update_by_idx(list_state.selected());

        _items = get_text_list(&fm_state, fm_state.list_current());
        _prevs = get_text_list(&fm_state, fm_state.list_prev());
        _nexts = get_text_list(&fm_state, fm_state.list_next());

        terminal.draw(|mut f| { 
            // TODO should probably move a good bit of widgeting out 
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(35),
                        Constraint::Percentage(40),
                    ]
                        .as_ref(),
                )
                .split(f.size());

            let list = create_list_widget(_items.clone(), pane);
            let prevs_list = create_list_widget(_prevs.clone(), pane);
            let nexts_list = create_list_widget(_nexts.clone(), pane);

            let wiwi = fm_state.get_preview();
            if let Some(content) = wiwi {
                let huhu = [
                    Text::raw(content)
                ];
                let thing = Paragraph::new(huhu.iter()).block(pane);
                f.render_widget(thing, chunks[2]);
            } else {
                f.render_widget(nexts_list, chunks[2]);
                
            }

            f.render_widget(prevs_list, chunks[0]);
            f.render_stateful_widget(list, chunks[1], &mut list_state);
        })?;
        let stdin = std::io::stdin();

        for c in stdin.keys() {
            handle_keybinds(c.unwrap(), &mut fm_state, &mut ks, &mut list_state);
            break;
        }
    }
    return Ok(());
}
