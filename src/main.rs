#[macro_use] extern crate glium;
extern crate bincode;
extern crate cgmath;
extern crate crypto;
extern crate glium_text;
extern crate glob;
extern crate image;
extern crate rand;
extern crate rusttype;
extern crate texture_packer;
extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;

mod atlas;
mod render;
mod board;
mod terrain;
mod util;
mod ui;
mod point;

use std::thread;
use std::time::{Duration, Instant};

use glium::glutin;
use glium::glutin::{VirtualKeyCode, ElementState};

use point::{Point, RectangleIter};

use board::Board;
use terrain::Terrain;
use ui::*;
use ui::elements::UiList;
use render::RenderContext;


pub struct InvLayer {
    list: UiList,
}

impl InvLayer {
    pub fn new() -> Self {
        InvLayer {
            list: UiList::new((100, 100), vec!["Dood", "Hello, my dear", "end of days", "something", "something else", "starfruit"]),
        }
    }
}

impl UiElement for InvLayer {
    fn draw(&self, renderer: &mut UiRenderer) {
        self.list.draw(renderer);
    }
}

impl UiLayer for InvLayer {
    fn on_event(&mut self, event: glutin::Event) -> EventResult {
        match event {
            glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                match code {
                    VirtualKeyCode::Escape |
                    VirtualKeyCode::Return |
                    VirtualKeyCode::Q => {
                        EventResult::Done
                    },
                    VirtualKeyCode::Up => {
                        self.list.select_prev();
                        EventResult::Consumed(None)
                    },
                    VirtualKeyCode::Down => {
                        self.list.select_next();
                        EventResult::Consumed(None)
                    },
                    _ => EventResult::Ignored,
                }
            },
            _ => EventResult::Ignored,
        }
    }
}

impl UiQuery for InvLayer {
    type QueryResult = String;

    fn result(&self) -> String {
        self.list.get_selected().unwrap().text()
    }
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

            if ctxt.update_ui(&event) {
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

        let millis = util::get_duration_millis(&Instant::now().duration_since(start));

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
