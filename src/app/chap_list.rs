use itertools::Either;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::StatefulWidget,
};
use std::fmt::Debug;
use std::iter::zip;
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

use royal_api::{ChapterReference, Fiction};

pub trait Listable: Debug {
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
        let time = match s {
            YEAR.. => format!("{} years", s / YEAR),
            MONTH.. => format!("{} months", s / MONTH),
            WEEK.. => format!("{} weeks", s / WEEK),
            DAY.. => format!("{} days", s / DAY),
            HOUR.. => format!("{} hours", s / HOUR),
            MINUTE.. => format!("{} minutes", s / MINUTE),
            _ => format!("{} seconds", s),
        };
        let full_len = self.title.len() + time.len();
        let spacing_width = (width as i32 - full_len as i32).max(3);
        let spacing = String::from_utf8(vec![b' '; spacing_width as usize]).unwrap();
        let name = self
            .title
            .chars()
            .take(width as usize - time.len() - spacing_width as usize)
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
            reversed: true,
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
        state.selected_line = state.selected_line.clamp(0, state.items.len() as u16 - 1);
        state.top_line = state.top_line.clamp(
            (state.selected_line as i32 - area.height as i32 - 3 - self.margin.1 as i32 * 2).max(0)
                as u16,
            state.selected_line,
        );
        let num_entries =
            (area.height - 2 - self.margin.1 * 2).min(state.items.len() as u16 - state.top_line);
        let line_num = (state.top_line..).take(num_entries as usize);
        let item_iter = if state.reversed {
            Either::Left(state.items.iter().rev())
        } else {
            Either::Right(state.items.iter())
        }
        .take(num_entries as usize);
        for (i, item) in zip(line_num, item_iter) {
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
