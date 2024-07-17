use itertools::Either;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::StatefulWidget,
};
use std::iter::zip;
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api::{ChapterReference, Fiction};

pub trait Listable {
    fn to_string(&self, width: u16, x_margin: u16) -> String;
}

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
const WEEK: u64 = 7 * DAY;
const MONTH: u64 = 30 * DAY;
const YEAR: u64 = 365 * DAY;

impl Listable for ChapterReference {
    fn to_string(&self, width: u16, x_margin: u16) -> String {
        let width = width - x_margin * 2;
        let s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64
            - self.time;
        let mut time = match s {
            YEAR.. => format!("{} years ago", s / YEAR),
            MONTH.. => format!("{} months ago", s / MONTH),
            WEEK.. => format!("{} weeks ago", s / WEEK),
            DAY.. => format!("{} days ago", s / DAY),
            HOUR.. => format!("{} hours ago", s / HOUR),
            MINUTE.. => format!("{} minutes ago", s / MINUTE),
            _ => format!("{} seconds ago", s),
        };
        let mut full_len = self.title.len() + 2 + time.len();
        if full_len > width as usize {
            let words: Vec<&str> = time.split(' ').collect();
            time = format!("{} {}", words[0], words[1]);
            full_len -= 4;
        }
        let spacing_width = (width as i32 - full_len as i32).max(2);
        let spacing = String::from_utf8(vec![b' '; spacing_width as usize]).unwrap();
        let name = self
            .title
            .chars()
            .take(width as usize - 2 * x_margin as usize - time.len() - spacing_width as usize)
            .collect::<String>();
        format!("{}{}{}", name, spacing, time)
    }
}

impl Listable for Fiction {
    fn to_string(&self, width: u16, x_margin: u16) -> String {
        if self.title.len() as u16 > width - 2 - 2 * x_margin {
            format!(
                "{}...",
                self.title
                    .chars()
                    .take((width - 5 - 2 * x_margin) as usize)
                    .collect::<String>()
            )
        } else {
            self.title.clone()
        }
    }
}

pub struct ListWidget<T: Listable> {
    phantom: PhantomData<T>,
    margin: (u16, u16),
}

#[derive(Debug)]
pub struct ListState<T: Listable> {
    pub selected_line: u16,
    pub top_line: u16,
    pub items: Vec<T>,
    pub reversed: bool,
}

impl<T: Listable> ListState<T> {
    pub fn new(items: Vec<T>, selected_line: u16, top_line: u16) -> ListState<T> {
        Self {
            items,
            selected_line,
            top_line,
            reversed: false,
        }
    }
}

impl<T: Listable> ListWidget<T> {
    pub fn new(margin: (u16, u16)) -> ListWidget<T> {
        Self {
            phantom: PhantomData,
            margin,
        }
    }
}

impl<T: Listable> StatefulWidget for ListWidget<T> {
    type State = ListState<T>;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // state not validated in event handler
        if state.items.is_empty() {
            return;
        }
        state.selected_line = state.selected_line.max(0);
        state.selected_line = state.selected_line.min(state.items.len() as u16 - 1);
        state.top_line = state.top_line.min(state.selected_line);
        state.top_line = state.top_line.max(
            (state.selected_line as i32 - area.height as i32 - 3 - self.margin.1 as i32 * 2).max(0)
                as u16,
        );
        let num_entries = area.height - 2 - self.margin.1 * 2;
        let end = (state.top_line + num_entries).min(state.items.len() as u16);
        let index_iter = state.top_line..end;
        let item_iter = if state.reversed {
            Either::Left((state.top_line..end).rev())
        } else {
            Either::Right(state.top_line..end)
        }
        .map(|i| &state.items[i as usize]);
        for (i, item) in zip(index_iter, item_iter) {
            let style = if i as u16 == state.selected_line {
                Style::default().fg(Color::Black).bg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };
            buf.set_line(
                area.x + 1 + self.margin.0,
                area.y + self.margin.1 + 1 + i - state.top_line,
                &Line::styled(item.to_string(area.width, self.margin.0 + 1), style),
                area.width,
            );
        }
    }
}
