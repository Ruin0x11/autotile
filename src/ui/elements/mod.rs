use ui::*;
use ui::renderer::*;

mod message;
pub use self::message::UiMessageLog;

pub trait UiElement {
    fn draw(&self, renderer: &mut UiRenderer);
}

impl UiElement for UiWindow {
    fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        let (w, h) = self.size;

        // center
        renderer.add_tex_stretch("win",
                                 (x as i32,     y as i32,
                                  (x + w) as i32, (y + h) as i32),
                                 None,
                                 (0, 0), (64, 64));

        renderer.repeat_tex("win", TexDir::Area,
                            (x,     y,
                            x + w,  y + h),
                            (0, 64), (64, 64));

        // corners
        renderer.add_tex("win",  (x as i32,              y as i32),               None, (64,  0), (16, 16));
        renderer.add_tex("win",  (x as i32,              (y + (h - 16)) as i32),  None, (64, 48), (16, 16));
        renderer.add_tex("win",  ((x + (w - 16)) as i32, y as i32),               None, (112, 0),  (16, 16));
        renderer.add_tex("win",  ((x + (w - 16)) as i32, (y + (h - 16)) as i32),  None, (112, 48), (16, 16));

        // borders
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 16,       y,            x + (w - 16), y + 16),       (80, 0),  (16, 16));
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 16,       y + (h - 16), x + (w - 16), y + h),        (80, 48), (16, 16));
        renderer.repeat_tex("win", TexDir::Vertical,   (x,            y + 16,       x + 16,       y + (h - 16)), (64, 16), (16, 16));
        renderer.repeat_tex("win", TexDir::Vertical,   (x + (w - 16), y + 16,       x + w,        y + (h - 16)), (112, 16), (16, 16));
    }
}

pub struct UiText {
    pub pos: (i32, i32),
    text_lines: Vec<String>,
}

impl UiText {
    pub fn new(pos: (i32, i32), text: &str) -> Self {
        let split = text.split("\n").map(|s| s.to_string()).collect::<Vec<String>>();
        UiText {
            pos: pos,
            text_lines: split,
        }
    }

    pub fn text(&self) -> String {
        self.text_lines.join("\n")
    }
}

impl UiElement for UiText {
    fn draw(&self, renderer: &mut UiRenderer) {
        for (idx, line) in self.text_lines.iter().enumerate() {
            let pos = (self.pos.0, self.pos.1 + (idx as u32 * renderer.get_font_size()) as i32);
            renderer.add_string(pos, None, (0, 0, 0, 255), line);
        }
    }
}

pub struct UiList {
    window: UiWindow,
    items: Vec<UiText>,
    selected: usize,
}

impl UiList {
    pub fn new(pos: (u32, u32), items: Vec<&str>) -> Self {
        let item_height = 20;
        let mut text_items = Vec::new();
        for (idx, item) in items.into_iter().enumerate() {
            let pos = (pos.0 as i32 + 32, pos.1 as i32 + 32 + (item_height * idx as u32) as i32);
            let text = UiText::new(pos, &item);
            text_items.push(text);
        }

        let win = UiWindow::new(pos);

        UiList {
            window: win,
            items: text_items,
            selected: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected == self.items.len() - 1 {
            return;
        }
        self.selected += 1;
    }

    pub fn select_prev(&mut self) {
        if self.selected == 0 {
            return;
        }
        self.selected -= 1;
    }

    pub fn get_selected(&self) -> Option<&UiText> {
        self.items.get(self.selected)
    }
}

impl UiElement for UiList {
    fn draw(&self, renderer: &mut UiRenderer) {
        self.window.draw(renderer);
        for item in self.items.iter() {
            item.draw(renderer);
        }
        if let Some(item) = self.get_selected() {
            let (ix, iy) = item.pos;
            renderer.add_tex("win", (ix - 16, iy - 12), None, (96, 24), (16, 16));
        }
    }
}
