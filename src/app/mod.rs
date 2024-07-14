use std::io::{stdout, Result, Stdout};

use chap_list::{ChapterList, ChapterListState};
use reading_window::{ReadingWindow, ReadingWindowState};

use crate::api::{Chapter, RoyalClient};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Constraint, Direction, Layout},
    style::Color,
    widgets::{Block, Borders},
    Terminal,
};
mod chap_list;
mod reading_window;

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    client: RoyalClient,
    reading_state: ReadingWindowState,
    is_reading: bool,
    chapter_state: ChapterListState,
}

impl App {
    pub fn new() -> Result<App> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(App {
            terminal: Terminal::new(CrosstermBackend::new(stdout()))?,
            client: RoyalClient::new(),
            reading_state: ReadingWindowState::default(),
            chapter_state: ChapterListState::new(Vec::new(), 0, 0),
            is_reading: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let chapters = self.client.get_fiction(40920).unwrap().chapters;
        self.chapter_state.chapters = chapters;
        loop {
            self.terminal.draw(|frame| {
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                    ])
                    .split(frame.size());
                frame.render_widget(
                    Block::new()
                        .title("Fictions")
                        .style(Color::Blue)
                        .borders(Borders::ALL),
                    layout[0],
                );
                frame.render_widget(
                    Block::new()
                        .title("Chapters")
                        .style(Color::Blue)
                        .borders(Borders::ALL),
                    layout[1],
                );
                frame.render_stateful_widget(
                    ChapterList::new((2, 1)),
                    layout[1],
                    &mut self.chapter_state,
                );
                frame.render_widget(
                    Block::new()
                        .title("Content")
                        .style(Color::Blue)
                        .borders(Borders::ALL),
                    layout[2],
                );

                frame.render_stateful_widget(
                    ReadingWindow::new((3, 2)),
                    layout[2],
                    &mut self.reading_state,
                )
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if self.handle_key(key) {
                        break;
                    }
                }
            }
        }
        self.terminal.clear()?;
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Press {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                match key.code {
                    KeyCode::Char('J') => {
                        self.chapter_state.selected_line += 1;
                    }
                    KeyCode::Char('K') => {
                        self.chapter_state.selected_line -= 1;
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => {
                        return true;
                    }
                    KeyCode::Char('j') => {
                        self.reading_state.line += 1;
                    }
                    KeyCode::Char('k') => {
                        // overflow mega sadge
                        self.reading_state.line = self.reading_state.line.max(1) - 1;
                    }
                    KeyCode::Enter => {
                        self.reading_state.is_reading = true;
                        self.reading_state.text = Chapter::from_reference(
                            &self.chapter_state.chapters[self.chapter_state.selected_line as usize],
                            &self.client,
                        )
                        .unwrap()
                        .content;
                    }
                    _ => {}
                }
            }
        }
        false
    }
}
