use std::collections::VecDeque;

use ui::elements::UiElement;
use ui::renderer::{TexDir, UiRenderer};

const LINE_HEIGHT: usize =  16;

pub struct UiMessageLog {
    pos: (u32, u32),
    size: (u32, u32),

    log: VecDeque<String>,

    next_line: bool,
}

impl UiMessageLog {
    pub fn new() -> Self {
        UiMessageLog {
            pos: (0, 480),
            size: (800, 120),

            log: VecDeque::new(),

            next_line: true,
        }
    }

    pub fn clear(&mut self) {
        self.log.clear();
    }

    pub fn append(&mut self, text: &str) {
        if self.next_line {
            self.log.push_front(String::new());
            self.next_line = false;
        }

        let mut current = match self.log.pop_front() {
            Some(line) => line,
            None       => String::new(),
        };

        current.push_str(text);
        current.push_str(" ");

        self.log.push_front(current);
    }

    pub fn next_line(&mut self) {
        self.next_line = true;
    }

    pub fn max_lines(&self) -> usize {
        self.size.1 as usize / LINE_HEIGHT
    }
}

impl UiElement for UiMessageLog {
    fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        let (w, h) = self.size;
        println!("asd!");

        renderer.with_color((128, 128, 128, 255), |r| {
            r.repeat_tex("textwin", TexDir::Area,
                         (x,     y,
                          x + w,  y + h),
                         (0, 0), (46, 45));
        });

        let (tx, ty) = (x as i32 + 8, (y + h - 8) as i32);

        let max_lines = self.max_lines();
        let mut idx = 0;

        for line in self.log.iter() {
            if !line.is_empty() {
                for wrapped in renderer.font().wrap_text(line, w - 16) {
                    let offset = (idx * LINE_HEIGHT) as i32;

                    renderer.add_string_shadow((tx, ty - offset), None, (255, 255, 255, 255), &wrapped);

                    idx += 1;

                    if idx >= max_lines {
                        return;
                    }
                }
            }
        }
    }
}
