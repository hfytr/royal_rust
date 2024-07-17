use std::io::{stdout, Result, Stdout};

use chap_list::{ListState, ListWidget};
use reading_window::{ReadingWindow, ReadingWindowState};

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
    Frame, Terminal,
};
use royal_api::{Chapter, ChapterReference, Fiction, RoyalClient};
mod chap_list;
mod reading_window;

pub struct App {
    client: RoyalClient,
    reading_state: ReadingWindowState,
    fiction_state: ListState<Fiction>,
    chapter_state: ListState<ChapterReference>,
    fictions_showing: bool,
    fiction_in: Option<String>,
}

impl App {
    pub fn new() -> Result<App> {
        let path = format!(
            "{}/.cache/royal-rust/fictions.txt",
            home::home_dir().unwrap().to_str().unwrap()
        );
        let client = RoyalClient::new();
        let fiction_vec = Fiction::from_file(&client, &path).unwrap_or(Vec::new());
        let app = App {
            client,
            reading_state: ReadingWindowState::default(),
            fiction_state: ListState::new(fiction_vec, 0, 0),
            chapter_state: ListState::new(Vec::new(), 0, 0),
            fictions_showing: true,
            fiction_in: None,
        };
        Ok(app)
    }

    pub fn run(&mut self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && self.handle_key(key) {
                        break;
                    }
                }
            }
        }
        terminal.clear()?;
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
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
                ListWidget::new((1, 1)),
                layout[0],
                &mut self.fiction_state,
            );
        } else {
            frame.render_stateful_widget(
                ListWidget::new((1, 1)),
                layout[0],
                &mut self.chapter_state,
            );
        }

        let title = if self.reading_state.is_reading {
            format!(
                " {} - {} ",
                self.fiction_state.items[self.get_fiction_ind()].title,
                self.chapter_state.items[self.get_chapter_ind()].title,
            )
        } else {
            String::new()
        };
        frame.render_widget(
            Block::new()
                .title(title)
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
                        // prevent underflow
                        self.fiction_state.selected_line = self.fiction_state.selected_line.max(1);
                        self.fiction_state.selected_line -= 1;
                    } else {
                        self.chapter_state.selected_line = self.chapter_state.selected_line.max(1);
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
                KeyCode::Backspace => {
                    self.fiction_in.as_mut().unwrap().pop();
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
                    let path = format!(
                        "{}/.cache/royal-rust/fictions.txt",
                        home::home_dir().unwrap().to_str().unwrap()
                    );
                    Fiction::write_to_file(&path, &self.fiction_state.items)
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
                KeyCode::Char('d') => {
                    self.fiction_state.items.remove(self.get_item_ind());
                }
                KeyCode::Char('l') => {
                    let item_ind = self.get_item_ind();
                    if self.fictions_showing && !self.fiction_state.items.is_empty() {
                        self.fictions_showing = false;
                        self.chapter_state.items =
                            self.fiction_state.items[item_ind].chapters.clone();
                    } else if !self.fictions_showing && !self.chapter_state.items.is_empty() {
                        self.reading_state.is_reading = true;
                        self.reading_state.text = Chapter::from_reference(
                            &self.chapter_state.items[item_ind],
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
                KeyCode::Char('r') => {
                    if self.fictions_showing {
                        self.fiction_state.reversed = !self.fiction_state.reversed;
                    } else {
                        self.chapter_state.reversed = !self.chapter_state.reversed;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn get_item_ind(&self) -> usize {
        if self.fictions_showing {
            self.get_fiction_ind()
        } else {
            self.get_chapter_ind()
        }
    }

    fn get_chapter_ind(&self) -> usize {
        if self.chapter_state.reversed {
            self.chapter_state.items.len() - 1 - self.chapter_state.selected_line as usize
        } else {
            self.chapter_state.selected_line as usize
        }
    }

    fn get_fiction_ind(&self) -> usize {
        if self.fiction_state.reversed {
            self.fiction_state.items.len() - 1 - self.fiction_state.selected_line as usize
        } else {
            self.fiction_state.selected_line as usize
        }
    }
}
