use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::StatefulWidget,
};

use crate::api::ChapterReference;

pub struct ListWidget {
    margin: (u16, u16),
}

#[derive(Debug)]
pub struct ListState {
    pub selected_line: u16,
    pub top_line: u16,
    pub items: Vec<ChapterReference>,
}

impl ListState {
    pub fn new(chapters: Vec<ChapterReference>, selected_line: u16, top_line: u16) -> ListState {
        Self {
            items: chapters,
            selected_line,
            top_line,
        }
    }
}

impl ListWidget {
    pub fn new(margin: (u16, u16)) -> ListWidget {
        Self { margin }
    }
}

impl StatefulWidget for ListWidget {
    type State = ListState;
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
        // dbg!(state.top_line);
        let num_entries = area.height - 2 - self.margin.1 * 2;
        for i in state.top_line..(state.top_line + num_entries).min(state.items.len() as u16) {
            let style = if i as u16 == state.selected_line {
                Style::default().fg(Color::Black).bg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };
            buf.set_line(
                area.x + 1 + self.margin.0,
                area.y + self.margin.1 + 1 + i - state.top_line,
                &Line::styled(
                    state.items[i as usize].to_string(area.width, self.margin.0 + 1),
                    style,
                ),
                area.width,
            );
        }
    }
}
