extern crate cgmath;
#[macro_use] extern crate glium;
extern crate image;
extern crate rand;

mod board;
mod point;
mod terrain;
mod texture_atlas;
mod tilemap;

use std::thread;
use std::time::{Duration, Instant};

use glium::glutin;
use glium::{DisplayBuild, Surface};

use point::{Point, RectangleIter};

use board::Board;
use terrain::Terrain;
use tilemap::Tilemap;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

pub struct Viewport {
    position: (u32, u32),
    size: (u32, u32),
}

pub trait Renderable {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport)
        where F: glium::backend::Facade, S: glium::Surface;
}

fn main() {
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium()
        .unwrap();

    let mut board = Board::new(20, 20, Terrain::Wall);

    for pos in RectangleIter::new(Point::new(4, 4), Point::new(8, 8)) {
        board.set(&pos, Terrain::Floor);
    }

    board.set(&Point::new(6, 6), Terrain::Wall);
    board.set(&Point::new(5, 6), Terrain::Wall);
    board.set(&Point::new(7, 6), Terrain::Wall);
    board.set(&Point::new(6, 5), Terrain::Wall);
    board.set(&Point::new(6, 7), Terrain::Wall);

    let tile = Tilemap::new(&display, &board, "./data/map.png");

    let mut viewport = Viewport { position: (0, 0), size: (SCREEN_WIDTH, SCREEN_HEIGHT) };

    start_loop(|| {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        tile.render(&display, &mut target, &viewport);
        target.finish().unwrap();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,
                glutin::Event::Resized(w, h) => {
                    viewport = Viewport { position: (0, 0), size: (w, h) };
                },
                _ => ()
            }
        }

        Action::Continue
    })
}
pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(mut callback: F) where F: FnMut() -> Action {
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();

    loop {
        match callback() {
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

        thread::sleep(fixed_time_stamp - accumulator);
    }
}
