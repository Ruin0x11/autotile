extern crate rusttype;
extern crate cgmath;
#[macro_use] extern crate glium;
extern crate glium_text;
extern crate image;
extern crate rand;
extern crate texture_packer;

mod atlas_frame;
mod background;
mod board;
mod font;
mod point;
mod spritemap;
mod terrain;
mod texture_atlas;
mod tilemap;
mod ui;
mod util;

use std::thread;
use std::time::{Duration, Instant};

use glium::glutin;
use glium::glutin::{VirtualKeyCode, ElementState};
use glium::{DisplayBuild, Surface};
use glium::backend::glutin_backend::GlutinFacade;

use point::{Point, RectangleIter};

use background::Background;
use board::Board;
use terrain::Terrain;
use ui::*;
use spritemap::SpriteMap;
use tilemap::TileMap;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

#[derive(Debug)]
pub struct Viewport {
    position: (u32, u32),
    size: (u32, u32),
    scale: f32,
    camera: (i32, i32),
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
        let tile = TileMap::new(&display, "./data/map.png");
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

        let millis = get_duration_millis(duration);
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

fn get_duration_millis(duration: &Duration) -> u64 {
    let nanos = duration.subsec_nanos() as u64;
    (1000*1000*1000 * duration.as_secs() + nanos)/(1000 * 1000)
}

fn main() {
    let mut board = Board::new(20, 20, Terrain::Wall);

    for pos in RectangleIter::new(Point::new(2, 2), Point::new(8, 8)) {
        board.set(&pos, Terrain::Floor);
    }

    for pos in RectangleIter::new(Point::new(6, 6), Point::new(10, 10)) {
        board.set(&pos, Terrain::Important);
    }

    board.set(&Point::new(6, 6), Terrain::Wall);
    // board.set(&Point::new(5, 6), Terrain::Wall);
    // board.set(&Point::new(7, 6), Terrain::Wall);
    board.set(&Point::new(6, 5), Terrain::Wall);
    board.set(&Point::new(6, 7), Terrain::Wall);

    let mut ctxt = RenderContext::new();

    ctxt.update(&board);

    start_loop(|duration| {
        // polling and handling the events received by the window
        for event in ctxt.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,
                glutin::Event::Resized(w, h) => {
                    ctxt.set_viewport(w, h);
                    return Action::Continue;
                },
                _ => (),
            }

            if ctxt.ui_active() {
                ctxt.update_ui(event);
                return Action::Continue;
            }

            match event {
                glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                    println!("Key: {:?}", code);
                    match code {
                        VirtualKeyCode::Escape |
                        VirtualKeyCode::Q => {
                            return Action::Stop;
                        },
                        VirtualKeyCode::I => {
                            let res = ctxt.query(&mut InvLayer::new());
                            println!("{}", res);
                        },
                        VirtualKeyCode::Left => {
                            ctxt.viewport.camera.0 -= 48;
                        },
                        VirtualKeyCode::Up => {
                            ctxt.viewport.camera.1 -= 48;
                        },
                        VirtualKeyCode::Down => {
                            ctxt.viewport.camera.1 += 48;
                        },
                        VirtualKeyCode::Right => {
                            ctxt.viewport.camera.0 += 48;
                        },
                        VirtualKeyCode::R => {
                            ctxt.refresh_shaders();
                        },
                        _ => (),
                    }
                },
                _ => ()
            }
        }

        ctxt.render(duration);

        Action::Continue
    })
}
pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(mut callback: F) where F: FnMut(&Duration) -> Action {
    let start = Instant::now();
    let mut frame_count: u32 = 0;
    let mut last_time: u64 = 0;
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();

    loop {
        match callback(&Instant::now().duration_since(start)) {
            Action::Stop => break,
            Action::Continue => ()
        };

        let now = Instant::now();
        accumulator += now - previous_clock;
        previous_clock = now;

        let fixed_time_stamp = Duration::new(0, 16666667);
        while accumulator >= fixed_time_stamp {
            accumulator -= fixed_time_stamp;

            // if you have a game, update the state here
        }

        let millis = get_duration_millis(&Instant::now().duration_since(start));

        if millis - last_time >= 1000 {
            let ms_per_frame = 1000.0 / frame_count as f32;
            println!("{} ms/frame | {} fps", ms_per_frame, 1000.0 / ms_per_frame);
            frame_count = 0;
            last_time += 1000;
        }

        thread::sleep(fixed_time_stamp - accumulator);

        frame_count += 1;
    }
}
