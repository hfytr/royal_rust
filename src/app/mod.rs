use std::io::{stdout, Result, Stdout};

use chap_list::{ListState, ListWidget};
use reading_window::{ReadingWindow, ReadingWindowState};

use crate::api::{Chapter, ChapterReference, Fiction, RoyalClient};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Widget},
    Terminal,
};
mod chap_list;
mod reading_window;

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    client: RoyalClient,
    reading_state: ReadingWindowState,
    fiction_state: ListState<Fiction>,
    chapter_state: ListState<ChapterReference>,
    fictions_showing: bool,
    fiction_in: Option<String>,
}

impl App {
    pub fn new() -> Result<App> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut app = App {
            terminal: Terminal::new(CrosstermBackend::new(stdout()))?,
            client: RoyalClient::new(),
            reading_state: ReadingWindowState::default(),
            fiction_state: ListState::new(Vec::new(), 0, 0),
            chapter_state: ListState::new(Vec::new(), 0, 0),
            fictions_showing: true,
            fiction_in: None,
        };
        Ok(app)
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|frame| {
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
                    .split(frame.size());
                let title = if self.fictions_showing {
                    "Fictions"
                } else {
                    "Chapters"
                };
                frame.render_widget(
                    Block::new()
                        .title(title)
                        .style(Color::Blue)
                        .borders(Borders::ALL),
                    layout[0],
                );
                if self.fictions_showing {
                    frame.render_stateful_widget(
                        ListWidget::new((1, 2)),
                        layout[0],
                        &mut self.fiction_state,
                    );
                } else {
                    frame.render_stateful_widget(
                        ListWidget::new((1, 2)),
                        layout[0],
                        &mut self.chapter_state,
                    );
                }

                frame.render_widget(
                    Block::new()
                        .title("Content")
                        .style(Color::Blue)
                        .borders(Borders::ALL),
                    layout[1],
                );

                frame.render_stateful_widget(
                    ReadingWindow::new((3, 2)),
                    layout[1],
                    &mut self.reading_state,
                );

                if self.fiction_in.is_some() {
                    let x = frame.size().width / 2 - 35;
                    let y = frame.size().height / 2 - 2;
                    Block::new()
                        .title("Fiction Input")
                        .style(Style::default().bg(Color::Reset).fg(Color::White))
                        .borders(Borders::ALL)
                        .render(
                            Rect {
                                width: 70,
                                height: 4,
                                x,
                                y,
                            },
                            frame.buffer_mut(),
                        );
                    frame.buffer_mut().set_line(
                        x + 2,
                        y + 2,
                        &Line::styled(self.fiction_in.as_ref().unwrap(), Color::White),
                        17,
                    );
                }
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && self.handle_key(key) {
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
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Char('J') => {
                    if self.fictions_showing {
                        self.fiction_state.selected_line += 1;
                    } else {
                        self.chapter_state.selected_line += 1;
                    }
                }
                KeyCode::Char('K') => {
                    if self.fictions_showing {
                        self.fiction_state.selected_line -= 1;
                    } else {
                        self.chapter_state.selected_line -= 1;
                    }
                }
                _ => {}
            }
        } else if self.fiction_in.is_some() {
            match key.code {
                KeyCode::Char(c) => {
                    if c.is_digit(10) {
                        if self.fiction_in.as_ref().unwrap().parse::<f64>().is_ok() {
                            self.fiction_in.as_mut().unwrap().push(c);
                        } else {
                            // nothing input yet, default message
                            *self.fiction_in.as_mut().unwrap() = String::from(c);
                        }
                    }
                }
                KeyCode::Esc => {
                    self.fiction_in = None;
                }
                KeyCode::Enter => {
                    match self
                        .client
                        .get_fiction(self.fiction_in.as_ref().unwrap().parse::<usize>().unwrap())
                    {
                        Some(x) => {
                            self.fiction_in = None;
                            self.fiction_state.items.push(x);
                        }
                        None => {
                            self.fiction_in = Some(String::from("Invalid ID"));
                        }
                    }
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') => {
                    Fiction::write_to_file(
                        "~/.cache/royal-rust/fictions.txt",
                        &self.fiction_state.items,
                    )
                    .expect("failed to save fictions");
                    return true;
                }
                KeyCode::Char('j') => {
                    self.reading_state.line += 1;
                }
                KeyCode::Char('k') => {
                    // overflow mega sadge
                    self.reading_state.line = self.reading_state.line.max(1) - 1;
                }
                KeyCode::Char('l') => {
                    if self.fictions_showing {
                        self.fictions_showing = false;
                        self.chapter_state.items = self.fiction_state.items;
                    } else {
                        self.reading_state.is_reading = true;
                        self.reading_state.text = Chapter::from_reference(
                            &self.chapter_state.items[self.chapter_state.selected_line as usize],
                            &self.client,
                        )
                        .unwrap()
                        .content;
                    }
                }
                KeyCode::Char('h') => {
                    self.fictions_showing = true;
                }
                KeyCode::Char('o') => {
                    self.fictions_showing = true;
                    self.fiction_in = Some(String::from("Enter New Fiction ID"));
                }
                _ => {}
            }
        }
        false
    }
}
