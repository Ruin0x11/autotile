use ui::elements::UiElement;
use ui::renderer::{TexDir, UiRenderer};

pub struct UiMessageLog {
    pos: (u32, u32),
    size: (u32, u32),
}

impl UiMessageLog {
    pub fn new() -> Self {
        UiMessageLog {
            pos: (0, 480),
            size: (800, 120),
        }
    }
}

impl UiElement for UiMessageLog {
    fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        let (w, h) = self.size;
        println!("asd!");

        renderer.repeat_tex("textwin", TexDir::Area,
                            (x,     y,
                            x + w,  y + h),
                            (0, 0), (46, 45));
    }
}
