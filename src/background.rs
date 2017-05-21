use glium;
use glium::index::PrimitiveType;

use tilemap::{Vertex, QUAD_INDICES};
use util;

pub const BG_QUAD: [Vertex; 4] = [
    Vertex { position: [-1, 1], },
    Vertex { position: [1, 1], },
    Vertex { position: [-1, -1], },
    Vertex { position: [1, -1], },
];


pub fn render_background<F, S>(display: &F, target: &mut S, viewport: &::Viewport, msecs: u64)
    where F: glium::backend::Facade, S: glium::Surface {
    let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);

    let vertices = glium::VertexBuffer::immutable(display, &BG_QUAD).unwrap();
    let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();
    let vertex_shader = util::read_string("./data/bg.vert");
    let fragment_shader = util::read_string("./data/bg.frag");

    let program = glium::Program::from_source(display, &vertex_shader, &fragment_shader, None).unwrap();

    let uniforms = uniform! {
        u_resolution: [w, h],
        u_time: msecs as f32 / 1000.0,
    };

    let params = glium::DrawParameters {
        .. Default::default()
    };

    target.draw(&vertices,
                &indices,
                &program,
                &uniforms,
                &params).unwrap();
}
