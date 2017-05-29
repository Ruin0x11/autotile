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
use render::{Action, RenderContext};

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
                        println!("Go");
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

    let mut context = RenderContext::new();

    context.update(&board);

    context.start_loop(|ctxt| {
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
                        VirtualKeyCode::A => {
                            ctxt.message("Live, die, repeat.");
                        },
                        VirtualKeyCode::N => {
                            ctxt.next_line();
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

        ctxt.render();

        Action::Continue
    })
}
