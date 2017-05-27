use glium;
use glium::glutin;
use glium::backend::Facade;

use render::{Renderable, Viewport};

pub mod elements;
mod layer;
mod renderer;

pub use self::elements::{UiElement};
pub use self::renderer::UiRenderer;
pub use self::layer::{EventResult, UiLayer, UiQuery};

// 1. update state somehow
// 2. output vertices of texture coordinates inside UI texture atlas

// self.bar.update(BarData { current: 100, max: 1000 }):
// drawlist.extend(self.bar.output());

// For text, use the font atlas and output one texture piece for each glyph

pub struct Ui {
    renderer: UiRenderer,
    valid: bool,
    layers: Vec<Box<UiLayer>>,
}

impl Ui {
    pub fn new<F: Facade>(display: &F) -> Self {
        Ui {
            renderer: UiRenderer::new(display),
            valid: true,
            layers: Vec::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        !self.layers.is_empty()
    }

    pub fn draw_layer<T: 'static + UiLayer>(&mut self, layer: &T) {
        layer.draw(&mut self.renderer)
    }

    pub fn push_layer<T: 'static + UiLayer>(&mut self, layer: T) {
        self.layers.push(Box::new(layer));
        self.invalidate();
    }

    pub fn pop_layer(&mut self) {
        self.layers.pop();
        self.invalidate();
    }

    pub fn clear(&mut self) {
        self.renderer.clear();
        self.valid = true;
    }

    pub fn update(&mut self, event: glutin::Event) {
        let result = match self.layers.last_mut() {
            None => EventResult::Ignored,
            Some(layer) => layer.on_event(event),
        };

        match result {
            EventResult::Ignored => (),
            EventResult::Consumed(callback) => {
                self.invalidate();
                match callback {
                    None => (),
                    Some(cb) => cb(self)
                }
            }
            EventResult::Done => self.pop_layer(),
        }

        self.redraw();
    }

    fn redraw(&mut self) {
        if !self.valid {
            self.renderer.clear();
            for layer in self.layers.iter() {
                layer.draw(&mut self.renderer);
            }

            self.valid = true;
        }
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
        println!("invalidate ui");
    }
}

impl<'a> Renderable for Ui {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        self.renderer.render(display, target, viewport, msecs);
    }
}

pub struct UiWindow {
    pos: (u32, u32),
    size: (u32, u32),
}

impl UiWindow {
    pub fn new(pos: (u32, u32)) -> Self {
        UiWindow {
            pos: pos,
            size: (300, 400),
        }
    }
}
