use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    text::Line,
    widgets::{StatefulWidget, Widget},
};

use tui_big_text::BigText;

pub struct FictionInWidget {
    margin: (u16, u16),
}

#[derive(Default)]
pub struct FictionInState {
    pub text: String,
}

impl FictionInState {
    pub fn new(text: String) -> FictionInState {
        Self { text }
    }
}

impl FictionInWidget {
    pub fn new(margin: (u16, u16)) -> FictionInWidget {
        Self { margin }
    }
}

impl StatefulWidget for FictionInWidget {
    type State = FictionInState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.line = state.line.clamp(0, state.text.len() as u16 - 2);
        let lines = state.wrap_lines(area.width - 2, area.height - 2, &state.text, self.margin);
        for (i, line) in lines.into_iter().enumerate() {
            buf.set_line(
                area.x + 1,
                area.y + i as u16,
                &Line::styled(line, Color::White),
                area.width,
            );
        }
    }
}
