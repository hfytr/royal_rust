use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{StatefulWidget, Widget},
};

use crate::api::ChapterReference;

pub struct ChapterList {
    chapters: Vec<ChapterReference>,
    margin: (u16, u16),
}

#[derive(Debug)]
pub struct ChapterListState {
    pub selected_line: u16,
    pub top_line: u16,
}

impl ChapterListState {
    pub fn new(selected_line: u16, top_line: u16) -> ChapterListState {
        Self {
            selected_line,
            top_line,
        }
    }
}

impl ChapterList {
    pub fn new(chapters: Vec<ChapterReference>, margin: (u16, u16)) -> ChapterList {
        Self { chapters, margin }
    }
}

impl StatefulWidget for ChapterList {
    type State = ChapterListState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // state not validated in event handler
        state.selected_line = state.selected_line.max(0);
        state.selected_line = state.selected_line.min(self.chapters.len() as u16 - 1);
        state.top_line = state.top_line.min(state.selected_line);
        state.top_line = state.top_line.max(
            (state.selected_line as i32 - area.height as i32 - 2 - self.margin.1 as i32 * 2).max(0)
                as u16,
        );
        let num_entries = area.height - 2 - self.margin.1 * 2;
        for i in state.top_line..(state.top_line + num_entries).min(self.chapters.len() as u16) {
            let style = if i as u16 == state.selected_line {
                Style::default().fg(Color::Black).bg(Color::Blue)
            } else {
                Style::default().fg(Color::Blue)
            };
            buf.set_line(
                area.x + 1,
                area.y + self.margin.1 + 1 + i - state.top_line,
                &Line::styled(
                    self.chapters[i as usize].to_string(area.width, self.margin.0 + 1),
                    style,
                ),
                area.width,
            );
        }
    }
}
