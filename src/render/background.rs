use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;

use render::{Renderable, Viewport, Vertex, QUAD_INDICES};
use util;

pub const BG_QUAD: [Vertex; 4] = [
    Vertex { position: [-1, 1], },
    Vertex { position: [1, 1], },
    Vertex { position: [-1, -1], },
    Vertex { position: [1, -1], },
];

pub struct Background {
    program: glium::Program,
}

fn load_program<F: Facade>(display: &F, vert: &str, frag: &str) -> Result<glium::Program, glium::ProgramCreationError> {
    let vertex_shader = util::read_string(vert);
    let fragment_shader = util::read_string(frag);

    glium::Program::from_source(display, &vertex_shader, &fragment_shader, None)
}

impl Background {
    pub fn new<F: Facade>(display: &F) -> Self {
        Background {
            program: load_program(display, "data/bg.vert", "data/bg.frag").unwrap(),
        }
    }

    pub fn refresh_shaders<F: Facade>(&mut self, display: &F) {
        match load_program(display, "data/bg.vert", "data/bg.frag") {
            Ok(program) => self.program = program,
            Err(e)      => println!("Shader error: {:?}", e),
        }
    }
}

impl Renderable for Background {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: Facade, S: glium::Surface {
        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let scale = viewport.scale;

        let vertices = glium::VertexBuffer::immutable(display, &BG_QUAD).unwrap();
        let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();

        let uniforms = uniform! {
            u_resolution: [w * scale, h * scale],
            u_time: msecs as f32 / 1000.0,
        };

        let params = glium::DrawParameters {
            .. Default::default()
        };

        target.draw(&vertices,
                    &indices,
                    &self.program,
                    &uniforms,
                    &params).unwrap();
    }
}

