use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    text::Line,
    widgets::{StatefulWidget, Widget},
};

use tui_big_text::BigText;

pub struct ReadingWindow {
    margin: (u16, u16),
}

#[derive(Default)]
pub struct ReadingWindowState {
    pub text: Vec<String>,
    pub is_reading: bool,
    pub line: u16,
}

impl ReadingWindowState {
    pub fn wrap_lines(
        &self,
        width: u16,
        height: u16,
        text: &Vec<String>,
        margin: (u16, u16),
    ) -> Vec<String> {
        let mut wrapped_lines = vec![String::new(); margin.1 as usize];
        let mut text_line = self.line;
        let mut line_words = Vec::new();
        while wrapped_lines.len() < height as usize && text_line < text.len() as u16 {
            let mut cur_line = String::from_utf8(vec![b' '; margin.0 as usize]).unwrap();
            if line_words.is_empty() {
                line_words = text[text_line as usize].split(' ').rev().collect();
            }
            while !line_words.is_empty()
                && line_words.last().unwrap().len() + cur_line.len()
                    < (width as usize - margin.0 as usize * 2)
            {
                if !cur_line.is_empty() {
                    cur_line.push(' ');
                }
                cur_line.push_str(line_words.pop().unwrap());
            }
            wrapped_lines.push(cur_line);
            if line_words.is_empty() {
                text_line += 1;
                wrapped_lines.push(String::new());
            }
        }
        wrapped_lines
    }

    pub fn new(text: Vec<String>, line: u16, is_reading: bool) -> ReadingWindowState {
        Self {
            text,
            line,
            is_reading,
        }
    }
}

impl ReadingWindow {
    pub fn new(margin: (u16, u16)) -> ReadingWindow {
        Self { margin }
    }
}

impl StatefulWidget for ReadingWindow {
    type State = ReadingWindowState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.is_reading {
            state.line = state.line.max(0);
            state.line = state.line.min(state.text.len() as u16 - 2);
            let lines = state.wrap_lines(area.width - 2, area.height - 2, &state.text, self.margin);
            for (i, line) in lines.into_iter().enumerate() {
                buf.set_line(
                    area.x + 1,
                    area.y + i as u16,
                    &Line::styled(line, Color::White),
                    area.width,
                );
            }
        } else {
            BigText::builder()
                .style(Color::White)
                .alignment(Alignment::Center)
                .lines(vec![
                    Line::styled("Royal", Color::White),
                    Line::styled("Rust", Color::White),
                ])
                .build()
                .unwrap()
                .render(
                    Rect {
                        x: area.x,
                        y: (area.y + area.height) / 2 - 7,
                        height: area.height,
                        width: area.width,
                    },
                    buf,
                );
        }
    }
}
