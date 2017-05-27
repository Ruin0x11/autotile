use std::time::Duration;

use glium;
use glium::glutin;
use glium::{DisplayBuild, Surface};
use glium::backend::glutin_backend::GlutinFacade;


use board::Board;
use ui::*;
use util;
use self::background::Background;
use self::spritemap::SpriteMap;
use self::tilemap::TileMap;

mod background;
mod spritemap;
mod tilemap;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];
pub const QUAD: [Vertex; 4] = [
    Vertex { position: [0, 1], },
    Vertex { position: [1, 1], },
    Vertex { position: [0, 0], },
    Vertex { position: [1, 0], },
];

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [i32; 2],
}

implement_vertex!(Vertex, position);

#[derive(Debug)]
pub struct Viewport {
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub scale: f32,
    pub camera: (i32, i32),
}

pub struct RenderContext {
    backend: GlutinFacade,
    ui: Ui,
    background: Background,
    spritemap: SpriteMap,
    tilemap: TileMap,
    pub viewport: Viewport,
}

impl RenderContext {
    pub fn new() -> Self{
        let display = glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
            .build_glium()
            .unwrap();

        let bg = Background::new(&display);
        let ui = Ui::new(&display);
        let tile = TileMap::new(&display);
        let sprite = SpriteMap::new(&display);

        let scale = display.get_window().unwrap().hidpi_factor();

        let viewport = Viewport {
            position: (0, 0),
            size: (SCREEN_WIDTH, SCREEN_HEIGHT),
            scale: scale,
            camera: (0, 0)
        };

        RenderContext {
            backend: display,
            background: bg,
            ui: ui,
            spritemap: sprite,
            tilemap: tile,
            viewport: viewport,
        }
    }

    pub fn update(&mut self, board: &Board) {
        self.tilemap.update(board);
    }

    pub fn refresh_shaders(&mut self) {
        self.background.refresh_shaders(&self.backend);
    }

    pub fn render(&mut self, duration: &Duration) {
        let mut target = self.backend.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let millis = util::get_duration_millis(duration);
        self.background.render(&self.backend, &mut target, &self.viewport, millis);

        self.tilemap.render(&self.backend, &mut target, &self.viewport, millis);
        self.spritemap.render(&self.backend, &mut target, &self.viewport, millis);
        self.ui.render(&self.backend, &mut target, &self.viewport, millis);

        target.finish().unwrap();
    }

    pub fn set_viewport(&mut self, w: u32, h: u32) {
        self.viewport = Viewport {
            position: (0, 0),
            size: (w, h),
            scale: self.viewport.scale,
            camera: self.viewport.camera,
        };
    }

    pub fn poll_events(&self) -> Vec<glutin::Event> {
        self.backend.poll_events().collect()
    }

    pub fn ui_active(&self) -> bool {
        self.ui.is_active()
    }

    pub fn update_ui(&mut self, event: glutin::Event) {
        self.ui.update(event);
    }

    pub fn query<R, T: 'static + UiQuery<QueryResult=R>>(&mut self, layer: &mut T) -> R {
        loop {
            for event in self.backend.poll_events() {
                match layer.on_event(event) {
                    EventResult::Done => {
                        self.ui.clear();
                        self.ui.invalidate();
                        return layer.result();
                    },
                    _ => self.ui.draw_layer(layer),
                }
            }

            self.render(&Duration::new(0, 0));
        }
    }
}

pub trait Renderable {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface;
}
