use std::collections::HashSet;
use std::time::Duration;

use cgmath;
use glium;
use glium::glutin;
use glium::{DisplayBuild, Surface};
use glium::backend::glutin_backend::GlutinFacade;
use glium::backend::Facade;


use board::Board;
use point::{Point, CircleIter, RectangleIter};
use ui::*;
use util;
use self::background::Background;
use self::shadowmap::ShadowMap;
use self::spritemap::SpriteMap;
use self::tilemap::TileMap;

mod background;
mod shadowmap;
mod spritemap;
mod tilemap;

pub fn load_program<F: Facade>(display: &F, vert: &str, frag: &str) -> Result<glium::Program, glium::ProgramCreationError> {
    let vertex_shader = util::read_string(&format!("data/shaders/{}", vert));
    let fragment_shader = util::read_string(&format!("data/shaders/{}", frag));

    glium::Program::from_source(display, &vertex_shader, &fragment_shader, None)
}

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

pub type RendererSubarea = ([[f32; 4]; 4], glium::Rect);

impl Viewport {
    pub fn main_window(&self) -> RendererSubarea {
        let (w, h) = self.scaled_size();
        self.make_subarea((0, 0, w, h - 120))
    }

    pub fn scaled_size(&self) -> (u32, u32) {
        ((self.size.0 as f32 * self.scale) as u32, (self.size.1 as f32 * self.scale) as u32)
    }

    fn make_subarea(&self, area: (u32, u32, u32, u32)) -> RendererSubarea {
        (self.camera_projection(), self.scissor(area))
    }

    pub fn static_projection(&self) -> [[f32; 4]; 4] {
        self.make_projection_matrix((0, 0))
    }

    pub fn camera_projection(&self) -> [[f32; 4]; 4] {
        self.make_projection_matrix(self.camera)
    }

    fn make_projection_matrix(&self, offset: (i32, i32)) -> [[f32; 4]; 4] {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let (x, y) = (offset.0 as f32, offset.1 as f32);

        let left = x;
        let right = x + w;
        let bottom = y + h;
        let top = y;

        cgmath::ortho(left, right, bottom, top, -1.0, 1.0).into()
    }

    fn scissor(&self, area: (u32, u32, u32, u32)) -> glium::Rect {
        let (ax, ay, aw, ah) = area;
        let (_, h) = self.scaled_size();
        let conv = |i| (i as f32 * self.scale) as u32;

        glium::Rect { left:   conv(ax),
                      bottom: conv(ay) + conv(h - ah),
                      width:  conv(aw - ax),
                      height: conv(ah) - conv(ay * 2),
        }
    }
}

pub struct RenderContext {
    backend: GlutinFacade,
    ui: Ui,

    background: Background,
    spritemap: SpriteMap,
    tilemap: TileMap,
    shadowmap: ShadowMap,

    // msecs_elapsed: u64,
    pub viewport: Viewport,
}

impl RenderContext {
    pub fn new() -> Self{
        let display = glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
            .with_title("Tile")
            .build_glium()
            .unwrap();

        let bg = Background::new(&display);
        let ui = Ui::new(&display);
        let tile = TileMap::new(&display);

        let mut vis = HashSet::new();
        for point in CircleIter::new(Point::new(6, 6), 5) {
            vis.insert(point);
        }

        let shadow = ShadowMap::new(&display, RectangleIter::new(Point::new(0, 0), Point::new(20, 20)), vis);

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
            shadowmap: shadow,
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
        self.shadowmap.render(&self.backend, &mut target, &self.viewport, millis);
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

    pub fn update_ui(&mut self, event: &glutin::Event) -> bool {
        if self.ui.is_active() {
            self.ui.on_event(event.clone());
            self.ui.update();
            return true;
        } else {
            self.ui.update();
            return false;
        }
    }

    pub fn query<R, T: 'static + UiQuery<QueryResult=R>>(&mut self, layer: &mut T) -> R {
        loop {
            for event in self.backend.poll_events() {
                match layer.on_event(event) {
                    EventResult::Done => {
                        self.ui.render_all();
                        return layer.result();
                    },
                    _ => {
                        self.ui.render_all();
                        self.ui.draw_layer(layer);
                    }
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
