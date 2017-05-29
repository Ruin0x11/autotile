use ui::elements::UiElement;
use ui::renderer::{TexDir, UiRenderer};

fn clamp(input: i32, min: i32, max: i32) -> i32 {
    if input > max {
        max
    }
    else if input < min {
        min
    } else {
        input
    }
}

pub struct UiBar {
    pos: (i32, i32),
    max: i32,
    current: i32,
    color: (u8, u8, u8, u8)
}

impl UiBar {
    pub fn new(pos: (i32, i32), max: i32, color: (u8, u8, u8, u8)) -> Self {
        assert!(max >= 0);

        UiBar {
            pos: pos,
            max: max,
            current: max / 2,
            color: color,
        }
    }

    pub fn set(&mut self, amount: i32) {
        self.current = clamp(amount, 0, self.max);
    }

    pub fn percent(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}


impl UiElement for UiBar {
    fn draw(&self, renderer: &mut UiRenderer) {
        let bar_portion = (258 as f32 * self.percent()) as u32;
        renderer.add_tex("bar", self.pos, None, (0, 30), (258, 30));

        renderer.with_color(self.color, |r| {
            r.add_tex("bar", self.pos, None, (0, 0), (bar_portion, 30));
        });

        let text = format!("{} / {}", self.current, self.max);
        let text_width = renderer.font().text_width_px(&text) as i32;

        let text_pos = (self.pos.0 + (258 / 2) - (text_width / 2),
                        self.pos.1 + 30 - (renderer.font().get_font_size() as i32));

        renderer.add_string_shadow(text_pos, None, (255, 255, 255, 255), &text);
    }
}
